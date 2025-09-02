use crate::indexer::indexing_errors::IndexingError;
use serde::Serialize;
use std::fmt;
use surt_rs::generate_surt;
use url::Url;
use warc::{BufferedBody, Record, WarcHeader};

#[derive(Serialize)]
pub struct RecordUrl(Url);

impl RecordUrl {
    /// # Get the url of the record
    ///
    /// Get the url from the `WarcHeader::TargetURI` field.
    ///
    /// # Errors
    ///
    /// Returns `RecordUrlError` if there is any problem parsing
    /// the url, this is a wrapper for `url::ParseError`.
    /// Alternatively returns `ValueNotFound` if no `TargetURI` field
    /// is present in the WARC header.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        if let Some(warc_header_url) = record.header(WarcHeader::TargetURI) {
            match Url::parse(&warc_header_url) {
                Ok(record_url) => return Ok(Self(record_url)),
                Err(parse_error) => return Err(IndexingError::RecordUrlError(parse_error)),
            }
        } else {
            return Err(IndexingError::ValueNotFound(
                "TargetURI not present in the WARC header".to_owned(),
            ));
        }
    }
    /// # Compose searchable string
    ///
    /// Take a url and return a Sort-friendly URI Reordering Transform (SURT)
    /// formatted string. It is cast to lowercase when displayed. This function
    /// relies on the [surt-rs](https://github.com/mijho/surt-rs) crate.
    ///
    /// # Errors
    ///
    /// Returns a `RecordUrlError` as a wrapper for `url::ParseError`
    /// if there is any problem parsing the url.
    pub fn as_searchable_string(&self) -> Result<String, IndexingError> {
        match generate_surt(self.0.as_str()) {
            Ok(sorted_url) => return Ok(sorted_url),
            Err(sorting_parse_error) => {
                return Err(IndexingError::RecordUrlError(sorting_parse_error));
            }
        }
    }
}
impl fmt::Display for RecordUrl {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        let url_string: String = self.0.clone().into();
        return write!(message, "{}", url_string.to_lowercase());
    }
}

#[cfg(test)]
mod tests {
    use super::RecordUrl;
    use pretty_assertions::assert_eq;
    use warc::{BufferedBody, Record, WarcHeader};

    #[test]
    fn valid_url() {
        let target_url = "https://thehtml.review/04/ascii-bedroom-archive/";

        let mut headers = Record::<BufferedBody>::new();
        headers
            .set_header(WarcHeader::TargetURI, target_url)
            .unwrap();
        let record = headers.add_body("");

        let parsed_url = RecordUrl::new(&record).unwrap().to_string();

        assert_eq!(parsed_url, target_url);
    }

    #[test]
    fn valid_surt() {
        let target_url = "https://thehtml.review/04/ascii-bedroom-archive/";

        let mut headers = Record::<BufferedBody>::new();
        headers
            .set_header(WarcHeader::TargetURI, target_url)
            .unwrap();
        let record = headers.add_body("");

        let surt_parsed_url = RecordUrl::new(&record)
            .unwrap()
            .as_searchable_string()
            .unwrap();

        assert_eq!(surt_parsed_url, "review,thehtml)/04/ascii-bedroom-archive");
    }
}
