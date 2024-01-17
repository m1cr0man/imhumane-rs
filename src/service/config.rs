use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

#[cfg(feature = "cli")]
use clap_serde_derive::ClapSerde;

#[cfg(not(feature = "cli"))]
fn default_address() -> SocketAddr {
    SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3001))
}

#[cfg(not(feature = "cli"))]
const fn default_counts() -> usize {
    8
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    feature = "cli",
    derive(clap_serde_derive::clap::Parser, ClapSerde),
    command(author, version, about)
)]
pub struct Config {
    #[cfg_attr(not(feature = "cli"), serde(default = "default_address"))]
    #[cfg_attr(
        feature = "cli",
        default(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3001))),
        arg(
            short,
            long,
            default_value = "0.0.0.0:3001",
            help = "Address to listen on",
            env = "IMHUMANE_ADDRESS",
        )
    )]
    pub address: SocketAddr,
    #[cfg_attr(
        feature = "cli",
        arg(
            short,
            long,
            help = "Directory containing images to use for validation",
            env = "IMHUMANE_IMAGES_DIRECTORY"
        )
    )]
    pub images_directory: PathBuf,
    #[cfg_attr(not(feature = "cli"), serde(default = "default_counts"))]
    #[cfg_attr(
        feature = "cli",
        default(8),
        arg(
            short,
            long,
            default_value = "8",
            help = "Number of images to pre-generate. Must be >= 1",
            env = "IMHUMANE_BUFFER_SIZE"
        )
    )]
    pub buffer_size: usize,
    #[cfg_attr(not(feature = "cli"), serde(default = "default_counts"))]
    #[cfg_attr(
        feature = "cli",
        default(8),
        arg(
            short,
            long,
            default_value = "8",
            help = "Number of image generation threads. Must be >= 1",
            env = "IMHUMANE_THREADS"
        )
    )]
    pub threads: usize,
}
