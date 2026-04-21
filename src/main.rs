use std::{env, error::Error, path::Path};
use wacksy::WACZ;
fn main() -> Result<(), Box<dyn Error>> {
    let raw_args: Vec<_> = env::args_os().skip(1).collect();

    let output_path = raw_args
        .windows(2)
        .find(|w| w[0] == "--output")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "output.wacz".into());

    let mut skip_next = false;
    let warc_args: Vec<_> = raw_args.iter().filter(|a| {
        if skip_next { skip_next = false; return false; }
        if *a == "--output" { skip_next = true; return false; }
        true
    }).collect();

    if warc_args.is_empty() {
        eprintln!("Usage: wacksy <file.warc.gz> [--output out.wacz]");
        return Ok(());
    }

    let warc_file_paths: Vec<&Path> = warc_args.iter().map(|p| Path::new(p)).collect();
    let wacz_object = WACZ::from_files(&warc_file_paths)?;
    let zipped_wacz: Vec<u8> = wacz_object.as_zip_archive()?;
    std::fs::write(&output_path, zipped_wacz)?;
    Ok(())
}
