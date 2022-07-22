use std::{
    ffi::OsStr,
    io::Cursor,
    iter::repeat_with,
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::future::join_all;
use prettytable::{cell, row, Table};
use regex::Regex;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};
use url::Url;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    config::{Arguments, Command, Config, Info, Note},
    error::Error,
};

pub(crate) mod apod;
pub(crate) mod twir;

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
                // Grab NASA Astronomy Picture of the Day note.
                Note::APoD { update_daily } => self.grab_apod(*update_daily).await?,

                // Grab This Week in Rust note.
                Note::TWiR {
                    issues,
                    update_daily,
                } => self.grab_twir(issues, *update_daily).await?,
            },

            // Show additional information.
            Command::Show { ref info } => match info {
                // Show This Week in Rust issues.
                Info::TWiR { last } => self.show_twir(*last).await?,
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

        let response = reqwest::get(url).await?.json::<apod::Info>().await?;

        let root_path = self.config.root();
        let files_path = root_path.join("Files");
        tokio::fs::create_dir_all(files_path.as_path()).await?;
        let news_path = root_path.join("Base").join("News");
        tokio::fs::create_dir_all(news_path.as_path()).await?;

        let media_ref: String;
        match response.media_type() {
            apod::MediaType::Image => {
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

            apod::MediaType::Video => {
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

            apod::MediaType::Unknown => {
                return Err(Error::UnknownMediaType);
            }
        }

        let date = response.date().format("%Y-%m-%d").to_string();
        let daily_path = root_path.join("Daily").join(format!("{}.md", date));

        let mut content = vec![
            "---\ntype: news".to_string(),
            format!("name: \"{}\"", response.title()),
            "issue: APoD".to_string(),
            format!("date: {}", date),
            "tags:\n- news/apod\n- science/astronomy\n---\n".to_string(),
            if update_daily && daily_path.exists() && daily_path.is_file() {
                format!("[[{}]]\n", date)
            } else {
                if update_daily {
                    log::warn!("Irrelevant daily path \"{}\"", daily_path.display());
                }

                format!("{}\n", date)
            },
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

        if update_daily && daily_path.exists() && daily_path.is_file() {
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

    ///
    /// Grab This Week in Rust single note.
    ///
    async fn grab_twir_note(
        &self,
        number: u32,
        notes: Arc<twir::Notes>,
        path: &Path,
        update_daily: bool,
    ) -> Result<(), Error> {
        let note = notes.find(number)?;
        let html_content = reqwest::get(note.url()).await?.text().await?;
        let document = scraper::Html::parse_document(&html_content);

        let article_selector = scraper::Selector::parse("article.post-content").unwrap();
        let article = document
            .select(&article_selector)
            .next()
            .ok_or(Error::IllegalHTMLContent)?;
        let md_content = html2md::parse_html(article.inner_html().as_str());

        let date = note.datetime().format("%Y-%m-%d").to_string();

        let mut content = vec![
            format!("---\ntype: news\nissue: {}", number),
            format!("date: {}\ntags:\n- rust\n- news/twir\naliases:", date),
            format!("- \"{}\"", note.title()),
            format!("- \"TWiR {} This Week in Rust {}\"", date, number),
            format!("url: {}\n---\n", note.url()),
        ];

        let next = number + 1;
        if number > 1 {
            let prev = number - 1;
            content.push(format!(
                "<< [[TWiR {0}|{0}]] | [[TWiR {1}|{1}]] >>\n",
                prev, next
            ));
        } else {
            content.push(format!("| [[TWiR {0}|{0}]] >>\n", next));
        }

        let daily_path = self
            .config
            .root()
            .join("Daily")
            .join(format!("{}.md", date));

        if update_daily && daily_path.exists() && daily_path.is_file() {
            content.push(format!("# [[{}]]: This Week in Rust {}\n", date, number));
        } else {
            if update_daily {
                log::warn!("Irrelevant daily path \"{}\"", daily_path.display());
            }

            content.push(format!("# {}: This Week in Rust {}\n", date, number));
        }
        content.push(md_content);

        let content = content.join("\n");
        let note_path = path.join(format!("TWiR {}.md", number));
        {
            let mut file = File::create(note_path.as_path()).await?;
            file.write_all(content.as_bytes()).await?;
            log::info!(
                "The This Weel in Rust note \"{}\" has been created",
                note_path.display()
            );
        }

        if update_daily && daily_path.exists() && daily_path.is_file() {
            // Read content of the daily note.
            let mut buffer = String::new();
            {
                let mut file = File::open(daily_path.as_path()).await?;
                file.read_to_string(&mut buffer).await?;
            }

            let line = format!(
                "\n\n`rir:Newspaper` [[Twir {0}|This Week in Rust {0}]]\n",
                number
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

    ///
    /// Grab This Week in Rust issues.
    ///
    async fn grab_twir(&self, issues: &twir::Issues, update_daily: bool) -> Result<(), Error> {
        let notes = Arc::new(twir::Notes::select().await?);

        let root_path = self.config.root();
        let news_path = root_path.join("Base").join("News");
        tokio::fs::create_dir_all(news_path.as_path()).await?;

        match issues {
            // The issues range.
            twir::Issues::Range(min_number, max_number) => {
                let errors: Vec<_> = join_all(
                    (*min_number..=*max_number)
                        .zip((*min_number..=*max_number).map(|_| notes.clone()))
                        .map(|(number, cloned)| {
                            self.grab_twir_note(number, cloned, news_path.as_path(), update_daily)
                        }),
                )
                .await
                .into_iter()
                .filter_map(|r| r.err())
                .collect();

                if !errors.is_empty() {
                    return Err(Error::MultipleExecutorsError(errors));
                }
            }

            // The single issue.
            twir::Issues::Single(number) => {
                self.grab_twir_note(*number, notes.clone(), news_path.as_path(), update_daily)
                    .await?;
            }
        }

        Ok(())
    }

    ///
    /// Show This Week in Rust issues.
    ///
    async fn show_twir(&self, last: bool) -> Result<(), Error> {
        let mut notes = twir::Notes::select().await?;
        if last {
            notes = notes.first();
        }

        // Create the table.
        let mut table = Table::new();
        table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table.set_titles(row!["Date", "Title", "URL"]);
        for note in notes.iter() {
            table.add_row(row![
                note.datetime().format("%Y-%m-%d"),
                note.title(),
                note.url()
            ]);
        }

        // Print the table to stdout
        table.printstd();

        Ok(())
    }
}
