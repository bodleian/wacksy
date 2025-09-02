use crate::indexer::indexing_errors::IndexingError;
use std::fmt;
use warc::{BufferedBody, Record};

#[derive(PartialEq, Eq)]
pub struct RecordStatus(pub u16);

impl RecordStatus {
    /// # Parse record status
    ///
    /// Parse the record body with httparse and get the status code
    /// from the response.
    ///
    /// # Errors
    ///
    /// Returns a `RecordStatusError`, which can contain either
    /// a _parsing_ error from httparse, or an error arising
    /// from an empty response code.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut response = httparse::Response::new(&mut headers);

        match response.parse(record.body()) {
            Ok(_) => match response.code {
                Some(response_code) => return Ok(Self(response_code)),
                None => {
                    return Err(IndexingError::RecordStatusError(
                        "response code is empty".to_owned(),
                    ));
                }
            },
            Err(http_parsing_error) => {
                return Err(IndexingError::RecordStatusError(
                    http_parsing_error.to_string(),
                ));
            }
        };
    }
}
impl fmt::Display for RecordStatus {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return write!(message, "{}", self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::RecordStatus;
    use pretty_assertions::assert_eq;
    use warc::{BufferedBody, Record};

    #[test]
    fn valid_status() {
        let status = "200";
        let body = format!("HTTP/1.1 {status}\n");
        let record = Record::<BufferedBody>::new().add_body(body);
        let generated_status = RecordStatus::new(&record).unwrap().to_string();

        assert_eq!(generated_status, status);
    }

    // todo: check invalid status too
}
