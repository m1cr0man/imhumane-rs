use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

#[cfg(feature = "cli")]
use clap_serde_derive::ClapSerde;

#[cfg(not(feature = "cli"))]
const fn default_three() -> usize {
    3
}

#[cfg(not(feature = "cli"))]
const fn default_eight() -> usize {
    8
}

#[cfg(not(feature = "cli"))]
const fn default_size() -> usize {
    96
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    feature = "cli",
    derive(clap_serde_derive::clap::Parser, ClapSerde),
    command(author, version, about)
)]
pub struct Config {
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
    #[cfg_attr(
        feature = "cli",
        default(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3001))),
        arg(
            short,
            long,
            // Sadly, clap_serde_derive defaults aren't used in clap's help output.
            // Emulate it via help text.
            help = "Address to listen on. [default: 0.0.0.0:3001]",
            env = "IMHUMANE_ADDRESS",
        )
    )]
    pub address: SocketAddr,
    #[cfg_attr(not(feature = "cli"), serde(default = "default_eight"))]
    #[cfg_attr(
        feature = "cli",
        default(8),
        arg(
            short,
            long,
            help = "Number of images to pre-generate. Must be >= 1. [default: 8]",
            env = "IMHUMANE_BUFFER_SIZE"
        )
    )]
    pub buffer_size: usize,
    #[cfg_attr(not(feature = "cli"), serde(default = "default_eight"))]
    #[cfg_attr(
        feature = "cli",
        default(8),
        arg(
            short,
            long,
            help = "Number of image generation threads. Must be >= 1. [default: 8]",
            env = "IMHUMANE_THREADS"
        )
    )]
    pub threads: usize,
    #[cfg_attr(not(feature = "cli"), serde(default = "default_size"))]
    #[cfg_attr(
        feature = "cli",
        default(96),
        arg(
            long,
            help = "Size of each image in a challenge, in pixels. [default: 96]",
            env = "IMHUMANE_IMAGE_SIZE"
        )
    )]
    pub image_size: u32,
    #[cfg_attr(not(feature = "cli"), serde(default = "default_eight"))]
    #[cfg_attr(
        feature = "cli",
        default(8),
        arg(
            long,
            help = "Size of the gap between images, in pixels. [default: 8]",
            env = "IMHUMANE_GAP_SIZE"
        )
    )]
    pub gap_size: u32,
    #[cfg_attr(not(feature = "cli"), serde(default = "default_three"))]
    #[cfg_attr(
        feature = "cli",
        default(3),
        arg(
            long,
            help = "The length of the grid - How many images per row + column. [default: 3]",
            env = "IMHUMANE_GRID_LENGTH"
        )
    )]
    pub grid_length: u32,
}
