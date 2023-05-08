use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    io::{self, Cursor},
    iter::repeat_with,
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use chrono::{Datelike, NaiveDate};
use futures::stream::{self, StreamExt};
use prettytable::{row, Table};
use regex::Regex;
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};
use url::Url;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    cli::{AddedObject, Arguments, Command, Info, Note, RemovedObject, RenamedObject},
    config::Config,
    error::Error,
};

pub(crate) mod apod;
pub(crate) mod entry;
pub(crate) mod meta;
pub(crate) mod twir;

///
/// The command line application.
///
#[derive(Debug)]
pub struct Application {
    config: Config,
}

impl Application {
    pub(crate) const QUALIFIER: &'static str = "ru";
    pub(crate) const AUTHOR: &'static str = "new-divos";
    pub(crate) const NAME: &'static str = "nta";
    pub(crate) const DESCRIPTION: &'static str = "A Very simple Notes Attendant";

    ///
    /// Create command line application with configuration.
    ///
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    ///
    /// Setup the application logger.
    ///
    pub fn setup_logger(args: &Arguments, config: &Config) -> Result<(), Error> {
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .chain(
                fern::Dispatch::new()
                    .level(log::LevelFilter::Warn)
                    .level_for("nta", log::LevelFilter::Info)
                    .chain(io::stdout()),
            )
            .chain({
                let mut file_config = fern::Dispatch::new();

                file_config = match args.verbosity {
                    0 => file_config
                        .level(log::LevelFilter::Warn)
                        .level_for("nta", log::LevelFilter::Info),
                    1 => file_config
                        .level(log::LevelFilter::Info)
                        .level_for("nta", log::LevelFilter::Debug),
                    2 => file_config
                        .level(log::LevelFilter::Info)
                        .level_for("nta", log::LevelFilter::Trace),
                    3 => file_config.level(log::LevelFilter::Debug),
                    _ => file_config.level(log::LevelFilter::Trace),
                };

                file_config.chain(fern::log_file(config.log_file())?)
            })
            .chain(
                fern::Dispatch::new()
                    .level(log::LevelFilter::Error)
                    .chain(io::stderr()),
            )
            .apply()?;

        Ok(())
    }

