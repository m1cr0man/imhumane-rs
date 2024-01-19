use std::{fs::File, io::BufReader, sync::Arc, thread};

use axum::Router;
use clap_serde_derive::ClapSerde;
use tokio::runtime::Handle;

use crate::service::{config::Config, ImHumane};

use clap::Parser;

fn app(service: Arc<ImHumane>) -> Router {
    crate::http::get_router(service)
}

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    // TODO make optional
    /// Config file
    #[arg(short, long = "config", default_value = "config.json")]
    config_path: std::path::PathBuf,

    /// Rest of arguments
    #[command(flatten)]
    pub config: <Config as clap_serde_derive::ClapSerde>::Opt,
}

pub(crate) async fn main() {
    let mut args = Args::parse();

    let config = if let Ok(cfg_file) = File::open(&args.config_path) {
        match serde_json::from_reader::<_, <Config as ClapSerde>::Opt>(BufReader::new(cfg_file)) {
            Ok(config) => Config::from(config).merge(&mut args.config),
            Err(err) => panic!("Error in configuration file:\n{err:#?}"),
        }
    } else {
        Config::from(&mut args.config)
    };

    if config.buffer_size < 1 {
        panic!("Buffer size must be >= 1");
    }
    if config.threads < 1 {
        panic!("Threads must be >= 1");
    }

    let service = Arc::new(ImHumane::from(&config));
    service
        .scan_for_collections(&config.images_directory)
        .unwrap();

    // Start threads for the challenge generators
    let handle = Handle::try_current().expect("Failed to get handle for current tokio runtime");
    let mut threads = Vec::new();
    for _ in 1..config.threads {
        let svc = service.clone();
        let handle = handle.clone();
        threads.push(thread::spawn(move || svc.run_generator(handle)))
    }

    // Start the web server
    let addr = config.address;
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app(service).into_make_service())
        .await
        .unwrap();

    threads.into_iter().for_each(|t| t.join().unwrap());
}
