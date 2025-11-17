use chrono::DateTime;
use flate2::bufread::GzDecoder;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read as _, Seek as _, SeekFrom},
    path::Path,
    str::FromStr as _,
};

pub fn indexer(warc_file_path: &Path) -> Vec<IndexRecord> {
    let mut index = Vec::with_capacity(512);

    for index_record in WarcReader::new(warc_file_path) {
        if index_record.record_type.is_some()
            && !index_record.mime_type.is_empty()
            && index_record.http_status_code != 0
        {
            index.push(index_record);
        }
    }
    return index;
}

pub fn to_cdxj_string(index: &[IndexRecord]) -> String {
    let mut cdxj_index = String::with_capacity(512);

    for record in index {
        let surt = create_surt(&record.url).unwrap();
        // Parse the timestamp, and write out a formatted string
        let timestamp = DateTime::parse_from_rfc3339(&record.timestamp).unwrap();
        timestamp.format("%Y%m%d%H%M%S").to_string();
        let formatted_record = format!(
            "{} {} {{\"url\":\"{}\",\"digest\":\"{}\",\"mime\":\"{}\",\"offset\":{},\"length\":{},\"status\":{},\"filename\":\"{}\"}}\n",
            surt,
            timestamp,
            record.url,
            record.digest,
            record.mime_type,
            record.offset,
            record.content_length,
            record.http_status_code,
            record.file_name
        );
        cdxj_index.push_str(&formatted_record);
    }
    return cdxj_index.trim_end().to_owned();
}
pub fn to_pages_json_string(index: &[IndexRecord]) -> String {
    let mut pages_index =
        "{\"format\":\"json-pages-1.0\",\"id\":\"pages\",\"title\":\"All Pages\"}\n".to_owned();

    for record in index.iter().enumerate() {
        let record_struct = record.1;
        let record_number = record.0;
        if record_struct.is_page {
            let formatted_record = format!(
                "{{\"id\":\"{}\",\"url\":\"{}\",\"ts\":\"{}\"}}\n",
                record_number, record_struct.url, record_struct.timestamp,
            );
            pages_index.push_str(&formatted_record);
        }
    }
    return pages_index.trim_end().to_owned();
}

fn create_surt(url: &str) -> Option<String> {
    let url_without_protocol = match url {
        url if url.starts_with("https") => url.get(8..),
        url if url.starts_with("http") => url.get(7..),
        // URLs starting with urn are not surt-able.
        url if url.starts_with("urn") => return None,
        _ => None,
    }
    .unwrap();
    let url_split = url_without_protocol.split_once('/').unwrap();
    let mut host: Vec<&str> = url_split.0.split('.').collect();
    host.reverse();
    let host_reversed = host.join(",");
    return Some(format!("{host_reversed})/{}", url_split.1));
}

#[derive(Debug, PartialEq, Clone)]
enum WarcRecordType {
    Response,
    Revisit,
    Resource,
    Metadata,
}
#[derive(Debug, Clone)]
pub struct IndexRecord {
    offset: usize,
    content_length: usize,
    header_length: usize,
    digest: String,
    timestamp: String,
    record_type: Option<WarcRecordType>,
    url: String,
    is_page: bool,
    is_http: bool,
    http_status_code: usize,
    mime_type: String,
    file_name: String,
}
impl IndexRecord {
    fn new() -> Self {
        return Self {
            offset: 0,
            content_length: 0,
            header_length: 0,
            digest: String::with_capacity(128),
            timestamp: String::with_capacity(36),
            record_type: None,
            url: String::with_capacity(128),
            is_page: false,
            is_http: false,
            http_status_code: 0,
            mime_type: String::with_capacity(36),
            file_name: String::with_capacity(36),
        };
    }
}

struct WarcReader {
    reader: BufReader<File>,
    file_offset: usize,
    file_size: usize,
    file_name: String,
    is_gzip: bool,
}
impl WarcReader {
    fn new(warc_file_path: &Path) -> Self {
        let file = File::open(warc_file_path).unwrap();
        let file_size = usize::try_from(file.metadata().unwrap().len()).unwrap();

        // Check whether the warc is gzipped
        let is_gzip = warc_file_path
            .extension()
            .is_some_and(|extension| return extension == "gz");

        // Define the filename, to pass into each record.
        let file_name = warc_file_path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        return Self {
            reader: BufReader::new(file),
            file_offset: 0,
            file_size,
            file_name,
            is_gzip,
        };
    }
}
impl Iterator for WarcReader {
    type Item = IndexRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let mut parsed_record = IndexRecord::new();

