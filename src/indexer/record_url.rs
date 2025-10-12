use crate::indexer::indexing_errors::IndexingError;
use serde::Serialize;
use std::fmt;
use url::{Position, Url};
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
    /// Alternatively, returns `ValueNotFound` if no `TargetURI` field
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
    /// follows [the example in the CDXJ spec](https://specs.webrecorder.net/cdxj/0.1.0/#searchable-url),
    /// which goes through the following steps:
    /// 
    /// 1. lowercasing the URL
    /// 2. removing the protocol portion (HTTP or HTTPS)
    /// 3. replacing the [host name](https://url.spec.whatwg.org/#hosts-(domains-and-ip-addresses)) portion
    /// of the URL with a reversed, comma separated equivalent: `www.example.org`` becomes `org,example,www`
    /// 4. adding a `)` separator
    /// 5. adding the remaining portion of the URL (path and query)
    /// 
    /// This is fast and simple, but might not be completely compatible with
    /// other SURT implementations.
    ///
    /// # Errors
    ///
    /// Returns a `RecordUrlError` as a wrapper for `url::ParseError`
    /// if there is any problem parsing the url.
    pub fn as_searchable_string(&self) -> Result<String, IndexingError> {
        if let Some(host) = self.0.host_str() {
            // split the host string into an array at each dot
            let mut host_split: Vec<&str> = host.split('.').collect();

            // reverse the order of the array
            host_split.reverse();

            // join the array back into a comma-separated string
            let host_reversed = host_split.join(",");

            // capture everything else on the end of the url
            let url_path = &self.0[Position::BeforePath..];

            // put it all together
            return Ok(format!("{host_reversed}){url_path}"));
        } else {
            // print the full url here
            let url = self.0.as_str();
            return Err(IndexingError::ValueNotFound(format!(
                "{url} does not have a host, unable to construct a searchable string"
            )));
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

        assert_eq!(surt_parsed_url, "review,thehtml)/04/ascii-bedroom-archive/");
    }
}
