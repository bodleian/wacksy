use crate::indexer::indexing_errors::IndexingError;
use std::fmt;
use warc::{BufferedBody, Record, WarcHeader};

pub struct RecordDigest(String);

impl RecordDigest {
    /// # Get Warc digest
    ///
    /// Takes the digest from `WarcHeader::PayloadDigest`, and
    /// returns it as a string.
    ///
    /// # Errors
    ///
    /// Returns a `ValueNotFound` error if no payload digest is found
    /// in the WARC header.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        if let Some(record_digest) = record.header(WarcHeader::PayloadDigest) {
            return Ok(Self(record_digest.to_string()));
        } else {
            return Err(IndexingError::ValueNotFound(format!(
                "Record {} does not have a payload digest in the WARC header",
                record.warc_id()
            )));
        }
    }
}
impl fmt::Display for RecordDigest {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return write!(message, "{}", self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::RecordDigest;
    use pretty_assertions::assert_eq;
    use warc::{BufferedBody, Record, WarcHeader};

    #[test]
    fn valid_digest() {
        let digest = "sha256:ea8fac7c65fb589b0d53560f5251f74f9e9b243478dcb6b3ea79b5e36449c8d9";
        let mut headers = Record::<BufferedBody>::new();
        headers
            .set_header(WarcHeader::PayloadDigest, digest)
            .unwrap();
        let record = headers.add_body("");

        let generated_digest = RecordDigest::new(&record).unwrap().to_string();

        assert_eq!(generated_digest, digest);
    }
}
