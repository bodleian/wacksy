use std::{
    fs::File,
    io::{BufReader, Read as _, Seek as _, SeekFrom},
    path::Path,
};

use flate2::bufread::GzDecoder;

fn main() {
    read_file_loop();
}

fn read_file_loop() {
    struct WarcReader {
        reader: BufReader<File>,
        byte_offset: usize,
        file_size: usize,
    }

    impl WarcReader {
        fn new() -> Self {
            let path = Path::new("parser/example.warc.gz");
            let file = File::open(path).unwrap();
            let file_size = usize::try_from(file.metadata().unwrap().len()).unwrap();

            return Self {
                reader: BufReader::new(file),
                byte_offset: 0,
                file_size,
            };
        }
    }

    impl Iterator for WarcReader {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            if self.file_size > self.byte_offset {
                println!("reading from byte {}", self.byte_offset);

                // Seek to the byte offset and start reading
                // from there onwards.
                let reader = &mut self.reader;
                reader
                    .seek(SeekFrom::Start(self.byte_offset.try_into().unwrap())) // convert usize to u64
                    .unwrap();

                // Wrap the reader in a GzDecoder and instantiate
                // an empty string to copy data into.
                let mut gz = GzDecoder::new(reader);
                let mut decode_string = String::with_capacity(2048);

                // Read bytes from the decoder to the string,
                // and bytes_out is the number of bytes produced
                // after decompression.
                let bytes_out = gz.read_to_string(&mut decode_string).unwrap();
                println!("decoded       {bytes_out} bytes");

                // Find the position of the reader in the file after decompression.
                let file_position =
                    usize::try_from(gz.get_mut().stream_position().unwrap()).unwrap();

                // The number of bytes read will be the position of
                // the reader in the file, minus the offset it read from.
                let bytes_read = file_position - self.byte_offset;
                println!("read in       {bytes_read} bytes");

                // Now add the bytes_read back to the offset
                // for the next record in the file
                self.byte_offset += bytes_read;

                println!("offset is now {}", self.byte_offset);
                println!("file size is  {}", self.file_size);

                return Some(decode_string);
            } else {
                // If the byte offset is greater than the file size,
                // we're at the end of the file, so return none
                // and end the iterator.
                return None;
            }
        }
    }

    for gzip_member in WarcReader::new() {
        println!("{gzip_member}");
    }
}
