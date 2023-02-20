use std::path::{Path, PathBuf};

use image::ImageError;
use snafu::prelude::*;


#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not scan for collections in {path}"))]
    Scan { path: String, source: std::io::Error },
    #[snafu(display("Could not convert a folder name to a string: {path}"))]
    CollectionName { path: String },
    #[snafu(display("Could not read image {path}"))]
    ReadImage { path: String, source: std::io::Error },
    #[snafu(display("Could not lock state for key {key}"))]
    StateLock { key: String },
    #[snafu(display("Insufficient collections for a valid question"))]
    InsufficientCollections,
    #[snafu(display("Failed to generate collage image: {source}"))]
    GenerateImage { source: ImageError },
    #[snafu(display("Failed to open image {path}"))]
    OpenImage { path: String, source: ImageError },
}

impl From<&Path> for ScanSnafu<String> {
    fn from(value: &Path) -> Self {
        Self {
            path: value.display().to_string()
        }
    }
}

impl From<&Path> for CollectionNameSnafu<String> {
    fn from(value: &Path) -> Self {
        Self {
            path: value.display().to_string()
        }
    }
}

impl From<&PathBuf> for OpenImageSnafu<String> {
    fn from(value: &PathBuf) -> Self {
        Self {
            path: value.display().to_string()
        }
    }
}
