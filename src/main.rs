#![feature(type_alias_impl_trait)]
pub mod imhumane;
pub mod http;

use std::{net::{Ipv4Addr, SocketAddr}, path:: Path, thread, sync::Arc};

use axum::Router;
use tokio::runtime::Handle;

use crate::imhumane::ImHumane;

fn app(service: Arc<ImHumane>) -> Router {
    crate::http::get_router(service)
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3000));
    tracing::info!("Listening on {}", addr);

    let service = Arc::new(ImHumane::new(8));
    service.scan_for_collections(Path::new("images")).unwrap();

    // Start threads for the challenge generators
    let handle = Handle::try_current().expect("Failed to get handle for current tokio runtime");
    let mut threads = Vec::new();
    for _ in 1..8 {
        let svc = service.clone();
        let handle = handle.clone();
        threads.push(thread::spawn(move || svc.run_generator(handle)))
    }

    // Start the web server
    axum::Server::bind(&addr)
        .serve(app(service).into_make_service())
        .await
        .unwrap();

    threads.into_iter().for_each(|t| t.join().unwrap());
}
