use std::net::IpAddr;
use std::{env, error::Error, path::PathBuf, process::exit, sync::Arc, thread};

use axum::body::Bytes;
use axum::extract::{Path, Request};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::any};
use axum::{middleware, Extension, Router};
use std::io::Write;
use tokio::net::TcpStream;
use tokio::runtime::Handle;

use crate::service::{config::Config, ImHumane};

use clap::{crate_authors, crate_description, crate_version, Arg, ArgAction, Command};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct AppConfig {
    listener_address: tokio_listener::ListenerAddress,
    images_directory: PathBuf,
    threads: usize,
    protected_proxy_to: Option<String>,
}

fn parse_config<'a, T: serde::Deserialize<'a>>(prefix: &str) -> Result<T, Box<dyn Error>> {
    let cfg_source = config::Config::builder()
        .add_source(
            config::Environment::with_prefix(prefix)
                .convert_case(config::Case::ScreamingSnake)
                .try_parsing(true),
        )
        .build()?;

    cfg_source.try_deserialize().map_err(|err| {
        tracing::error!("Error in the provided configuration: {}", err);
        exit(2);
    })
}

fn app(service: Arc<ImHumane>, backend: Option<String>) -> Router {
    let router = crate::http::get_router(service);

    if let Some(backend_url) = backend {
        router.route(
            "/proxy/*rest",
            any(reverse_proxy_handler)
                .with_state(backend_url.trim_end_matches("/").to_string())
                .layer(middleware::from_fn(
                    crate::http::router::token_validate_middleware,
                )),
        )
    } else {
        router
    }
}

async fn reverse_proxy_handler(
    State(backend): State<String>,
    client_ip: IpAddr,
    req: Request,
) -> Response {
    // TODO hyper_reverse_proxy uses a really old version of http crate.
    // I can't just pass req to the reverse proxy as a result.
    hyper_reverse_proxy::call(client_ip, &backend, req).await
}

fn setup_logger() {
    // Set a default level
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    // Adapted from env_logger examples. <3 Systemd support
    match std::env::var("RUST_LOG_STYLE") {
        Ok(s) if s == "SYSTEMD" => env_logger::builder()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "<{}>{}: {}",
                    match record.level() {
                        log::Level::Error => 3,
                        log::Level::Warn => 4,
                        log::Level::Info => 6,
                        log::Level::Debug => 7,
                        log::Level::Trace => 7,
                    },
                    record.target(),
                    record.args()
                )
            })
            .init(),
        _ => pretty_env_logger::init(),
    };
}

pub(crate) async fn main() {
    let cli = Command::new("ImHumane")
        .about(format!(
            "{}\n{} {}",
            crate_description!(),
            "Configuration is managed using environment variables.",
            "See the docs for more information.",
        ))
        .arg(
            Arg::new("check")
                .action(ArgAction::SetTrue)
                .short('c')
                .long("check")
                .help("Check the configuration"),
        )
        .version(crate_version!())
        .author(crate_authors!("\n"));

    let args = cli.get_matches();

    setup_logger();

    let app_config: AppConfig = parse_config("IMHUMANE").unwrap();
    let config: Config = parse_config("IMHUMANE").unwrap();
    let user_opts: tokio_listener::UserOptions = parse_config("IMHUMANE_LISTENER").unwrap();

    if config.buffer_size < 1 {
        tracing::error!("Buffer size must be >= 1");
        exit(2);
    }

    if app_config.threads < 1 {
        tracing::error!("Threads must be >= 1");
        exit(2);
    }

    if args.get_flag("check") {
        tracing::info!("Configuration is valid.");
        exit(0);
    }

    let service = Arc::new(ImHumane::from(&config));
    service
        .scan_for_collections(&app_config.images_directory)
        .unwrap();

    // Start threads for the challenge generators
    let handle = Handle::try_current().expect("Failed to get handle for current tokio runtime");
    let mut threads = Vec::new();
    for _ in 1..app_config.threads {
        let svc = service.clone();
        let handle = handle.clone();
        threads.push(thread::spawn(move || svc.run_generator(handle)))
    }

    let app = app(service, app_config.protected_proxy_to);

    // Start the web server
    let listener = tokio_listener::Listener::bind(
        &app_config.listener_address,
        &tokio_listener::SystemOptions::default(),
        &user_opts,
    )
    .await
    .map_err(|err| {
        tracing::error!("Failed to configure listener: {}", err);
        exit(3);
    })
    .unwrap();

    tracing::info!("Listening on {}", app_config.listener_address);
    tokio_listener::axum07::serve(listener, app.into_make_service())
        .await
        .unwrap();

    threads.into_iter().for_each(|t| t.join().unwrap());
}
