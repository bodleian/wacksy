use std::{env, error::Error, path::Path};
use wacksy::WACZ;
fn main() -> Result<(), Box<dyn Error>> {
    let raw_args: Vec<_> = env::args_os().skip(1).collect();

    // This argument parsing is a bit gnarly, could definitely
    // be refactored to be easier to read
    #[allow(clippy::implicit_return)]
    let output_path = raw_args
        .windows(2)
        .find(|w| w[0] == "--output")
        .map_or_else(|| "output.wacz".into(), |w| w[1].clone());

    let mut skip_next = false;

    let warc_args: Vec<_> = raw_args
        .iter()
        .filter(|a| {
            if skip_next {
                skip_next = false;
                return false;
            }
            if *a == "--output" {
                skip_next = true;
                return false;
            }
            return true;
        })
        .collect();

    if warc_args.is_empty() {
        eprintln!("Usage: wacksy <file.warc.gz> [--output out.wacz]");
        return Ok(());
    }

    let warc_file_paths: Vec<&Path> = warc_args.iter().map(Path::new).collect();
    let wacz_object = WACZ::from_files(&warc_file_paths)?;
    let zipped_wacz: Vec<u8> = wacz_object.as_zip_archive()?;
    std::fs::write(&output_path, zipped_wacz)?;
    return Ok(());
}
