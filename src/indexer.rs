//! Reads the WARC file and composes a CDX(J) index.

use std::fmt;
use std::fs::File;
use std::io::Read as _;
use std::path::Path;
use warc::{BufferedBody, Record, RecordType, WarcReader};

mod indexing_errors;
pub use indexing_errors::IndexingError;
mod page_record;
pub use page_record::PageRecord;
mod record_timestamp;
pub use record_timestamp::RecordTimestamp;
mod warc_filename;
pub use warc_filename::WarcFilename;
mod record_digest;
pub use record_digest::RecordDigest;
mod record_content_type;
pub use record_content_type::RecordContentType;
mod record_url;
pub use record_url::RecordUrl;
mod record_status;
pub use record_status::RecordStatus;

pub struct Index {
    pub cdxj: CDXJIndex,
    pub pages: PageIndex,
    pub records_read: NumberOfRecordsRead,
}

impl Index {
    /// # Indexer
    ///
    /// This function sets off looping through the
    /// records to build the CDXJ and Pages.jsonl file.
    ///
    /// # Errors
    ///
    /// Returns a [file io error](IndexingError::WarcFileError) from
    /// `WarcReader::from_path`/`from_path_gzip` in case of any problem reading
    /// the WARC file. An [unrecoverable error](IndexingError::CriticalRecordError)
    /// when reading the WARC record will stop the indexer and propogate
    /// all the way up to the top.
    pub fn index_file(warc_file_path: &Path) -> Result<Self, IndexingError> {
        impl Gzipped for Path {}
        trait Gzipped {
            /// A gzip archive starts with the magic number '1f8b' followed by
            /// the compression flag which should be '8' for DEFLATE.
            /// If the first three bytes of the file match this sequence then
            /// it's probably a gzip file.
            fn is_gzip(&self) -> bool
            where
                Self: AsRef<Path>,
            {
                // open the file and create a 3-byte buffer
                let mut file = File::open(self).unwrap();
                let mut buffer = [0; 3];

                // read exactly 3 bytes into the buffer
                file.read_exact(&mut buffer).unwrap();

                // if the buffer matches this byte sequence then
                // the expression is true
                return buffer == [0x1f, 0x8B, 0x08];
            }
        }

        // this looping function accepts a generic type which
        // this allows us to pass in both gzipped and non-gzipped records
        fn loop_over_records<
            RecordIterator: Iterator<Item = Result<Record<BufferedBody>, warc::Error>>,
        >(
            file_records: RecordIterator,
            warc_file_path: &Path,
        ) -> Result<Index, IndexingError> {
            let mut record_count: usize = 0;
            let mut byte_counter: u64 = 0;
            let mut cdxj_index: Vec<CDXJIndexRecord> = Vec::with_capacity(1024);
            let mut page_index: Vec<PageRecord> = Vec::with_capacity(1024);

            for record in file_records.enumerate() {
                record_count = record.0 + 1; // enumerate is zero-indexed, so add 1 here to compensate
                match record.1 {
                    Ok(record) => {
                        match CDXJIndexRecord::new(&record, byte_counter, warc_file_path) {
                            Ok(processed_record) => {
                                // if the record was successfully indexed,
                                // add it to the index
                                cdxj_index.push(processed_record);
                                // now try creating a page record
                                match PageRecord::new(&record, record_count) {
                                    Ok(processed_record) => {
                                        page_index.push(processed_record);
                                    }
                                    Err(err) => eprintln!(
                                        "Could not create page record for warc record {record_count} with id {}: {err}",
                                        record.warc_id()
                                    ),
                                }
                            }
                            Err(err) => {
                                eprintln!(
                                    // Any error with the record means we have to
                                    // skip over it and move on to the next one.
                                    "Could not create cdxj record for warc record {record_count} with id {}: {err}",
                                    record.warc_id()
                                );
                            }
                        }

                        // Get the length of the record body in content_length,
                        // added to the length of the unwrapped record header
                        let record_length: u64 = record.content_length()
                            + u64::try_from(record.into_raw_parts().0.to_string().len()).unwrap();

                        // increment the byte counter after processing the record
                        byte_counter = byte_counter.wrapping_add(record_length);
                    }
                    Err(warc_error) => {
                        return Err(IndexingError::CriticalRecordError(
                            warc_error,
                            record_count,
                            byte_counter,
                        ));
                    }
                }
            }

            return Ok(Index {
                cdxj: CDXJIndex(cdxj_index),
                pages: PageIndex(page_index),
                records_read: NumberOfRecordsRead(record_count),
            });
        }

        if warc_file_path.is_gzip() {
            match WarcReader::from_path_gzip(warc_file_path) {
                Ok(file_gzip) => {
                    let file_records = file_gzip.iter_records();
                    let index = loop_over_records(file_records, warc_file_path)?;
                    return Ok(index);
                }
                Err(file_read_error) => return Err(IndexingError::WarcFileError(file_read_error)),
            }
        } else {
            match WarcReader::from_path(warc_file_path) {
                Ok(file_not_gzip) => {
                    let file_records = file_not_gzip.iter_records();
                    let index = loop_over_records(file_records, warc_file_path)?;
                    return Ok(index);
                }
                Err(file_read_error) => return Err(IndexingError::WarcFileError(file_read_error)),
            }
        }
    }
}

