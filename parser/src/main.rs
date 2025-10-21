use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::Path,
};

fn main() {
    read_file_loop();
}

fn read_file_loop() {
    let wooo = TestFileIterator::new();

    for x in wooo.enumerate() {
        println!("{}", x.1);
    }

    struct TestFileIterator {
        reader: BufReader<File>,
        offset: usize,
    }
    impl TestFileIterator {
        fn new() -> TestFileIterator {
            let path = Path::new("parser/parsing_test.txt");
            let file = File::open(path).unwrap();

            TestFileIterator {
                reader: BufReader::new(file),
                offset: 0,
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
            // println!("offset is {}", self.offset);
            let mut data_buffer: Vec<u8> = vec![];
            loop {
                // read until first '&' character
                match reader.read_until(0x26, &mut data_buffer) {
                    Ok(bytes_read) => {
                        // println!("read {bytes_read} bytes");
                        if bytes_read == 0 {
                            return None;
                        }
                        let mut buf = [0; 1];
                        match reader.read_exact(&mut buf) {
                            Ok(_) => {
                                if &buf != &[0x26; 1] {
                                    self.offset = self.offset + bytes_read + 1;
                                    data_buffer.extend_from_slice(&buf);
                                } else {
                                    self.offset = self.offset + bytes_read + 1;
                                    // remove the extra byte from the data buffer
                                    // before returning it
                                    data_buffer.pop();
                                    return Some(
                                        String::from_utf8_lossy(&data_buffer).into_owned(),
                                    );
                                }
                            }
                            Err(_) => {
                                self.offset = self.offset + bytes_read;
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
