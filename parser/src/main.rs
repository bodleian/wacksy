use std::{
    fs::File,
    io::{BufRead as _, BufReader, Seek as _, SeekFrom},
    path::Path,
    str::FromStr as _,
};

fn main() {
    read_file_loop();
}

fn read_file_loop() {
    let warc_file_path = std::path::Path::new("parser/example.warc");

    for warc_record in WarcReader::new(warc_file_path) {
        println!("{warc_record:?}");
    }
}

struct WarcReader {
    reader: BufReader<File>,
    file_offset: usize,
    file_size: usize,
}
impl WarcReader {
    fn new(warc_file_path: &Path) -> Self {
        let file = File::open(warc_file_path).unwrap();
        let file_size = usize::try_from(file.metadata().unwrap().len()).unwrap();

        return Self {
            reader: BufReader::new(file),
            file_offset: 0,
            file_size,
        };
    }
}

#[derive(Debug, PartialEq)]
enum WarcRecordType {
    Response,
    Revisit,
    Resource,
    Metadata,
}

#[derive(Debug)]
struct ParsedIndexRecord {
    content_length: Option<usize>,
    header_length: usize,
    digest: String,
    timestamp: String,
    record_type: Option<WarcRecordType>,
    url: String,
    is_page: bool,
    is_http: bool,
    http_status_code: Option<usize>,
    mime_type: Option<String>,
}
impl ParsedIndexRecord {
    const fn new() -> Self {
        return Self {
            content_length: None,
            header_length: 0,
            digest: String::new(),
            timestamp: String::new(),
            record_type: None,
            url: String::new(),
            is_page: false,
            is_http: false,
            http_status_code: None,
            mime_type: None,
        };
    }
}

impl Iterator for WarcReader {
    type Item = ParsedIndexRecord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.file_size > self.file_offset {
            // Seek to the byte offset and start reading
            // from there onwards.
            let reader = &mut self.reader;

            reader
                .seek(SeekFrom::Start(self.file_offset.try_into().unwrap())) // convert usize to u64
                .unwrap();
            println!("reading from {} bytes", self.file_offset);

            let warc_header_buffer = read_header_block(reader)?;
            let mut parsed_header = ParsedIndexRecord::new();

            parsed_header.header_length = warc_header_buffer.len();
            println!("header was {} bytes long", parsed_header.header_length);

            // First, check whether the first 8 bytes of the record
            // match "WARC/1.1".
            if warc_header_buffer.starts_with("WARC/1.1") {
                parsed_header = process_headers(parsed_header, &warc_header_buffer);

                // Now that we've parsed the header, add the header length
                // and content length to the file offset. Also add 4 bytes
                // to account for the newlines separating each record.
                self.file_offset +=
                    parsed_header.header_length + parsed_header.content_length.unwrap() + 4;
                println!("next record offset is {}", self.file_offset);

                // If both of these conditions are met,
                // the record contains an HTTP resource.
                if [
                    Some(WarcRecordType::Response),
                    Some(WarcRecordType::Revisit),
                ]
                .contains(&parsed_header.record_type)
                    && parsed_header.is_http
                {
                    let http_header_buffer = read_header_block(reader)?;
                    parsed_header = process_headers(parsed_header, &http_header_buffer);
                }

                return Some(parsed_header);
            } else {
                // If the header does not start with "WARC/1.1"
                // then return none. This should be an error.
                return None;
            }
        } else {
            // If the byte offset is greater than the file size,
            // we're at the end of the file, so return none
            // and close the iterator.
            return None;
        }
    }
}

fn read_header_block(reader: &mut BufReader<File>) -> Option<String> {
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

fn process_headers(mut parsed_header: ParsedIndexRecord, buffer: &str) -> ParsedIndexRecord {
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
        parsed_header.http_status_code = Some(
            // TODO! Return 'None' or 'Error' if this doesn't work.
            raw_status_code.parse::<usize>().unwrap(),
        );
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
                        parsed_header.content_length = Some(value.parse::<usize>().unwrap());
                    }
                    "warc-payload-digest" => {
                        parsed_header.digest = String::from_str(value).unwrap();
                    }
                    "warc-date" => {
                        parsed_header.timestamp = String::from_str(value).unwrap();
                    }

                    "warc-target-uri" => {
                        parsed_header.url = String::from_str(value).unwrap();
                    }
                    "warc-type" => {
                        println!("warc type is {value}");
                        parsed_header.record_type = match value {
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
                            parsed_header.is_http = true;
                        }
                    }
                    "warc-resource-type" => parsed_header.is_page = true,
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
                    parsed_header.mime_type = Some(value.to_owned());
                }
            }
        }
    }
    return parsed_header;
}
