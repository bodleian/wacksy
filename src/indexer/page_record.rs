use crate::indexer::{
    RecordContentType, RecordStatus, RecordTimestamp, RecordUrl, indexing_errors::IndexingError,
};
use serde::Serialize;
use std::fmt;
use warc::{BufferedBody, Record, RecordType};

/// A page which would make up a line in a pages.jsonl file.
#[derive(Serialize)]
pub struct PageRecord {
    /// A unique incrementing counter to identify the page record
    pub id: usize,
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
    pub fn new(record: &Record<BufferedBody>, record_count: usize) -> Result<Self, IndexingError> {
        let mime = RecordContentType::new(record)?;
        let status = RecordStatus::new(record)?;

        // First check whether the record is either a response, revisit,
        // resource, or metadata and check whether the record mime type
        // refers to a web page.
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
                id: record_count,
                timestamp: RecordTimestamp::new(record)?, // when this gets serialised to json it prints the RFC-3339 formatted string, but, why? investigate.
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
    use serde_json::Value;
    use std::{error::Error, fs::File};
    use warc::{BufferedBody, Record, RecordType, WarcHeader};

    #[test]
    fn valid_page_record() -> Result<(), Box<dyn Error>> {
        let timestamp = "2025-08-06T14:37:28+01:00";
        let target_url = "https://thehtml.review/04/ascii-bedroom-archive/";

        let mut headers = Record::<BufferedBody>::new();
        headers.set_warc_type(RecordType::Resource);
        headers.set_header(WarcHeader::Date, timestamp).unwrap();
        headers
            .set_header(WarcHeader::TargetURI, target_url)
            .unwrap();
        let record = headers.add_body("HTTP/1.1 200\ncontent-type: text/html\n");

        let generated_page_record = PageRecord::new(&record, 1)?;
        let instance: Value = serde_json::to_value(&generated_page_record)?;

        let schema: Value =
            serde_json::from_reader(File::open("tests/schemas/page-record.schema.json")?)?;

        // Build & reuse (faster)
        let validator = jsonschema::validator_for(&schema)?;

        // Iterate over errors
        for error in validator.iter_errors(&instance) {
            eprintln!("Error: {error}");
            eprintln!("Location: {}", error.instance_path);
        }

        // Boolean result
        assert!(jsonschema::draft202012::is_valid(&schema, &instance));

        Ok(())
    }
}