        parsed_record.offset = self.file_offset;
        self.file_name.clone_into(&mut parsed_record.file_name);

        if self.file_size > self.file_offset {
            // Seek to the byte offset and start reading
            // from there onwards.
            let reader = &mut self.reader;

            // Start the reader from the file offset.
            reader
                .seek(SeekFrom::Start(self.file_offset.try_into().unwrap())) // convert usize to u64
                .unwrap();
            if self.is_gzip {
                // Wrap the reader in a GzDecoder and instantiate
                // an empty string to copy data into.
                let mut decoder = GzDecoder::new(reader);
                let mut byte_buffer = Vec::with_capacity(2048);

                // Read bytes from the decoder to a byte vector.
                decoder.read_to_end(&mut byte_buffer).unwrap();

                // Find the position of the reader in the file after decompression.
                let file_position =
                    usize::try_from(decoder.get_mut().stream_position().unwrap()).unwrap();

                // The number of bytes read will be the position of
                // the reader in the file, minus the offset it read from.
                let bytes_read = file_position - self.file_offset;

                // Now add the bytes_read back to the offset
                // for the next record in the file
                self.file_offset += bytes_read;

                // A byte slice has a Read trait, and can be passed into
                // read_header_block().
                let mut byte_reader = byte_buffer.as_slice();

                let warc_header_buffer = read_header_block(&mut byte_reader)?;

                // Set the header length
                parsed_record.header_length = warc_header_buffer.len();

                // First, check whether the first 8 bytes of the record
                // match "WARC/1.1".
                if warc_header_buffer.starts_with("WARC/1.1") {
                    parsed_record = process_headers(parsed_record, &warc_header_buffer);

                    // If both of these conditions are met,
                    // the record contains an HTTP resource.
                    if [
                        Some(WarcRecordType::Response),
                        Some(WarcRecordType::Revisit),
                    ]
                    .contains(&parsed_record.record_type)
                        && parsed_record.is_http
                    {
                        let http_header_buffer = read_header_block(&mut byte_reader)?;
                        parsed_record = process_headers(parsed_record, &http_header_buffer);
                    }
                    return Some(parsed_record);
                } else {
                    // If the header does not start with "WARC/1.1"
                    // then return none. This should be an error.
                    return None;
                }
            } else {
                // This could be broken into a separate parse_header function.

                // Read through the WARC header and return a string
                let warc_header_buffer = read_header_block(reader)?;

                // Set the header length
                parsed_record.header_length = warc_header_buffer.len();

                // First, check whether the first 8 bytes of the record
                // match "WARC/1.1".
                if warc_header_buffer.starts_with("WARC/1.1") {
                    parsed_record = process_headers(parsed_record, &warc_header_buffer);

                    // Now that we've parsed the header, add the header length
                    // and content length to the file offset. Also add 4 bytes
                    // to account for the newlines separating each record. The
                    // new file offset should now be at the start of the next record.
                    self.file_offset +=
                        parsed_record.header_length + parsed_record.content_length + 4;

                    // If both of these conditions are met,
                    // the record contains an HTTP resource.
                    if [
                        Some(WarcRecordType::Response),
                        Some(WarcRecordType::Revisit),
                    ]
                    .contains(&parsed_record.record_type)
                        && parsed_record.is_http
                    {
                        let http_header_buffer = read_header_block(reader)?;
                        parsed_record = process_headers(parsed_record, &http_header_buffer);
                    }

                    return Some(parsed_record);
                } else {
                    // If the header does not start with "WARC/1.1"
                    // then return none. This should be an error.
                    return None;
                }
            }
        } else {
            // If the byte offset is greater than the file size,
            // we're at the end of the file, so return none
            // and close the iterator.
            return None;
        }
    }
}

