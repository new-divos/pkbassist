use std::{ffi::OsStr, iter::repeat_with, sync::Arc};

use futures::future::join_all;
use regex::Regex;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use walkdir::WalkDir;

use crate::{
    config::{Arguments, Command, Config},
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
            Command::Repair { wiki_refs } => {
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
        let re = Arc::new(
            Regex::new(
                r"\[\[\s*(?P<file>[A-Za-z\d\-\.]+(?:\s+[\w\d\-_\.\(\)]+)*)\s*\|\s+(?P<descr>.[^\[\]]+)\s*?\]\]",
            )
            .unwrap(),
        );
        let errors: Vec<_> = join_all(
            WalkDir::new(self.config.root())
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(OsStr::to_str) == Some("md"))
                .zip(repeat_with(|| re.clone()))
                .map(|(e, re)| async move {
                    log::info!("Start processing of the file \"{}\"", e.path().display());
                    let mut buffer = String::new();
                    {
                        let mut file = File::open(e.path()).await?;
                        file.read_to_string(&mut buffer).await?;
                    }

                    let content = re.replace_all(&buffer, "[[$file|$descr]]");
                    {
                        let mut file = File::create(e.path()).await?;
                        file.write_all(content.as_bytes()).await?;
                    }

                    log::info!("Finish processing of the file \"{}\"", e.path().display());
                    Ok(())
                }),
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
}
