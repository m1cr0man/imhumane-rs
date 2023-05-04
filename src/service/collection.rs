use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Collection {
    // pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) images: Vec<PathBuf>,
}