    ///
    /// Run the application.
    ///
    pub async fn run(&self, args: &Arguments) -> Result<(), Error> {
        match args.command {
            // Repair notes set.
            Command::Repair {
                wiki_refs,
                remove_unused_files,
                rename_files,
                twir_issues,
                apod_issues,
                remove_created,
                banners,
            } => {
                if wiki_refs {
                    self.repair_wiki_refs().await?;
                }

                if remove_unused_files {
                    self.remove_unused_files().await?;
                }

                if rename_files {
                    self.rename_attached_files().await?;
                }

                if twir_issues {
                    self.repair_twir_issues().await?;
                }

                if apod_issues {
                    self.repair_apod_issues().await?;
                }

                if remove_created {
                    self.remove_created().await?;
                }

                if banners {
                    self.repair_banners().await?;
                }
            }

            // Grab note into notes set.
            Command::Grab { ref note } => match note {
                // Grab NASA Astronomy Picture of the Day note.
                Note::APoD {
                    update_daily,
                    subtags,
                } => self.grab_apod(*update_daily, subtags).await?,

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

            // Add the additional information to the notes set.
            Command::Add { ref object } => match object {
                // Add the banner to the note.
                AddedObject::Banner {
                    file_name,
                    note_type,
                    note_tags,
                } => {
                    self.add_banner(
                        file_name.as_str(),
                        note_type.as_str(),
                        note_tags
                            .as_deref()
                            .map(|e| e.iter().map(|s| s.deref()).collect()),
                    )
                    .await?
                }

                // Add the calendar to the monthly note.
                AddedObject::Calendar { year, month } => self.add_calendar(*year, *month).await?,

                // Add a creation date to the notes.
                AddedObject::Created { note_type } => self.add_created(note_type.as_str()).await?,
            },

            // Rename the banner file name.
            Command::Rename { ref object } => match object {
                RenamedObject::Banner { old_name, new_name } => {
                    self.rename_banner(old_name.as_str(), new_name.as_str())
                        .await?
                }
            },

            // Remove the line from the vault notes.
            Command::Remove { ref object } => match object {
                RemovedObject::Line { line } => self.remove_line(line).await?,
                RemovedObject::Notes { raindrop } => {
                    if *raindrop {
                        self.remove_raindrop_notes().await?;
                    }
                }
            },

            // Configure the application.
            Command::Config {
                ref key,
                ref value,
                update,
            } => self.configure(key.as_str(), value.as_str(), update).await?,
        }

        Ok(())
    }

    ///
    /// Repair the wiki references.
    ///
    async fn repair_wiki_refs(&self) -> Result<(), Error> {
        let root = self.check_root()?;
        let re = Arc::new(
            Regex::new(
                r"\[\[\s*(?P<file>[A-Za-z\d\-\.]+(?:\s+[\w\d\-_\.\(\)]+)*)\s*\|\s+(?P<descr>.[^\[\]]+)\s*?\]\]",
            )
            .unwrap(),
        );
        let errors = stream::iter(WalkDir::new(root).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                }

                None
            })
            .zip(stream::iter(repeat_with(|| re.clone())))
            .then(|(e, re)| async move {
                log::trace!("Start processing of the file \"{}\"", e.path().display());
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

                log::trace!("Finish processing of the file \"{}\"", e.path().display());
                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Remove the unused files.
    ///
    async fn remove_unused_files(&self) -> Result<(), Error> {
        let root = self.check_root()?;
        let files_path = self.config.files_path().ok_or(Error::VaultRootIsAbsent)?;

        let files = Arc::new(
            stream::iter(WalkDir::new(files_path).into_iter())
                .filter_map(|e| async move {
                    if let Ok(e) = e {
                        if e.path().exists() && e.path().is_file() {
                            if let Some(file_name) = e.path().file_name().and_then(OsStr::to_str) {
                                return Some((file_name.to_string(), PathBuf::from(e.path())));
                            }
                        }
                    }

                    None
                })
                .collect::<HashMap<String, PathBuf>>()
                .await,
        );

        let mix = stream::iter(WalkDir::new(root).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists() && e.path().is_file() {
                        if let Some(extension) = e.path().extension().and_then(OsStr::to_str) {
                            if extension == "md" || extension == "canvas" {
                                return Some(e);
                            }
                        }
                    }
                }

                None
            })
            .zip(stream::iter(repeat_with(|| files.clone())))
            .then(|(e, files)| async move {
                log::trace!("Start processing of the file \"{}\"", e.path().display());
                let mut content = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut content).await?;
                }

                let mut links: Vec<String> = Vec::new();
                for (file_name, _) in files.iter() {
                    if content.contains(file_name.as_str()) {
                        links.push(file_name.clone());
                    }
                }
                links.shrink_to_fit();

                log::trace!("Finish processing of the file \"{}\"", e.path().display());
                Ok(links) as Result<Vec<String>, Error>
            })
            .collect::<Vec<_>>()
            .await;

        let mut links: HashSet<String> = HashSet::new();
        let mut errors: Vec<Error> = Vec::new();
        for r in mix.into_iter() {
            match r {
                Ok(l) => links.extend(l),
                Err(e) => errors.push(e),
            }
        }

        if errors.is_empty() {
            let unused: Vec<_> = files
                .iter()
                .filter_map(|(name, path)| {
                    if links.contains(name) {
                        None
                    } else {
                        Some(PathBuf::from(path))
                    }
                })
                .collect();

            if !unused.is_empty() {
                // Create the table.
                let mut table = Table::new();
                table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

                table.set_titles(row!["Unused Files"]);
                for path in &unused {
                    table.add_row(row![path.display()]);
                    tokio::fs::remove_file(path.as_path()).await?;
                }

                // Print the table to stdout
                table.printstd();
            }

            return Ok(());
        }

        Err(Error::MultipleExecutorsError(errors))
    }

    ///
    /// Rename the attached files.
    ///
    async fn rename_attached_files(&self) -> Result<(), Error> {
        let root = self.check_root()?;
        let files_path = self.config.files_path().ok_or(Error::VaultRootIsAbsent)?;

        let re = Arc::new(
            Regex::new(
                r"^[\dA-Fa-f]{8}\-[\dA-Fa-f]{4}\-[\dA-Fa-f]{4}\-[\dA-Fa-f]{4}-[\dA-Fa-f]{12}$",
            )
            .unwrap(),
        );

        let files = Arc::new(
            stream::iter(WalkDir::new(files_path).into_iter())
                .filter_map(|e| async move {
                    if let Ok(e) = e {
                        if e.path().exists() && e.path().is_file() {
                            return Some(e);
                        }
                    };

                    None
                })
                .zip(stream::iter(repeat_with(|| re.clone())))
                .filter_map(|(e, re)| async move {
                    if let Some(stem) = e.path().file_stem().and_then(OsStr::to_str) {
                        if !re.is_match(stem) {
                            if let Some(entry) = entry::FileEntry::new(e.path(), Uuid::new_v4()) {
                                return Some((stem.to_string(), entry));
                            }
                        }
                    }

                    None
                })
                .collect::<HashMap<_, _>>()
                .await,
        );

        let mut errors = stream::iter(WalkDir::new(root).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists() && e.path().is_file() {
                        if let Some(extension) = e.path().extension().and_then(OsStr::to_str) {
                            if extension == "md" || extension == "canvas" {
                                return Some(e);
                            }
                        }
                    }
                }

                None
            })
            .zip(stream::iter(repeat_with(|| files.clone())))
            .then(|(e, files)| async move {
                log::trace!("Start processing of the file \"{}\"", e.path().display());
                let mut content = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut content).await?;
                }

                let mut dirty = false;
                for (stem, fe) in files.iter() {
                    if content.contains(stem) {
                        content = content.replace(fe.old_name(), fe.new_name().as_ref());
                        dirty = true;
                    }
                }

                if dirty {
                    let mut file = File::create(e.path()).await?;
                    file.write_all(content.as_bytes()).await?;
                }

                log::trace!("Finish processing of the file \"{}\"", e.path().display());
                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        errors.extend(
            stream::iter(files.iter())
                .then(|(_, fe)| async move {
                    fs::rename(fe.old_path(), fe.new_path()).await?;
                    Ok(()) as Result<(), Error>
                })
                .filter_map(|r| async move { r.err() })
                .collect::<Vec<_>>()
                .await,
        );

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Repair the This Week in Rust issues.
    ///
    async fn repair_twir_issues(&self) -> Result<(), Error> {
        let _ = self.check_root()?;
        let twir_path = self.config.twir_path().ok_or(Error::VaultRootIsAbsent)?;

        let re = Arc::new(Regex::new(r"^TWiR\s+(?P<number>\d+)$").unwrap());
        let errors = stream::iter(WalkDir::new(twir_path).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                };

                None
            })
            .zip(stream::iter(repeat_with(|| re.clone())))
            .filter_map(|(e, re)| async move {
                if let Some(twir_path) = self.config.twir_path() {
                    if let Some(stem) = e.path().file_stem().and_then(OsStr::to_str) {
                        if let Some(cap) = re.captures_iter(stem).next() {
                            let number = &cap["number"];
                            let new_path = twir_path.join(format!("ISS.TWiR.{number}-.md"));
                            let number = number.parse::<i32>().ok()?;
                            return Some((e, new_path, number));
                        }
                    }
                }

                None
            })
            .then(|(e, new_path, number)| async move {
                log::trace!("Start processing of the file \"{}\"", e.path().display());

                let mut content = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut content).await?;
                }

                let next = number + 1;
                content = content
                    .replace("type: news", "type: issue")
                    .replace("news/twir", "issue/twir")
                    .replace(
                        format!("TWiR {next}").as_str(),
                        format!("ISS.TWiR.{next}-").as_str(),
                    );

                if number > 1 {
                    let prev = number - 1;
                    content = content.replace(
                        format!("TWiR {prev}").as_str(),
                        format!("ISS.TWiR.{prev}").as_str(),
                    );
                }
                {
                    let mut file = File::create(new_path).await?;
                    file.write_all(content.as_bytes()).await?;
                }
                fs::remove_file(e.path()).await?;

                log::trace!("Finish processing of the file \"{}\"", e.path().display());
                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Repair the Astronomy Picture of the Day issues.
    ///
    async fn repair_apod_issues(&self) -> Result<(), Error> {
        let _ = self.check_root()?;
        let apod_path = self.config.apod_path().ok_or(Error::VaultRootIsAbsent)?;

        let re = Arc::new(
            Regex::new(r"^APoD\s+(?P<year>\d{1,4})-(?P<month>\d{1,2})-(?P<day>\d{1,2})$").unwrap(),
        );
        let errors = stream::iter(WalkDir::new(apod_path).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                };

                None
            })
            .zip(stream::iter(repeat_with(|| re.clone())))
            .filter_map(|(e, re)| async move {
                if let (Some(apod_path), Some(daily_path)) =
                    (self.config.apod_path(), self.config.daily_path())
                {
                    if let Some(stem) = e.path().file_stem().and_then(OsStr::to_str) {
                        if let Some(cap) = re.captures_iter(stem).next() {
                            let year = &cap["year"];
                            let month = &cap["month"];
                            let day = &cap["day"];

                            let new_path =
                                apod_path.join(format!("ISS.APoD.{year}.{month}.{day}.md"));
                            let daily_path = daily_path.join(format!("{year}-{month}-{day}.md"));

                            return Some((e, new_path, daily_path));
                        }
                    }
                }

                None
            })
            .then(|(e, new_path, daily_path)| async move {
                log::trace!("Start processing of the file \"{}\"", e.path().display());
                let mut content = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut content).await?;
                }

                content = content
                    .replace("type: news", "type: issue")
                    .replace("news/apod", "issue/apod")
                    .replace("science/astronomy", "astronomy");
                {
                    let mut file = File::create(new_path.as_path()).await?;
                    file.write_all(content.as_bytes()).await?;
                }
                fs::remove_file(e.path()).await?;

                if daily_path.exists() && daily_path.is_file() {
                    if let Some(old_stem) = e.path().file_stem().and_then(OsStr::to_str) {
                        if let Some(new_stem) =
                            new_path.as_path().file_stem().and_then(OsStr::to_str)
                        {
                            let mut content = String::new();
                            {
                                let mut file = File::open(daily_path.as_path()).await?;
                                file.read_to_string(&mut content).await?;
                            }

                            content = content.replace(old_stem, new_stem);
                            fs::remove_file(daily_path.as_path()).await?;

                            {
                                let mut file = File::create(daily_path.as_path()).await?;
                                file.write_all(content.as_bytes()).await?;
                            }
                        }
                    }
                }

                log::trace!("Finish processing of the file \"{}\"", e.path().display());
                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Remove created tag from the notes.
    ///
    async fn remove_created(&self) -> Result<(), Error> {
        let root = self.check_root()?;
        let errors = stream::iter(WalkDir::new(root).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                }

                None
            })
            .then(|e| async move {
                // Read content of the daily note.
                let mut buffer = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut buffer).await?;
                }

                if let Ok(mut m) = meta::Metadata::from_str(buffer.as_str()) {
                    if m.has_created() {
                        let _ = m.remove_created();
                        let buffer = m.embed(buffer)?;

                        {
                            let mut file = File::create(e.path()).await?;
                            file.write_all(buffer.as_bytes()).await?;
                            log::trace!("The note \"{}\" has been updated", e.path().display());
                        }
                    }
                }
                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Repair the banners info.
    ///
    async fn repair_banners(&self) -> Result<(), Error> {
        let root = self.check_root()?;
        let errors = stream::iter(WalkDir::new(root).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                }

                None
            })
            .then(|e| async move {
                // Read content of the daily note.
                let mut buffer = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut buffer).await?;
                }

                if let Ok(mut m) = meta::Metadata::from_str(buffer.as_str()) {
                    if m.fix_banner() {
                        let buffer = m.embed(buffer)?;

                        let mut file = File::create(e.path()).await?;
                        file.write_all(buffer.as_bytes()).await?;
                        log::trace!("The note \"{}\" has been updated", e.path().display());
                    }
                }

                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Grab the NASA's Astronomy Picture of the Day issue.
    ///
    async fn grab_apod(
        &self,
        update_daily: bool,
        subtags: &Option<Vec<String>>,
    ) -> Result<(), Error> {
        let _ = self.check_root()?;

        let nasa_key = self.config.apod_key().ok_or(Error::IllegalNASAKey)?;
        let url = format!("https://api.nasa.gov/planetary/apod?api_key={nasa_key}");

        let response = reqwest::get(url).await?.json::<apod::Info>().await?;

        let files_path = self.config.files_path().ok_or(Error::VaultRootIsAbsent)?;
        tokio::fs::create_dir_all(&files_path).await?;
        let apod_path = self.config.apod_path().ok_or(Error::VaultRootIsAbsent)?;
        tokio::fs::create_dir_all(&apod_path).await?;

        let media_ref: String;
        match response.media_type() {
            apod::MediaType::Image => {
                let image_url = Url::parse(response.url())?;
                let image_path = PathBuf::from(
                    image_url
                        .path_segments()
                        .ok_or_else(|| Error::IllegalURL(image_url.clone()))?
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
                    log::trace!(
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
        let file_date = response.date().format("%Y.%m.%d").to_string();
        let daily_path = self
            .config
            .daily_path()
            .ok_or(Error::VaultRootIsAbsent)?
            .join(format!("{date}.md"));

        let supertag = "astronomy".to_string();
        let mut tags: HashSet<String> = HashSet::new();
        if let Some(subtags) = subtags {
            if subtags.is_empty() {
                tags.insert(supertag);
            } else {
                tags.extend(subtags.iter().map(|e| {
                    let s = e.to_lowercase();
                    if s.starts_with(supertag.as_str()) {
                        s
                    } else {
                        format!("{supertag}/{s}")
                    }
                }));
            }
        } else {
            tags.insert(supertag);
        }

        let mut content = vec![
            "---\ntype: issue".to_string(),
            format!("name: \"{}\"", response.title()),
            "issue: APoD".to_string(),
            format!("date: {date}"),
            "tags:\n- issue/apod".to_string(),
        ];

        content.extend(tags.into_iter().map(|e| format!("- {e}")));
        if let Some(banner) = self.config.apod_banner() {
            content.push(format!("banner: {banner}"));
        }
        if let Some(icon) = self.config.apod_icon() {
            content.push(format!("banner_icon: {icon}"));
        }

        content.extend(
            vec![
                "---\n".to_string(),
                if update_daily && daily_path.exists() && daily_path.is_file() {
                    format!("[[{date}]]\n")
                } else {
                    if update_daily {
                        log::warn!("Irrelevant daily path \"{}\"", daily_path.display());
                    }

                    format!("{date}\n")
                },
                format!("# {}\n", response.title()),
                format!("{media_ref}\n"),
                format!("**Explanation:** {}\n", response.explanation()),
            ]
            .into_iter(),
        );

        if let Some(copyright) = response.copyright() {
            content.push(format!("*Image copyright:* {copyright}©\n"));
        }

        let content = content.join("\n");
        let note_path = apod_path.join(format!("ISS.APoD.{file_date}.md"));
        {
            let mut file = File::create(note_path.as_path()).await?;
            file.write_all(content.as_bytes()).await?;
            log::trace!(
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

            let prefix = self.config.apod_prefix();
            let link = if let Some(prefix) = prefix {
                format!("{prefix} [[ISS.APoD.{file_date}|Astronomy Picture of the Day]]")
            } else {
                format!("[[ISS.APoD.{file_date}|Astronomy Picture of the Day]]")
            };
            let buffer = Self::modify_daily(buffer, link, self.config.apod_marker());

            // Write updated content of the daily note.
            {
                let mut file = File::create(daily_path.as_path()).await?;
                file.write_all(buffer.as_bytes()).await?;
                log::trace!(
                    "The daily note \"{}\" has been updated",
                    daily_path.display()
                );
            }
        }

        Ok(())
    }

    ///
    /// Grab the This Week in Rust single note.
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
            format!("---\ntype: issue\nissue: {number}"),
            format!("date: {date}\ntags:\n- rust\n- issue/twir\naliases:"),
            format!("- \"{}\"", note.title()),
            format!("- \"TWiR {date} This Week in Rust {number}\""),
            format!("url: {}", note.url()),
        ];

        if let Some(banner) = self.config.twir_banner() {
            content.push(format!("banner: {banner}"));
        }
        if let Some(icon) = self.config.twir_icon() {
            content.push(format!("banner_icon: {icon}"));
        }

        content.push("---\n".to_string());

        let next = number + 1;
        if number > 1 {
            let prev = number - 1;
            content.push(format!(
                "<< [[ISS.TWiR.{prev}|{prev}]] | [[ISS.TWiR.{next}|{next}]] >>\n"
            ));
        } else {
            content.push(format!("| [[ISS.TWiR.{next}|{next}]] >>\n"));
        }

        let daily_path = self
            .config
            .daily_path()
            .ok_or(Error::VaultRootIsAbsent)?
            .join(format!("{date}.md"));

        if update_daily && daily_path.exists() && daily_path.is_file() {
            content.push(format!("# [[{date}]]: This Week in Rust {number}\n"));
        } else {
            if update_daily {
                log::warn!("Irrelevant daily path \"{}\"", daily_path.display());
            }

            content.push(format!("# {date}: This Week in Rust {number}\n"));
        }
        content.push(md_content);

        let content = content.join("\n");
        let note_path = path.join(format!("ISS.TWiR.{number}.md"));
        {
            let mut file = File::create(note_path.as_path()).await?;
            file.write_all(content.as_bytes()).await?;
            log::info!(
                "The This Week in Rust note \"{}\" has been created",
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

            let link = if let Some(prefix) = self.config.twir_prefix() {
                format!("{prefix} [[ISS.TWiR.{number}|This Week in Rust {number}]]")
            } else {
                format!("[[ISS.TWiR.{number}|This Week in Rust {number}]]")
            };
            let buffer = Self::modify_daily(buffer, link, self.config.twir_marker());

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
        let _ = self.check_root()?;

        let notes = Arc::new(twir::Notes::select().await?);

        let twir_path = Arc::new(PathBuf::from(
            self.config.twir_path().ok_or(Error::VaultRootIsAbsent)?,
        ));
        tokio::fs::create_dir_all(twir_path.as_path()).await?;

        match issues {
            // The issues range.
            twir::Issues::Range(min_number, max_number) => {
                let errors = stream::iter(*min_number..=*max_number)
                    .zip(stream::iter(repeat_with(|| {
                        (notes.clone(), twir_path.clone())
                    })))
                    .then(|(number, (notes, twir_path))| async move {
                        self.grab_twir_note(number, notes, twir_path.as_path(), update_daily)
                            .await
                    })
                    .filter_map(|r| async move { r.err() })
                    .collect::<Vec<_>>()
                    .await;

                if !errors.is_empty() {
                    return Err(Error::MultipleExecutorsError(errors));
                }
            }

            // The single issue.
            twir::Issues::Single(number) => {
                self.grab_twir_note(*number, notes.clone(), &twir_path, update_daily)
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

    ///
    /// Add the banner to the notes.
    ///
    async fn add_banner(
        &self,
        file_name: &str,
        note_type: &str,
        note_tags: Option<Vec<&str>>,
    ) -> Result<(), Error> {
        let root = self.check_root()?;
        let note_tags = Arc::new(note_tags);
        let errors = stream::iter(WalkDir::new(root).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                }

                None
            })
            .zip(stream::iter(repeat_with(|| note_tags.clone())))
            .then(|(e, note_tags)| async move {
                // Read content of the daily note.
                let mut buffer = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut buffer).await?;
                }

                if let Ok(mut m) = meta::Metadata::from_str(buffer.as_str()) {
                    if let Some(nt) = m.get_type() {
                        if nt == note_type {
                            if let Some(ref note_tags) = *note_tags {
                                if let Some(meta_tags) = m.get_tags() {
                                    for tag in note_tags.iter() {
                                        if !meta_tags.contains(tag) {
                                            return Ok(()) as Result<(), Error>;
                                        }
                                    }
                                }
                            }

                            let _ = m.set_banner(file_name);
                        }
                    }
                }

                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Add the calendar to the monthly note.
    ///
    async fn add_calendar(&self, year: i32, month: u32) -> Result<(), Error> {
        let _ = self.check_root()?;

        if year <= 0 || !(1..=12).contains(&month) {
            return Err(Error::IllegalDate);
        }

        let monthly_path = self
            .config
            .daily_path()
            .ok_or(Error::VaultRootIsAbsent)?
            .join(format!("{year}-{month:02}.md"));
        if !monthly_path.is_file() {
            return Err(Error::IllegalPath(format!("{}", monthly_path.display())));
        }

        let mut calendar = vec![
            "| Пн | Вт | Ср | Чт | Пт | Сб | Вс |".to_string(),
            "|:--:|:--:|:--:|:--:|:--:|:--:|:--:|".to_string(),
        ];

        let mut current =
            NaiveDate::from_ymd_opt(year, month, 1).ok_or_else(|| Error::IllegalDate)?;
        let mut n = current.weekday().num_days_from_monday() as usize;

        let mut row = "|".to_string();
        row.push_str("    |".repeat(n).as_str());

        loop {
            n += 1;
            row.push_str(
                format!(" [[{}\\|{}]] |", current.format("%Y-%m-%d"), current.day()).as_str(),
            );
            if n > 6 {
                calendar.push(row);
                row = "|".to_string();
                n = 0;
            }

            let prev = current;
            current = current.succ_opt().ok_or_else(|| Error::IllegalDate)?;
            if current.month() != month {
                n = prev.weekday().num_days_from_monday() as usize;
                row.push_str("    |".repeat(6 - n).as_str());
                calendar.push(row);
                break;
            }
        }

        let mut buffer = String::new();
        {
            let mut file = File::open(monthly_path.as_path()).await?;
            file.read_to_string(&mut buffer).await?;
        }

        buffer.push_str(format!("\n\n{}\n", calendar.join("\n")).as_str());

        // Write updated content of the monthly note.
        {
            let mut file = File::create(monthly_path.as_path()).await?;
            file.write_all(buffer.as_bytes()).await?;
            log::trace!(
                "The monthly note \"{}\" has been updated",
                monthly_path.display()
            );
        }

        Ok(())
    }

    ///
    /// Add a creation date to the note.
    ///
    async fn add_created(&self, note_type: &str) -> Result<(), Error> {
        let root = self.check_root()?;
        let errors = stream::iter(WalkDir::new(root).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                }

                None
            })
            .then(|e| async move {
                // Read content of the daily note.
                let mut buffer = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut buffer).await?;
                }

                if let Ok(mut m) = meta::Metadata::from_str(buffer.as_str()) {
                    if let Some(nt) = m.get_type() {
                        if nt == note_type && !m.has_created() {
                            let fm = fs::metadata(e.path()).await?;
                            let st = fm.created()?;

                            m.set_created(&st)?;
                            let buffer = m.embed(buffer)?;

                            {
                                let mut file = File::create(e.path()).await?;
                                file.write_all(buffer.as_bytes()).await?;
                                log::trace!("The note \"{}\" has been updated", e.path().display());
                            }
                        }
                    }
                }
                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Rename the banner file name.
    ///
    async fn rename_banner(&self, old_name: &str, new_name: &str) -> Result<(), Error> {
        let root = self.check_root()?;
        let errors = stream::iter(WalkDir::new(root).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                }

                None
            })
            .then(|e| async move {
                // Read content of the daily note.
                let mut buffer = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut buffer).await?;
                }

                if let Ok(mut m) = meta::Metadata::from_str(buffer.as_str()) {
                    if let Some(banner_name) = m.get_banner() {
                        if banner_name == old_name {
                            m.set_banner(new_name)?;
                            let buffer = m.embed(buffer)?;

                            {
                                let mut file = File::create(e.path()).await?;
                                file.write_all(buffer.as_bytes()).await?;
                                log::trace!("The note \"{}\" has been updated", e.path().display());
                            }
                        }
                    }
                }
                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Remove the specified line from every file of the vault.
    ///
    async fn remove_line<S: AsRef<str>>(&self, line: S) -> Result<(), Error> {
        let line = line.as_ref().trim();
        let root = self.check_root()?;
        let errors = stream::iter(WalkDir::new(root).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                }

                None
            })
            .then(|e| async move {
                // Read content of the daily note.
                let mut buffer = String::new();
                {
                    let mut file = File::open(e.path()).await?;
                    file.read_to_string(&mut buffer).await?;
                }

                let mut lines: Vec<_> = buffer.lines().collect();
                if let Some(idx) = lines.iter().position(|s| (*s).trim().ends_with(line)) {
                    let _ = lines.remove(idx);

                    let buffer = lines.join("\n");
                    let mut file = File::create(e.path()).await?;
                    file.write_all(buffer.as_bytes()).await?;
                    log::trace!("The note \"{}\" has been updated", e.path().display());
                }

                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Remove the raindrop.io notes.
    ///
    async fn remove_raindrop_notes(&self) -> Result<(), Error> {
        let raindrop_path = self.config.raindrop().path()?;
        let prefix = self.config.raindrop().prefix()?;

        let errors = stream::iter(WalkDir::new(raindrop_path).into_iter())
            .filter_map(|e| async move {
                if let Ok(e) = e {
                    if e.path().exists()
                        && e.path().is_file()
                        && e.path().extension().and_then(OsStr::to_str) == Some("md")
                    {
                        return Some(e);
                    }
                }

                None
            })
            .then(|e| async move {
                if let Some(file_name) = e.file_name().to_str() {
                    if file_name.starts_with(prefix) {
                        fs::remove_file(e.path()).await?;
                    }
                }

                Ok(()) as Result<(), Error>
            })
            .filter_map(|r| async move { r.err() })
            .collect::<Vec<_>>()
            .await;

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::MultipleExecutorsError(errors))
        }
    }

    ///
    /// Configure the application.
    ///
    async fn configure(&self, key: &str, value: &str, update: bool) -> Result<(), Error> {
        let mut config = self.config.clone();
        match key.to_lowercase().as_str() {
            "vault.root" => {
                let path = Path::new(value);
                config.set_root(path, update)?;
            }

            "vault.files" => {
                let path = Path::new(value);
                config.set_files_path(path);
            }

            "vault.daily" => {
                let path = Path::new(value);
                config.set_daily_path(path);
            }

            "vault.base" => {
                let path = Path::new(value);
                config.set_base_path(path);
            }

            "apod.path" => {
                let path = Path::new(value);
                config.set_apod_path(path);
            }

            "apod.key" => {
                config.set_apod_key(value);
            }

            "apod.banner" => {
                config.set_apod_banner(value);
            }

            "apod.prefix" => {
                config.set_apod_prefix(value);
            }

            "apod.marker" => {
                config.set_apod_marker(value);
            }

            "apod.icon" => {
                config.set_apod_icon(value);
            }

            "twir.path" => {
                let path = Path::new(value);
                config.set_twir_path(path);
            }

            "twir.banner" => {
                config.set_twir_banner(value);
            }

            "twir.prefix" => {
                config.set_twir_prefix(value);
            }

            "twir.marker" => {
                config.set_twir_marker(value);
            }

            "twir.icon" => {
                config.set_twir_icon(value);
            }

            _ => return config.set(key, value),
        }

        config.save().await?;
        Ok(())
    }

    ///
    /// Check the root path of the vault.
    ///
    #[inline]
    fn check_root(&self) -> Result<&Path, Error> {
        if let Some(path) = self.config.root() {
            if path.exists() && path.is_dir() {
                Ok(path)
            } else {
                Err(Error::IllegalVaultRoot(PathBuf::from(path)))
            }
        } else {
            Err(Error::VaultRootIsAbsent)
        }
    }

    ///
    /// Modify daily page.
    ///
    fn modify_daily<C: AsRef<str>, L: AsRef<str>>(
        content: C,
        link: L,
        marker: Option<&str>,
    ) -> String {
        let content = content.as_ref();
        let link = link.as_ref();

        let mut lines: Vec<_> = content.lines().collect();
        if let Some(marker) = marker {
            if let Some(idx) = lines.iter().position(|s| *s == marker) {
                lines.insert(idx + 1, link)
            } else {
                lines.push(link);
                lines.push("\n");
            }
        } else {
            lines.push(link);
            lines.push("\n");
        }

        lines.join("\n")
    }
}
