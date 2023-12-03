use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(about="I'm Humane is an anti bot form validator.", version=env!("CARGO_PKG_VERSION"))]
pub struct ImHumaneCli {
    #[arg(
        short,
        long,
        default_value_t=SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3001)),
        help = "Address to listen on",
        env = "IMHUMANE_ADDRESS"
    )]
    pub address: SocketAddr,
    #[arg(
        short,
        long,
        help = "Directory containing images to use for validation",
        env = "IMHUMANE_IMAGES_DIRECTORY"
    )]
    pub images_directory: PathBuf,
    #[arg(
        short,
        long,
        default_value_t = 8,
        help = "Number of images to pre-generate. Must be >= 1",
        env = "IMHUMANE_BUFFER_SIZE"
    )]
    pub buffer_size: usize,
    #[arg(
        short,
        long,
        default_value_t = 8,
        help = "Number of image generation threads. Must be >= 1",
        env = "IMHUMANE_THREADS"
    )]
    pub threads: usize,
}
