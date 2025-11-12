use crate::indexer::indexing_errors::IndexingError;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use warc::{BufferedBody, Record, WarcHeader};

#[derive(Deserialize, Serialize)]
pub struct RecordTimestamp(DateTime<chrono::FixedOffset>);

impl RecordTimestamp {
    /// # Get timestamp
    ///
    /// Get the timestamp from the WARC header field `WarcHeader::Date`,
    /// parse it to a `DateTime<FixedOffset>`.
    ///
    /// # Errors
    ///
    /// Returns a `RecordTimestampError` if there is a problem with
    /// parsing, and this wraps `chrono::ParseError`. Otherwise, returns
    /// `ValueNotFound` if there is no date in the WARC header.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        match record.header(WarcHeader::Date) {
            Some(warc_header_date) => match DateTime::parse_from_rfc3339(&warc_header_date) {
                Ok(parsed_datetime) => return Ok(Self(parsed_datetime)),
                Err(parsing_error) => {
                    return Err(IndexingError::RecordTimestampError(parsing_error));
                }
            },
            None => {
                return Err(IndexingError::ValueNotFound(format!(
                    "Record {} does not have a date in the WARC header",
                    record.warc_id()
                )));
            }
        }
    }
}
impl fmt::Display for RecordTimestamp {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return write!(message, "{}", self.0.format("%Y%m%d%H%M%S"));
    }
}

#[cfg(test)]
mod tests {
    use super::RecordTimestamp;
    use pretty_assertions::assert_eq;
    use warc::{BufferedBody, Record, WarcHeader};

    #[test]
    fn valid_timestamp() {
        let timestamp = "2025-08-06T14:37:28+01:00";
        let mut headers = Record::<BufferedBody>::new();
        headers.set_header(WarcHeader::Date, timestamp).unwrap();
        let record = headers.add_body("");
        let generated_timestamp = RecordTimestamp::new(&record).unwrap().to_string();

        assert_eq!(generated_timestamp, "20250806133728");
    }
}
