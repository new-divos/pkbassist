use chrono::{DateTime, FixedOffset};

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