fn read_header_block<R: BufRead>(reader: &mut R) -> Option<String> {
    // This function was adapted from the warc_reader.rs
    // module in the warc library at https://github.com/jedireza/warc
    //
    // MIT License
    //
    // Copyright (c) 2016 Reza Akhavan <reza@akhavan.me>
    //
    // Permission is hereby granted, free of charge, to any person obtaining
    // a copy of this software and associated documentation files (the
    // 'Software'), to deal in the Software without restriction, including
    // without limitation the rights to use, copy, modify, merge, publish,
    // distribute, sublicense, and/or sell copies of the Software, and to
    // permit persons to whom the Software is furnished to do so, subject to
    // the following conditions:
    //
    // The above copyright notice and this permission notice shall be
    // included in all copies or substantial portions of the Software.
    //
    // THE SOFTWARE IS PROVIDED 'AS IS', WITHOUT WARRANTY OF ANY KIND,
    // EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
    // MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
    // IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
    // CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
    // TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
    // SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

    let mut header_buffer = String::with_capacity(2048);
    let mut found_headers = false;

    while !found_headers {
        // Read line-by-line from the offset in a loop
        // and stop when the reader two newlines.
        let bytes_read = reader.read_line(&mut header_buffer).unwrap();

        if bytes_read == 0 {
            return None;
        }

        // If the line is empty and consists only of newline
        // characters, then we've reached the end of the
        // header block.
        if bytes_read == 2 {
            let last_two_chars = header_buffer.len() - 2;
            if &header_buffer[last_two_chars..] == "\r\n" {
                found_headers = true;
            }
        }
    }
    return Some(header_buffer);
}

fn process_headers(mut parsed_record: IndexRecord, buffer: &str) -> IndexRecord {
    #[derive(PartialEq)]
    enum HeaderType {
        Warc,
        Http,
    }

    // The first four characters of the buffer should be
    // either "WARC" or "HTTP".
    let header_first_line = buffer.get(..4).unwrap();
    let header_type = match header_first_line {
        "WARC" => HeaderType::Warc,
        "HTTP" => HeaderType::Http,
        &_ => todo!("Return an error if the first line is not WARC or HTTP"),
    };

    if header_type == HeaderType::Http {
        // Get a slice between 9 and 12 bytes in,
        // this should be the HTTP status code.
        let raw_status_code = buffer.get(9..12).unwrap();
        // TODO! Return 'None' or 'Error' if this doesn't work.
        parsed_record.http_status_code = raw_status_code.parse::<usize>().unwrap();
    }

    // Iterate over the lines in the header block, skipping
    // the first one as that's the WARC or HTTP declaration.
    let header_iterator = buffer.trim().lines();

    // Go over each field in the header to find the content-type.
    for header_field in header_iterator.skip(1) {
        let split_field = header_field.split_once(':').unwrap();
        let key = split_field.0.to_ascii_lowercase();
        let value = split_field.1.trim();

        match header_type {
            HeaderType::Warc => {
                match key.as_str() {
                    "content-length" => {
                        parsed_record.content_length = value.parse::<usize>().unwrap();
                    }
                    "warc-payload-digest" => {
                        parsed_record.digest = String::from_str(value).unwrap();
                    }
                    "warc-date" => {
                        parsed_record.timestamp = String::from_str(value).unwrap();
                    }
                    "warc-target-uri" => {
                        parsed_record.url = String::from_str(value).unwrap();
                    }
                    "warc-type" => {
                        parsed_record.record_type = match value {
                            "response" => Some(WarcRecordType::Response),
                            "revisit" => Some(WarcRecordType::Revisit),
                            "resource" => Some(WarcRecordType::Resource),
                            "metadata" => Some(WarcRecordType::Metadata),
                            // Should probably return with a defined
                            // error if the record type is unparseable
                            _ => None,
                        };
                    }
                    "content-type" => {
                        if value.get(..16).is_some_and(|truncated_content_type| {
                            return truncated_content_type == "application/http";
                        }) {
                            // If the first 16 characters of the content type
                            // match this then it's an HTTP resource
                            parsed_record.is_http = true;
                        }
                    }
                    _ => {
                        // Do nothing?
                        continue;
                    }
                }
            }
            // If this is an HTTP header, the content-type refers to the
            // response body, and we want to get that.
            HeaderType::Http => {
                if &key == "content-type" {
                    value.clone_into(&mut parsed_record.mime_type);
                }
            }
        }
    }

    // We additionally want to know, if the content-type
    // is "text/html", and the status code was successful,
    // set the is_page value to true.
    if parsed_record.mime_type == "text/html"
        && (200..299).contains(&parsed_record.http_status_code)
    {
        parsed_record.is_page = true;
    }
    return parsed_record;
}
