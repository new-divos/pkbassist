use std::str::FromStr;

use chrono::{DateTime, FixedOffset};
use regex::Regex;

use crate::error::Error;

const ARCHIVE_URL: &str = "https://this-week-in-rust.org/blog/archives/index.html";

///
/// This Week in Rust issue.
///
#[derive(Debug, Clone)]
pub(crate) struct Note {
    datetime: DateTime<FixedOffset>,
    title: String,
    url: String,
}

impl Note {
    ///
    /// Issue date and time.
    ///
    #[inline]
    pub(crate) fn datetime(&self) -> DateTime<FixedOffset> {
        self.datetime
    }

    ///
    /// Issue title.
    ///
    #[inline]
    pub(crate) fn title(&self) -> &str {
        self.title.as_str()
    }

    ///
    /// Issue url.
    ///
    pub(crate) fn url(&self) -> &str {
        self.url.as_str()
    }
}

///
/// The collection of This Week in Rust issues.
///
pub(crate) struct Notes {
    notes: Vec<Note>,
}

impl Notes {
    ///
    /// Select all This Week in Rust issues.
    ///
    pub(crate) async fn select() -> Result<Notes, Error> {
        let html_content = reqwest::get(ARCHIVE_URL).await?.text().await?;
        let document = scraper::Html::parse_document(&html_content);

        let row_selector = scraper::Selector::parse("div.row .post-title").unwrap();
        let time_selector = scraper::Selector::parse("time").unwrap();
        let href_selector = scraper::Selector::parse("a").unwrap();

        let mut notes: Vec<Note> = Vec::new();
        for row_html in document.select(&row_selector) {
            if let Some(time_html) = row_html.select(&time_selector).next() {
                if let Some(datetime) = time_html.value().attr("datetime") {
                    let datetime = DateTime::<FixedOffset>::parse_from_rfc3339(datetime)?;

                    if let Some(href_html) = row_html.select(&href_selector).next() {
                        if let Some(href) = href_html.value().attr("href") {
                            notes.push(Note {
                                datetime,
                                title: href_html.text().collect::<Vec<_>>().join(" "),
                                url: href.to_owned(),
                            });
                        }
                    }
                }
            }
        }

        notes.sort_by_key(|e| std::cmp::Reverse(e.datetime()));
        notes.shrink_to_fit();

        Ok(Notes { notes })
    }

    ///
    /// Get the issues collection containing only the first elemet
    /// of the source collection.
    ///
    pub(crate) fn first(self) -> Self {
        let mut notes: Vec<_> = self.notes.into_iter().take(1).collect();
        notes.shrink_to_fit();

        Self { notes }
    }

    ///
    /// Get the iterator to iterate issues collection.
    ///
    #[inline]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Note> {
        self.notes.iter()
    }

    ///
    /// Find the issue by it's number.
    ///
    pub(crate) fn find(&self, number: u32) -> Result<&Note, Error> {
        let issue = number.to_string();
        for item in self.notes.iter() {
            if item.title.ends_with(issue.as_str()) {
                return Ok(item);
            }
        }

        Err(Error::IllegalIssue(issue))
    }
}

///
/// The This week in Rust issue.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issues {
    ///
    /// The range of issues.
    ///
    Range(u32, u32),

    ///
    /// The single issue.
    ///
    Single(u32),
}

impl FromStr for Issues {
    type Err = Error;

    ///
    /// Convert string to the Issue instance.
    ///
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^\s*(?P<min>[1-9]\d*)\s*\.\.\s*(?P<max>[1-9]\d*)\s*$").unwrap();
        if let Some(caps) = re.captures(s) {
            let mut min = u32::from_str(&caps["min"]).unwrap();
            let mut max = u32::from_str(&caps["max"]).unwrap();
            if min > max {
                std::mem::swap(&mut min, &mut max);
            }

            return Ok(Self::Range(min, max));
        }

        let re = Regex::new(r"^\s*(?P<value>[1-9]\d*)\s*$").unwrap();
        if let Some(caps) = re.captures(s) {
            return Ok(Self::Single(u32::from_str(&caps["value"]).unwrap()));
        }

        Err(Error::IllegalIssue(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_test() {
        for i in 1..=100 {
            let value = Issues::from_str(format!("{}", i).as_str()).unwrap();
            assert_eq!(value, Issues::Single(i));
        }

        assert!(Issues::from_str("0").is_err());
        assert!(Issues::from_str("-1").is_err());

        for (i1, i2) in (1..=100).zip((1..=100).map(|x| x * x)) {
            let value = Issues::from_str(format!("{}..{}", i1, i2).as_str()).unwrap();
            assert_eq!(value, Issues::Range(i1, i2));
        }

        for (i1, i2) in (1..=100).zip((1..=100).map(|x| x + 10)) {
            let value = Issues::from_str(format!("{}..{}", i2, i1).as_str()).unwrap();
            assert_eq!(value, Issues::Range(i1, i2));
        }
    }
}
