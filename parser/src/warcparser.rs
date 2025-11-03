use std::{
    fs::File,
    io::{BufRead as _, BufReader, Seek as _, SeekFrom},
    path::Path,
    str::FromStr,
};

fn main() {
    read_file_loop();
}

fn read_file_loop() {
    for gzip_member in WarcReader::new() {
        println!("{:?}", gzip_member);
    }

    struct WarcReader {
        reader: BufReader<File>,
        file_offset: usize,
        file_size: usize,
    }
    impl WarcReader {
        fn new() -> Self {
            let path = Path::new("parser/example.warc");
            let file = File::open(path).unwrap();
            let file_size = usize::try_from(file.metadata().unwrap().len()).unwrap();

            return Self {
                reader: BufReader::new(file),
                file_offset: 0,
                file_size,
            };
        }
    }

    #[derive(Debug)]
    enum WarcRecordType {
        Response,
        Revisit,
        Resource,
        Metadata,
        Unparseable,
    }

    #[derive(Debug)]
    struct WarcHeaderParsed {
        content_length: usize,
        digest: String,
        timestamp: String,
        record_type: WarcRecordType,
        url: String,
        is_page: bool,
    }
    impl WarcHeaderParsed {
        fn new() -> Self {
            WarcHeaderParsed {
                content_length: 0,
                digest: String::new(),
                timestamp: String::new(),
                record_type: WarcRecordType::Unparseable,
                url: String::new(),
                is_page: false,
            }
        }
    }

    impl Iterator for WarcReader {
        type Item = WarcHeaderParsed;

        fn next(&mut self) -> Option<Self::Item> {
            if self.file_size > self.file_offset {
                // Seek to the byte offset and start reading
                // from there onwards.
                let reader = &mut self.reader;

                reader
                    .seek(SeekFrom::Start(self.file_offset.try_into().unwrap())) // convert usize to u64
                    .unwrap();
                println!("reading from {} bytes", self.file_offset);

                let mut header_buffer = String::with_capacity(2048);
                let mut found_headers = false;
                // This while block was adapted from the warc_reader.rs
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

                // First, check whether the first 8 bytes of the record
                // match "WARC/1.1".
                if &header_buffer[..8] == "WARC/1.1" {
                    // Iterate over the lines in the header, skipping
                    // the first one as that's the WARC declaration.
                    let header_list = header_buffer.trim().lines();

                    let mut parsed_header = WarcHeaderParsed::new();

                    for named_field in header_list.skip(1) {
                        let split_field = named_field.split_once(':').unwrap();
                        let key = split_field.0.to_ascii_lowercase();
                        let value = split_field.1.trim();

                        match key.as_str() {
                            "content-length" => {
                                parsed_header.content_length = value.parse::<usize>().unwrap();
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
                                parsed_header.record_type = match value {
                                    "response" => WarcRecordType::Response,
                                    "revisit" => WarcRecordType::Revisit,
                                    "resource" => WarcRecordType::Resource,
                                    "metadata" => WarcRecordType::Metadata,
                                    // Should probably return with a defined
                                    // error if the record type is unparseable
                                    _ => WarcRecordType::Unparseable,
                                };
                            }
                            "warc-resource-type" => parsed_header.is_page = true,
                            _ => {
                                continue;
                            }
                        }
                    }

                    let header_length: usize = header_buffer.len();
                    println!("header was {header_length} bytes long");

                    // Add the header length and content length to the
                    // file offset. Also add 4 bytes to account for the
                    // newlines separating each record.
                    self.file_offset += header_buffer.len() + parsed_header.content_length + 4;
                    println!("next record offset is {}", self.file_offset);

                    return Some(parsed_header);
                } else {
                    // If the first 8 bytes of the header do not match "WARC/1.1"
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
}
