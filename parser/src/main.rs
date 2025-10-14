use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::Path,
    string,
};

fn main() {
    read_file_loop();
}

fn read_file_loop() {
    let wooo = TestFileIterator::new();

    for x in wooo.enumerate() {
        println!("{}", x.1);
        if x.0 > 4 {
            break;
        }
    }

    struct TestFileIterator {
        count: usize,
        content: String,
        reader: BufReader<File>,
        offset: usize,
        record_data: Vec<u8>,
    }
    impl TestFileIterator {
        fn new() -> TestFileIterator {
            let path = Path::new("parser/parsing_test.txt");
            let file = File::open(path).unwrap();

            TestFileIterator {
                count: 0,
                reader: BufReader::new(file),
                content: String::new(),
                offset: 0,
                record_data: vec![],
            }
        }
    }

    impl Iterator for TestFileIterator {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            let reader = &mut self.reader;
            reader
                .seek(SeekFrom::Start(self.offset.try_into().unwrap())) // convert usize to u64
                .unwrap();
            let mut data_buffer: Vec<u8> = vec![];
            loop {
                match reader.read_until(0x26, &mut data_buffer) {
                    Ok(bytes_read) => {
                        let mut buf = [0; 1];
                        match reader.read_exact(&mut buf) {
                            Ok(_) => {
                                if &buf == b"&" {
                                    data_buffer.extend_from_slice(&buf);
                                    self.record_data.extend_from_slice(&data_buffer);
                                    let content = &self.record_data;
                                    self.offset = self.offset + bytes_read + 1;
                                    print!("found two &s, returning ");
                                    return Some(String::from_utf8_lossy(&content).into_owned());
                                } else {
                                    self.offset = self.offset + bytes_read;
                                    self.record_data.extend_from_slice(&data_buffer);
                                }
                            }
                            Err(_) => {
                                println!("reached end of file");
                                // reached end of the file
                                return Some(String::from_utf8_lossy(&data_buffer).into_owned());
                            }
                        }
                    }

                    Err(_) => {
                        println!("ended!");
                        return None;
                    }
                }
            }
        }
    }
}

// 1. Check the first bytes of the file, if we're dealing with
// GZIP then read through until the next GZIP member.
// 2. Decompress that set of bytes.
// 3. Parse etc.

// https://users.rust-lang.org/t/using-read-until-for-a-sequence-of-bytes/116935/2
