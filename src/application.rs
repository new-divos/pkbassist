use std::{ffi::OsStr, path};

use futures::future::join_all;
use tokio::{fs::File, io::AsyncReadExt};
use walkdir::WalkDir;

use crate::{
    config::{Arguments, Config},
    error::Error,
};

///
/// The command line application.
///
#[derive(Debug)]
pub struct Application {
    config: Config,
}

impl Application {
    ///
    /// Create command line application with configuration.
    ///
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    ///
    /// Run the application.
    ///
    pub async fn run(&self, args: &Arguments) -> Result<(), Error> {
        match args.command {
            crate::config::Command::Repair { wiki_refs } => {
                if wiki_refs {
                    self.repair_wiki_refs().await?;
                }
            }
        }

        Ok(())
    }

    ///
    /// Repair wiki references.
    ///
    async fn repair_wiki_refs(&self) -> Result<(), Error> {
        let errors: Vec<_> = join_all(
            WalkDir::new(self.config.root())
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(OsStr::to_str) == Some("md"))
                .map(|e| self.repair_file_wiki_refs(path::PathBuf::from(e.path()))),
        )
        .await
        .into_iter()
        .filter_map(|r| r.err())
        .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Repair file wiki references.
    ///
    async fn repair_file_wiki_refs(&self, path: path::PathBuf) -> Result<(), Error> {
        log::info!("Start processing of the file \"{}\"", path.display());
        let mut buffer = String::new();
        {
            let mut file = File::open(path.as_path()).await?;
            file.read_to_string(&mut buffer).await?;
        }

        log::info!("Finish processing of the file \"{}\"", path.display());
        Ok(())
    }
}
