use chrono::naive::NaiveDate;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

const APOD_DATE_FORMAT: &str = "%Y-%m-%d";

///
/// NASA Astronomy Picture of the Day API service version.
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Version {
    #[serde(rename = "v1")]
    V1_0,
}

impl Default for Version {
    #[inline]
    fn default() -> Self {
        Self::V1_0
    }
}

///
/// NASA Astronomy Picture of the Day API media type.
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum MediaType {
    #[serde(rename = "image")]
    Image,

    #[serde(rename = "video")]
    Video,

    #[serde(other)]
    Unknown,
}

///
/// NASA Astronomy Picture of the Day API response.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Info {
    ///
    /// This field (when present) contains the copyright holder of the image.
    /// If the field is not present, the image is public domain.
    ///
    copyright: Option<String>,

    ///
    /// This field just returns the date passed in the URL, so is only useful
    /// if not defining a target APOD date.
    ///
    #[serde(
        serialize_with = "serialize_apod_date",
        deserialize_with = "deserialize_apod_date"
    )]
    date: NaiveDate,

    ///
    /// A text description of the photo. This usually contains 100-200 words.
    ///
    explanation: String,

    ///
    /// If requested, this field contains the full quality version of the image.
    /// Note that this can be missing, even if you request it!
    ///
    hdurl: Option<String>,

    ///
    /// This field determines the type of content. Usually this is image,
    /// but can be video. There may be other media_types, but I haven’t found any.
    ///
    media_type: MediaType,

    ///
    /// This has always been v1, but.. could one day change.
    ///
    service_version: Version,

    ///
    /// This APOD title is usually 3-6 words long, and is reliably concise,
    /// factual, and descriptive.
    ///
    title: String,

    ///
    /// This field contains the actual APOD URL. Usually a .jpg, but for non-image APODs
    /// may be a YouTube video or another arbitrary URL.
    ///
    url: String,
}

// The signature of a serialize_with function must follow the pattern:
//
//    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer
//
// although it may also be generic over the input types T.
fn serialize_apod_date<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.format(APOD_DATE_FORMAT));
    serializer.serialize_str(&s)
}

// The signature of a deserialize_with function must follow the pattern:
//
//    fn deserialize<'de, D>(D) -> Result<T, D::Error>
//    where
//        D: Deserializer<'de>
//
// although it may also be generic over the output types T.
pub fn deserialize_apod_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, APOD_DATE_FORMAT).map_err(serde::de::Error::custom)
}

impl Info {
    ///
    /// Get the copyright holder of the image.
    ///
    #[inline]
    pub(crate) fn copyright(&self) -> Option<&str> {
        self.copyright.as_deref()
    }

    ///
    /// Get the target date.
    ///
    #[inline]
    pub(crate) fn date(&self) -> NaiveDate {
        self.date
    }

    ///
    /// Get the text description of the photo.
    ///
    #[inline]
    pub(crate) fn explanation(&self) -> &str {
        self.explanation.as_str()
    }

    ///
    /// Get the media type of the content.
    ///
    #[inline]
    pub(crate) fn media_type(&self) -> MediaType {
        self.media_type
    }

    ///
    /// Get the title of the image.
    ///
    #[inline]
    pub(crate) fn title(&self) -> &str {
        self.title.as_str()
    }

    ///
    /// Get the URL of the image.
    ///
    #[inline]
    pub(crate) fn url(&self) -> &str {
        self.url.as_str()
    }
}
