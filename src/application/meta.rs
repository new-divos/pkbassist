use std::{str::FromStr, string::ToString};

use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::error::Error;

pub(crate) struct Metadata {
    metadata: Yaml,
    first: usize,
    last: usize,
}

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

        if let Some(first) = first {
            if let Some(last) = last {
                let yaml = YamlLoader::load_from_str(yaml.join("\n").as_str())?;

                Ok(Metadata {
                    metadata: yaml[0].to_owned(),
                    first,
                    last,
                })
            } else {
                Err(Error::IllegalNoteMetadata)
            }
        } else {
            Err(Error::NoteMetadataNotFound)
        }
    }
}

impl ToString for Metadata {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl Metadata {
    #[inline]
    pub(crate) fn first(&self) -> usize {
        self.first
    }

    #[inline]
    pub(crate) fn last(&self) -> usize {
        self.last
    }

    // Get the banner file name from the note metadata.
    #[inline]
    pub(crate) fn get_banner(&self) -> Option<&str> {
        self.metadata["banner"].as_str().map(|s| {
            let patterns: &[_] = &['!', '[', ']'];
            s.trim_matches(patterns)
        })
    }

    // Set the banner file name into the note metadata.
    pub(crate) fn set_banner<S: AsRef<str>>(&mut self, file_name: S) -> Result<(), Error> {
        let file_name = file_name.as_ref();
        if let Yaml::Hash(ref mut hash) = self.metadata {
            let key = Yaml::String("banner".to_string());
            let _ = hash.insert(key, Yaml::String(format!("![[{file_name}]]")));

            Ok(())
        } else {
            Err(Error::IllegalNoteMetadata)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_parse1_test() {
        let s = include_str!("../../tests/data/note1.md");

        let mut pos = usize::MAX;
        for (idx, line) in s.lines().enumerate() {
            if line.contains("type") {
                pos = idx;
                break;
            }
        }

        let meta = Metadata::from_str(s).unwrap();

        assert_eq!(meta.first(), pos);
        assert_eq!(meta.last(), pos);

        assert_eq!(meta.metadata["type"].as_str(), Some("test"));
    }

    #[test]
    fn metadata_parse2_test() {
        let s = include_str!("../../tests/data/note2.md");

        let mut first_pos = usize::MAX;
        let mut last_pos: usize = usize::MAX;

        for (idx, line) in s.lines().enumerate() {
            if line.contains("type") && first_pos == usize::MAX {
                first_pos = idx;
            }

            if line.contains("banner_icon") && last_pos == usize::MAX {
                last_pos = idx;
            }
        }

        let meta = Metadata::from_str(s).unwrap();
        assert_eq!(meta.first(), first_pos);
        assert_eq!(meta.last(), last_pos);

        assert_eq!(meta.metadata["type"].as_str(), Some("software"));
        assert_eq!(meta.metadata["name"].as_str(), Some("yaml-rust"));
        assert_eq!(meta.metadata["languages"][0].as_str(), Some("rust"));
        assert_eq!(
            meta.metadata["attributes"]["crate"].as_str(),
            Some("https://crates.io/crates/yaml-rust")
        );
        assert_eq!(meta.metadata["attributes"]["msrv"].as_str(), Some("1.31"));
        assert_eq!(
            meta.metadata["banner"].as_str(),
            Some("![[rust-language-banner.jpg]]")
        );
        assert_eq!(meta.metadata["banner_icon"].as_str(), Some("🧩"));
    }

    #[test]
    fn metadata_banner_test() {
        let s = include_str!("../../tests/data/note2.md");

        let mut meta = Metadata::from_str(s).unwrap();
        assert_eq!(meta.get_banner(), Some("rust-language-banner.jpg"));

        meta.set_banner("rust-lang-banner.png").unwrap();
        assert_eq!(meta.get_banner(), Some("rust-lang-banner.png"));
    }
}