pub struct NumberOfRecordsRead(usize);
impl fmt::Display for NumberOfRecordsRead {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return write!(message, "{}", self.0);
    }
}

/// Contains a list of [CDX(J) records](CDXJIndexRecord).
pub struct CDXJIndex(Vec<CDXJIndexRecord>);
impl fmt::Display for CDXJIndex {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        let index_string: String = self.0.iter().map(ToString::to_string).collect();
        return write!(message, "{index_string}");
    }
}

pub struct PageIndex(Vec<PageRecord>);
impl fmt::Display for PageIndex {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        let index_string: String = self.0.iter().map(ToString::to_string).collect();
        let header: &'static str =
            "{\"format\":\"json-pages-1.0\",\"id\":\"pages\",\"title\":\"All Pages\"}";
        return write!(message, "{header}\n{index_string}");
    }
}

/// A record which would make up a line in a [CDX(J) index](CDXJIndex).
pub struct CDXJIndexRecord {
    /// The date and time when the web archive snapshot was created
    pub timestamp: RecordTimestamp,
    /// Sort-friendly formatted URL
    pub searchable_url: String,
    /// The URL that was archived
    pub url: RecordUrl,
    /// A cryptographic hash for the HTTP response payload       
    pub digest: RecordDigest,
    /// The media type for the response payload
    pub mime: RecordContentType,
    /// The WARC file where the WARC record is located
    pub filename: WarcFilename,
    /// The byte offset for the WARC record
    pub offset: u64,
    /// The length in bytes of the WARC record
    pub length: u64,
    // The HTTP status code for the HTTP response
    pub status: RecordStatus,
}

impl CDXJIndexRecord {
    /// # Create CDXJ index record
    ///
    /// Takes a `Record<BufferedBody>` and parses it to extract all
    /// the fields which make up a CDX(J) record.
    ///
    /// # Errors
    ///
    /// If the record is not a Warc `response`, `revisit`, `resource`, or `metadata`,
    /// an `UnindexableRecordType` error is returned. Otherwise, returns corresponding
    /// errors for each of the CDX(J) fields.
    pub fn new(
        record: &Record<BufferedBody>,
        byte_counter: u64,
        warc_file_path: &Path,
    ) -> Result<Self, IndexingError> {
        // first check whether the record is either
        // a response, revisit, resource, or metadata
        if [
            RecordType::Response,
            RecordType::Revisit,
            RecordType::Resource,
            RecordType::Metadata,
        ]
        .contains(record.warc_type())
        {
            let url = RecordUrl::new(record)?;
            let searchable_url = url.as_searchable_string()?;
            return Ok(Self {
                timestamp: RecordTimestamp::new(record)?,
                url,
                searchable_url,
                digest: RecordDigest::new(record)?,
                mime: RecordContentType::new(record)?,
                filename: WarcFilename::new(record, warc_file_path)?,
                offset: byte_counter,
                length: record.content_length(),
                status: RecordStatus::new(record)?,
            });
        } else {
            // if the record is not one of the types we want,
            // return an error
            return Err(IndexingError::UnindexableRecordType(
                record.warc_type().clone(),
            ));
        }
    }
}

/// Display the record to json as shown in [the example in the
/// spec](https://specs.webrecorder.net/cdxj/0.1.0/#example)
///
/// Could there be a better way to serialize this?
impl fmt::Display for CDXJIndexRecord {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return writeln!(
            message,
            "{} {} {{\"url\":\"{}\",\"digest\":\"{}\",\"mime\":\"{}\",\"offset\":{},\"length\":{},\"status\":{},\"filename\":\"{}\"}}",
            self.searchable_url,
            self.timestamp,
            self.url,
            self.digest,
            self.mime,
            self.offset,
            self.length,
            self.status,
            self.filename
        );
    }
}
