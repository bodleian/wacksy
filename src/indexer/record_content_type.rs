use crate::indexer::indexing_errors::IndexingError;
use std::{fmt, str};
use warc::{BufferedBody, Record, RecordType};

pub struct RecordContentType(String);

impl RecordContentType {
    /// # Parse record content type
    ///
    /// Parses the HTTP content type from the HTTP headers in
    /// the record body; this is not the same as the
    /// [content type from the WARC header](https://iipc.github.io/warc-specifications/specifications/warc-format/warc-1.1/#content-type),
    /// which would ususally be `application/http`.
    ///
    /// If the WARC record type is `revisit`, in which case the spec
    /// says to directly return that as the content type.
    ///
    /// # Errors
    ///
    /// Returns a `RecordContentTypeError` in case of any problems with
    /// parsing; this either wraps `httparse::Error`, or a `Utf8Error` when
    /// parsing the content type to string. Alternatively returns `ValueNotFound`
    /// if no content type is found in the HTTP headers.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        if record.warc_type() == &RecordType::Revisit {
            return Ok(Self("revisit".to_owned()));
        } else {
            // create a list of 64 empty headers, if this is not
            // enough then you'll get a TooManyHeaders error
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut response = httparse::Response::new(&mut headers);
            match response.parse(record.body()) {
                Ok(status) => status,
                Err(http_parsing_error) => {
                    return Err(IndexingError::RecordContentTypeError(
                        http_parsing_error.to_string(),
                    ));
                }
            };

            // loop through the list of headers looking for the content-type
            let mut content_type: Option<Result<&str, str::Utf8Error>> = None;
            for header in &headers {
                if header.name == "content-type" {
                    content_type = Some(str::from_utf8(header.value));
                    break;
                }
            }
            match content_type {
                Some(some_content_type) => match some_content_type {
                    Ok(parsed_content_type) => return Ok(Self(parsed_content_type.to_owned())),
                    Err(parsing_error) => {
                        return Err(IndexingError::RecordContentTypeError(
                            parsing_error.to_string(),
                        ));
                    }
                },
                None => {
                    return Err(IndexingError::ValueNotFound(
                        "content type not present in HTTP headers".to_owned(),
                    ));
                }
            };
        }
    }
}
impl fmt::Display for RecordContentType {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return write!(message, "{}", self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::RecordContentType;
    use warc::{BufferedBody, Record};
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_content_type() {
        let headers = Record::<BufferedBody>::new();
        let record = headers.add_body("HTTP/1.1 200\ncontent-type: text/html\n");

        let generated_content_type = RecordContentType::new(&record).unwrap().to_string();
        let example_content_type = "text/html";

        assert_eq!(generated_content_type, example_content_type);
    }
}
