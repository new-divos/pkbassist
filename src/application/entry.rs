use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use uuid::Uuid;

///
/// The file entry.
///
#[derive(Debug)]
pub(crate) struct FileEntry {
    // The old file path.
    old_path: PathBuf,
    // The old file name.
    old_name: String,
    // The new file path.
    new_path: PathBuf,
    // The new file name.
    new_name: String,
}

impl FileEntry {
    ///
    /// Create a new file entry.
    ///
    pub(crate) fn new<P: AsRef<Path>>(path: P, id: Uuid) -> Option<Self> {
        let old_path = PathBuf::from(path.as_ref());
        if let Some(old_name) = old_path.file_name().and_then(OsStr::to_str) {
            let old_name = old_name.to_string();

            let mut new_path = old_path.clone();
            new_path.set_file_name(&id.to_string());
            if let Some(ext) = old_path.extension() {
                new_path.set_extension(ext);
            }

            if let Some(new_name) = new_path.file_name().and_then(OsStr::to_str) {
                let new_name = new_name.to_string();

                return Some(Self {
                    old_path,
                    old_name,
                    new_path,
                    new_name,
                });
            }
        }

        None
    }

    ///
    /// Get the old file path.
    ///
    #[inline]
    pub(crate) fn old_path(&self) -> &Path {
        self.old_path.as_path()
    }

    ///
    /// Get the old file name.
    ///
    #[inline]
    pub(crate) fn old_name(&self) -> &str {
        self.old_name.as_str()
    }

    ///
    /// Get the new file path.
    ///
    #[inline]
    pub(crate) fn new_path(&self) -> &Path {
        self.new_path.as_path()
    }

    ///
    /// Get the new file name.
    ///
    #[inline]
    pub(crate) fn new_name(&self) -> &str {
        self.new_name.as_str()
    }
}
