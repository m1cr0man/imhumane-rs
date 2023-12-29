use figment::{Error, Figment, Metadata, Profile, Provider};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "cli", derive(clap::Parser))]
pub struct Config {
    #[cfg_attr(feature = "cli", arg(
        short,
        long,
        default_value_t=SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3001)),
        help = "Address to listen on",
        env = "IMHUMANE_ADDRESS"
    ))]
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
    #[cfg_attr(
        feature = "cli",
        arg(
            short,
            long,
            default_value_t = 8,
            help = "Number of images to pre-generate. Must be >= 1",
            env = "IMHUMANE_BUFFER_SIZE"
        )
    )]
    pub buffer_size: usize,
    #[cfg_attr(
        feature = "cli",
        arg(
            short,
            long,
            default_value_t = 8,
            help = "Number of image generation threads. Must be >= 1",
            env = "IMHUMANE_THREADS"
        )
    )]
    pub threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3001)),
            images_directory: "images".into(),
            buffer_size: 8,
            threads: 8,
        }
    }
}

impl Config {
    pub fn from<T: Provider>(provider: T) -> Result<Config, Error> {
        Figment::from(provider).extract()
    }

    pub fn figment() -> Figment {
        #[cfg(feature = "env")]
        use figment::providers::Env;
        let fig = Figment::from(Config::default());
        #[cfg(feature = "env")]
        let fig = fig.merge(Env::prefixed("IMHUMANE_"));
        fig
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("ImHumane Config")
    }

    fn data(&self) -> Result<figment::value::Map<Profile, figment::value::Dict>, Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }
}
