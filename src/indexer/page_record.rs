use crate::indexer::{
    RecordContentType, RecordStatus, RecordTimestamp, RecordUrl, indexing_errors::IndexingError,
};
use serde::Serialize;
use std::fmt;
use warc::{BufferedBody, Record, RecordType};

/// A page which would make up a line in a pages.jsonl file.
#[derive(Serialize)]
pub struct PageRecord {
    /// The date and time when the web archive snapshot was created
    #[serde(rename = "ts")]
    pub timestamp: RecordTimestamp,
    /// The URL that was archived
    pub url: RecordUrl,
}
impl PageRecord {
    /// # Create page record
    ///
    /// Takes a `Record<BufferedBody>` and extracts the timestamp
    /// and url for the pages.jsonl file. This will only produce page
    /// records for resources with a 200 OK response and a media type
    /// of either:
    ///
    /// * `text/html`
    /// * `application/xhtml+xml`
    /// * `text/plain`
    ///
    /// # Errors
    ///
    /// Returns an `UnindexableRecordType` error if the record is not
    /// a Warc `response`, `revisit`, or `resource`. Otherwise, returns
    /// corresponding errors for url, timestamp mime, or status fields.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        let mime = RecordContentType::new(record)?;
        let status = RecordStatus::new(record)?;

        // first check whether the record is either a response, revisit,
        // resource, or metadata and check whether the record mime type
        // refers to a web page
        if [
            RecordType::Response,
            RecordType::Revisit,
            RecordType::Resource,
        ]
        .contains(record.warc_type())
            && ["text/html", "application/xhtml+xml", "text/plain"]
                .contains(&mime.to_string().as_str())
            && status == RecordStatus(200)
        {
            return Ok(Self {
                timestamp: RecordTimestamp::new(record)?,
                url: RecordUrl::new(record)?,
            });
        } else {
            // if the record is not one of the types we want,
            // return an error
            // change this to a generic indexing error?
            return Err(IndexingError::UnindexableRecordType(
                record.warc_type().clone(),
            ));
        }
    }
}
/// Display the record to json.
impl fmt::Display for PageRecord {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        let pages_json_string = serde_json::to_string(self).unwrap();
        return writeln!(message, "{pages_json_string}");
    }
}

#[cfg(test)]
mod tests {
    use super::PageRecord;
    use pretty_assertions::assert_eq;
    use warc::{BufferedBody, Record, RecordType, WarcHeader};

    #[test]
    fn valid_page_record() {
        let timestamp = "2025-08-06T14:37:28+01:00";
        let target_url = "https://thehtml.review/04/ascii-bedroom-archive/";

        let mut headers = Record::<BufferedBody>::new();
        headers.set_warc_type(RecordType::Resource);
        headers.set_header(WarcHeader::Date, timestamp).unwrap();
        headers
            .set_header(WarcHeader::TargetURI, target_url)
            .unwrap();
        let record = headers.add_body("HTTP/1.1 200\ncontent-type: text/html\n");

        let generated_page_record = PageRecord::new(&record).unwrap().to_string();
        let example_page_record =
            format!("{{\"ts\":\"2025-08-06T13:37:28Z\",\"url\":\"{target_url}\"}}\n");

        assert_eq!(generated_page_record, example_page_record);
    }

    // todo: test the different conditions, eg. a resource with a different content type
}
