use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::io;

#[derive(Debug)]
pub enum IndexingError {
    /// could not read timestamp from record
    RecordTimestampError(chrono::ParseError),
    /// could not get WARC filename
    WarcFilenameError(String),
    /// could not parse record content type
    RecordContentTypeError(String),
    /// could nor parse the target url
    RecordUrlError(url::ParseError),
    /// could not parse HTTP status code
    RecordStatusError(String),
    /// some value was missing
    ValueNotFound(String),
    /// this type of record can not be indexed
    UnindexableRecordType(warc::RecordType),
    /// could not read the WARC file
    WarcFileError(io::Error),
    /// an unrecoverable error with a WARC record
    /// which means the rest of the WARC file cannot be read
    /// this error wraps [`warc::Error`] and returns
    /// the record number and the byte offset of the
    /// record in the WARC file
    CriticalRecordError(warc::Error, usize, u64),
}
impl Display for IndexingError {
    fn fmt(&self, message: &mut Formatter<'_>) -> Result {
        match self {
            Self::RecordTimestampError(parse_error_message) => {
                return write!(
                    message,
                    "Could not read timestamp from record: {parse_error_message}"
                );
            }
            Self::WarcFilenameError(error_message) => {
                return write!(message, "Could not get WARC filename: {error_message}");
            }
            Self::RecordContentTypeError(error_message) => {
                return write!(
                    message,
                    "Could not parse record content type: {error_message}"
                );
            }
            Self::RecordUrlError(parse_error_message) => {
                return write!(message, "Could not parse target url: {parse_error_message}");
            }
            Self::RecordStatusError(parse_int_error_message) => {
                return write!(
                    message,
                    "Could not parse HTTP status: {parse_int_error_message}"
                );
            }
            Self::ValueNotFound(error_message) => {
                return write!(message, "Value not found: {error_message}");
            }
            Self::UnindexableRecordType(warc_type) => {
                return write!(
                    message,
                    "Could not index this type of record: {}",
                    warc_type.to_string()
                );
            }
            Self::WarcFileError(io_error) => {
                return write!(message, "Could not read the WARC file: {io_error}");
            }
            Self::CriticalRecordError(warc_error, record_count, byte_counter) => {
                return write!(
                    message,
                    "A critical problem occurred with record {record_count}. \
                    Any error with the record here affects the offset counter at byte {byte_counter}, \
                    so the rest of the file cannot be indexed: {warc_error}"
                );
            }
        }
    }
}
impl Error for IndexingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RecordTimestampError(parse_error) => return Some(parse_error),
            Self::RecordUrlError(parse_error) => return Some(parse_error),
            Self::WarcFileError(io_error) => return Some(io_error),
            Self::CriticalRecordError(warc_error, ..) => return Some(warc_error),
            Self::ValueNotFound(_)
            | Self::RecordStatusError(_)
            | Self::UnindexableRecordType(_)
            | Self::RecordContentTypeError(_)
            | Self::WarcFilenameError(_) => return None,
        }
    }
}
