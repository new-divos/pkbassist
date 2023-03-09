use std::{str::FromStr, time::SystemTime};

use chrono::prelude::{DateTime, Local};
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::error::Error;

pub(crate) struct Metadata(Yaml);

impl FromStr for Metadata {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut first: Option<usize> = None;
        let mut last: Option<usize> = None;
        let mut yaml: Vec<&str> = Vec::new();

        for (idx, line) in s.lines().enumerate() {
            if line.starts_with("---") {
                if first.is_some() {
                    if idx > 0 {
                        last = Some(idx - 1);
                    }
                    break;
                } else {
                    first = Some(idx + 1);
                }
            }

            if last.is_none() {
                if let Some(first) = first {
                    if idx >= first {
                        yaml.push(line);
                    }
                }
            }
        }

        if first.is_some() {
            if last.is_some() {
                let yaml = YamlLoader::load_from_str(yaml.join("\n").as_str())?;

                Ok(Metadata(yaml[0].to_owned()))
            } else {
                Err(Error::IllegalNoteMetadata)
            }
        } else {
            Err(Error::NoteMetadataNotFound)
        }
    }
}

impl Metadata {
    // Get the note type.
    #[inline]
    pub(crate) fn get_type(&self) -> Option<&str> {
        self.0["type"].as_str()
    }

    // Get the note tags.
    #[inline]
    pub(crate) fn get_tags(&self) -> Option<Vec<&str>> {
        self.0["tags"]
            .as_vec()
            .map(|v| v.iter().filter_map(|e| e.as_str()).collect())
    }

    // Get the banner file name from the note metadata.
    #[inline]
    pub(crate) fn get_banner(&self) -> Option<&str> {
        self.0["banner"].as_str()
    }

    // Set the banner file name of the note metadata.
    pub(crate) fn set_banner<S: AsRef<str>>(&mut self, file_name: S) -> Result<(), Error> {
        if let Metadata(Yaml::Hash(ref mut hash)) = self {
            let key = Yaml::String("banner".to_string());
            let file_name = file_name.as_ref();

            let _ = hash.insert(key, Yaml::String(file_name.to_string()));
            Ok(())
        } else {
            Err(Error::IllegalNoteMetadata)
        }
    }

    // Fix the banner settings.
    pub(crate) fn fix_banner(&mut self) -> bool {
        if let Metadata(Yaml::Hash(ref mut hash)) = self {
            let mut flag = false;

            let key = Yaml::String("banner".to_string());
            if let Some(Yaml::String(ref file_name)) = hash.get(&key) {
                if !file_name.starts_with("Banners/") {
                    let patterns: &[_] = &['!', '[', ']'];
                    let file_name = file_name.trim_matches(patterns);

                    let _ = hash.insert(key, Yaml::String(format!("Banners/{file_name}")));
                    flag = true;
                }
            }

            let key = Yaml::String("banner_icon".to_string());
            if hash.get(&key).is_some() {
                hash.remove(&key);
                flag = true;
            }

            flag
        } else {
            false
        }
    }

    // Is creation timestamp present.
    #[inline]
    pub(crate) fn has_created(&self) -> bool {
        self.0["created"].as_str().is_some()
    }

    // Set the creation timestamp into the note metadata.
    pub(crate) fn set_created(&mut self, st: &SystemTime) -> Result<(), Error> {
        if let Metadata(Yaml::Hash(ref mut hash)) = self {
            let key = Yaml::String("created".to_string());

            let dt: DateTime<Local> = (*st).into();
            let value = format!("{}", dt.format("%FT%T%:z"));

            let _ = hash.insert(key, Yaml::String(value));
            Ok(())
        } else {
            Err(Error::IllegalNoteMetadata)
        }
    }

    // Remove the creation timestamp from the note metadata.
    pub(crate) fn remove_created(&mut self) -> Result<(), Error> {
        if let Metadata(Yaml::Hash(ref mut hash)) = self {
            let key = Yaml::String("created".to_string());

            hash.remove(&key);
            Ok(())
        } else {
            Err(Error::IllegalNoteMetadata)
        }
    }

    // Embed the metadata object into the note.
    pub(crate) fn embed<S: AsRef<str>>(&self, note: S) -> Result<String, Error> {
        let note = note.as_ref();
        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&self.0)?;

        let mut first: Option<usize> = None;
        let mut last: Option<usize> = None;
        let content: Vec<&str> = note.lines().collect();

        for (idx, line) in content.iter().enumerate() {
            if line.starts_with("---") {
                if first.is_some() {
                    if idx > 0 {
                        last = Some(idx);
                    }
                    break;
                } else {
                    first = Some(idx + 1);
                }
            }
        }

        let mut res: Vec<&str> = Vec::new();
        if let Some(first) = first {
            res.extend(content.iter().take(first - 1));
            res.push(out_str.as_str());
            if let Some(last) = last {
                res.extend(content.iter().skip(last));
            } else {
                res.push("---")
            }
        } else {
            res.push("---");
            res.push(out_str.as_str());
            res.push("---\n");
            res.extend(content.into_iter());
        }

        Ok(res.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_parse1_test() {
        let s = include_str!("../../tests/data/note1.md");
        let meta = Metadata::from_str(s).unwrap();
        assert_eq!(meta.0["type"].as_str(), Some("test"));
    }

    #[test]
    fn metadata_parse2_test() {
        let s = include_str!("../../tests/data/note2.md");
        let meta = Metadata::from_str(s).unwrap();

        assert_eq!(meta.0["type"].as_str(), Some("software"));
        assert_eq!(meta.0["name"].as_str(), Some("yaml-rust"));
        assert_eq!(meta.0["languages"][0].as_str(), Some("rust"));
        assert_eq!(
            meta.0["attributes"]["crate"].as_str(),
            Some("https://crates.io/crates/yaml-rust")
        );
        assert_eq!(meta.0["attributes"]["msrv"].as_str(), Some("1.31"));
        assert_eq!(
            meta.0["banner"].as_str(),
            Some("![[rust-language-banner.jpg]]")
        );
        assert_eq!(meta.0["banner_icon"].as_str(), Some("🧩"));
    }

    #[test]
    fn metadata_banner_test() {
        let s = include_str!("../../tests/data/note2.md");

        let mut meta = Metadata::from_str(s).unwrap();
        assert_eq!(meta.get_banner(), Some("rust-language-banner.jpg"));

        meta.set_banner("rust-lang-banner.png").unwrap();
        assert_eq!(meta.get_banner(), Some("rust-lang-banner.png"));
    }

    #[test]
    fn metadata_embed_test() {
        let s1 = include_str!("../../tests/data/note2.md");

        let mut meta = Metadata::from_str(s1).unwrap();
        meta.set_banner("rust-lang-banner.png").unwrap();

        let s2 = include_str!("../../tests/data/note1.md");
        let s3 = meta.embed(s2).unwrap();

        let meta = Metadata::from_str(s3.as_str()).unwrap();
        assert_eq!(meta.get_banner(), Some("rust-lang-banner.png"));
    }
}
