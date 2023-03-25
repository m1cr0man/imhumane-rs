use std::{fs::File, path::PathBuf};

use fs2::FileExt;

pub(super) struct LockedFile {
    pub(super) file: File,
}

impl LockedFile {
    pub(super) fn open_rw_no_truncate(path: PathBuf) -> Result<Self, std::io::Error> {
        File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?
            .try_into()
    }
}

impl TryFrom<File> for LockedFile {
    type Error = std::io::Error;

    fn try_from(value: File) -> Result<Self, Self::Error> {
        value.lock_exclusive()?;
        Ok(Self { file: value })
    }
}

impl Drop for LockedFile {
    fn drop(&mut self) {
        self.file
            .unlock()
            .expect("Failed to unlock previously locked file")
    }
}
