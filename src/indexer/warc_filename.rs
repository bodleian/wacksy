use crate::indexer::indexing_errors::IndexingError;
use std::fmt;
use std::path::Path;
use warc::{BufferedBody, Record, WarcHeader};

pub struct WarcFilename(String);

impl WarcFilename {
    /// # Create Warc filename
    ///
    /// Takes the filename from `WarcHeader::Filename`, and converts it
    /// to a string. If no filename is found in the record this function
    /// reads the path to the warc file.
    ///
    /// # Errors
    ///
    /// Returns a `WarcFilenameError` error if the filename cannot be
    /// inferred from the file path. Normally you should not hit this
    /// error.
    pub fn new(
        record: &Record<BufferedBody>,
        warc_file_path: &Path,
    ) -> Result<Self, IndexingError> {
        if let Some(record_filename) = record.header(WarcHeader::Filename) {
            println!("record filename is {record_filename} from file");
            return Ok(Self(record_filename.into_owned()));
        } else {
            // If no filename is found in the record
            // we get the filename from the file path
            if let Some(warc_file_path) = warc_file_path.file_name() {
                return Ok(Self(warc_file_path.to_string_lossy().to_string()));
            } else {
                // Hit this error case if the filename
                // cannot be inferred from the Path
                return Err(IndexingError::WarcFilenameError(format!(
                    "Cannot infer filename from {}",
                    warc_file_path.to_string_lossy()
                )));
            }
        }
    }
}
impl fmt::Display for WarcFilename {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return write!(message, "{}", self.0);
    }
}
#[cfg(test)]
mod tests {
    use std::path::Path;
    use warc::{BufferedBody, Record, WarcHeader};
    use super::WarcFilename;
    use pretty_assertions::assert_eq;

    #[test]
    fn valid_filename() {
        let filename = "example.warc";
        let path = Path::new(filename);
        let mut headers = Record::<BufferedBody>::new();
        headers.set_header(WarcHeader::Filename, filename).unwrap();
        let record = headers.add_body("");
        let parsed_filename = WarcFilename::new(&record, path).unwrap().to_string();

        assert_eq!(parsed_filename, filename);
    }

    // todo test case, what if the filename and warc are both present and they don't match?
}
