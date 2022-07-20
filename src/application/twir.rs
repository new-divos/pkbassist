use std::str::FromStr;

use chrono::{DateTime, FixedOffset};
use regex::Regex;

use crate::error::Error;

const ARCHIVE_URL: &str = "https://this-week-in-rust.org/blog/archives/index.html";

///
/// This Week in Rust issue.
///
#[derive(Debug)]
pub(crate) struct TWiRIssue {
    datetime: DateTime<FixedOffset>,
    title: String,
    url: String,
}

impl TWiRIssue {
    ///
    /// Select all This Week in Rust issues.
    ///
    pub(crate) async fn select() -> Result<Vec<TWiRIssue>, Error> {
        let response = reqwest::get(ARCHIVE_URL).await?.text().await?;
        let document = scraper::Html::parse_document(&response);

        let row_selector = scraper::Selector::parse("div.row .post-title").unwrap();
        let time_selector = scraper::Selector::parse("time").unwrap();
        let href_selector = scraper::Selector::parse("a").unwrap();

        let mut issues: Vec<Self> = Vec::new();
        for row_html in document.select(&row_selector) {
            if let Some(time_html) = row_html.select(&time_selector).next() {
                if let Some(datetime) = time_html.value().attr("datetime") {
                    let datetime = DateTime::<FixedOffset>::parse_from_rfc3339(datetime)?;

                    if let Some(href_html) = row_html.select(&href_selector).next() {
                        if let Some(href) = href_html.value().attr("href") {
                            issues.push(Self {
                                datetime,
                                title: href_html.text().collect::<Vec<_>>().join(" "),
                                url: href.to_owned(),
                            });
                        }
                    }
                }
            }
        }

        issues.sort_by_key(|e| std::cmp::Reverse(e.datetime()));
        issues.shrink_to_fit();

        Ok(issues)
    }

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
/// The This week in Rust issue.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Issue {
    ///
    /// The range of issues.
    ///
    Range(u32, u32),

    ///
    /// The single issue.
    ///
    Value(u32),
}

impl FromStr for Issue {
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
            return Ok(Self::Value(u32::from_str(&caps["value"]).unwrap()));
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
            let value = Issue::from_str(format!("{}", i).as_str()).unwrap();
            assert_eq!(value, Issue::Value(i));
        }

        assert!(Issue::from_str("0").is_err());
        assert!(Issue::from_str("-1").is_err());

        for (i1, i2) in (1..=100).zip((1..=100).map(|x| x * x)) {
            let value = Issue::from_str(format!("{}..{}", i1, i2).as_str()).unwrap();
            assert_eq!(value, Issue::Range(i1, i2));
        }

        for (i1, i2) in (1..=100).zip((1..=100).map(|x| x + 10)) {
            let value = Issue::from_str(format!("{}..{}", i2, i1).as_str()).unwrap();
            assert_eq!(value, Issue::Range(i1, i2));
        }
    }
}
