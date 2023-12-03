mod config;
pub mod http;
pub mod service;

use std::{sync::Arc, thread};

use axum::Router;
use clap::Parser;
use tokio::runtime::Handle;

use crate::service::ImHumane;

fn app(service: Arc<ImHumane>) -> Router {
    crate::http::get_router(service)
}

#[tokio::main]
async fn main() {
    let cli = crate::config::ImHumaneCli::parse();

    if cli.buffer_size < 1 {
        panic!("Buffer size must be >= 1");
    }
    if cli.threads < 1 {
        panic!("Threads must be >= 1");
    }

    let service = Arc::new(ImHumane::new(cli.buffer_size));
    service.scan_for_collections(&cli.images_directory).unwrap();

    // Start threads for the challenge generators
    let handle = Handle::try_current().expect("Failed to get handle for current tokio runtime");
    let mut threads = Vec::new();
    for _ in 1..8 {
        let svc = service.clone();
        let handle = handle.clone();
        threads.push(thread::spawn(move || svc.run_generator(handle)))
    }

    // Start the web server
    let addr = cli.address;
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app(service).into_make_service())
        .await
        .unwrap();

    threads.into_iter().for_each(|t| t.join().unwrap());
}
