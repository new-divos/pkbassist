use std::{ffi::OsStr, io::Cursor, iter::repeat_with, path::PathBuf, sync::Arc};

use futures::future::join_all;
use regex::Regex;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use url::Url;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    config::{Arguments, Command, Config, GrabNote},
    error::Error,
};

pub(crate) mod grab;

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
            // Repair notes set.
            Command::Repair { wiki_refs } => {
                if wiki_refs {
                    self.repair_wiki_refs().await?;
                }
            }

            // Grab note into notes set.
            Command::Grab { ref note } => match note {
                // Grab NASA Astronomy Picture of the Day.
                GrabNote::APoD { update_daily } => self.grab_apod(*update_daily).await?,
            },
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

    ///
    /// Grab NASA Astronomy Picture of the Day.
    ///
    async fn grab_apod(&self, update_daily: bool) -> Result<(), Error> {
        let nasa_key = self.config.nasa_key().ok_or(Error::IllegalNASAKey)?;
        let url = format!("https://api.nasa.gov/planetary/apod?api_key={}", nasa_key);

        let response = reqwest::get(url)
            .await?
            .json::<grab::APoDAPIResponse>()
            .await?;

        let root_path = self.config.root();
        let files_path = root_path.join("Files");
        tokio::fs::create_dir_all(files_path.as_path()).await?;
        let news_path = root_path.join("Base").join("News");
        tokio::fs::create_dir_all(news_path.as_path()).await?;

        let media_ref: String;
        match response.media_type() {
            grab::APoDAPIMediaType::Image => {
                let image_url = Url::parse(response.url())?;
                let image_path = PathBuf::from(
                    image_url
                        .path_segments()
                        .ok_or_else(|| Error::IllegalURL(image_url.clone()))?
                        .into_iter()
                        .last()
                        .ok_or_else(|| Error::IllegalURL(image_url.clone()))?,
                );

                let mut new_image_path = files_path.join(format!("{}", Uuid::new_v4()));
                if let Some(image_extension) = image_path.extension() {
                    new_image_path = new_image_path.with_extension(image_extension);
                }

                // Download the image file.
                {
                    let response = reqwest::get(image_url.as_str()).await?;
                    let mut file = File::create(new_image_path.as_path()).await?;
                    let mut content = Cursor::new(response.bytes().await?);
                    tokio::io::copy(&mut content, &mut file).await?;
                    log::info!(
                        "The image was downloaded from {} into the file \"{}\"",
                        image_url,
                        new_image_path.display()
                    );
                }

                // Get the reference to the media file.
                media_ref = format!(
                    "![[{}]]",
                    new_image_path.file_name().and_then(OsStr::to_str).unwrap()
                );
            }

            grab::APoDAPIMediaType::Video => {
                let src = format!("src=\"{}\"", response.url());
                media_ref = vec![
                    "<iframe width=\"100%\" height=\"450\"",
                    src.as_str(),
                    "title=\"YouTube video player\"",
                    "frameborder=\"0\"",
                    "allow=\"accelerometer; autoplay; clipboard-write;",
                    "encrypted-media; gyroscope; picture-in-picture\"",
                    "allowfullscreen></iframe>",
                ]
                .join(" ");
            }

            grab::APoDAPIMediaType::Unknown => {
                return Err(Error::UnknownMediaType);
            }
        }

        let date = response.date().format("%Y-%m-%d").to_string();
        let mut content = vec![
            "---\ntype: news".to_string(),
            format!("name: \"{}\"", response.title()),
            "issue: APoD".to_string(),
            format!("date: {}", date),
            "tags:\n- news/apod\n- science/astronomy\n---\n".to_string(),
            format!("[[{}]]\n", date),
            format!("# {}\n", response.title()),
            format!("{}\n", media_ref),
            format!("**Explanation:** {}\n", response.explanation()),
        ];

        if let Some(copyright) = response.copyright() {
            content.push(format!("*Image copyright:* {}©\n", copyright));
        }

        let content = content.join("\n");
        let note_path = news_path.join(format!("APoD {}.md", date));
        {
            let mut file = File::create(note_path.as_path()).await?;
            file.write_all(content.as_bytes()).await?;
            log::info!(
                "The Astronomy Picture of the Day note \"{}\" has been created",
                note_path.display()
            );
        }

        if update_daily {
            let daily_path = root_path.join("Daily").join(format!("{}.md", date));

            // Read content of the daily note.
            let mut buffer = String::new();
            {
                let mut file = File::open(daily_path.as_path()).await?;
                file.read_to_string(&mut buffer).await?;
            }

            let line = format!(
                "\n\n`rir:Star` [[APoD {}|Astronomy Picture of the Day]]\n",
                date
            );
            buffer.push_str(line.as_str());

            // Write updated content of the daily note.
            {
                let mut file = File::create(daily_path.as_path()).await?;
                file.write_all(buffer.as_bytes()).await?;
                log::info!(
                    "The daily note \"{}\" has been updated",
                    daily_path.display()
                );
            }
        }

        Ok(())
    }
}
