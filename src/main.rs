use std::{env, error::Error, path::Path};
use wacksy::WACZ;
fn main() -> Result<(), Box<dyn Error>> {
    if let Some(path_argument) = env::args_os().nth(1) {
        println!("The warc file path is {}", path_argument.display());
        let warc_file_path = Path::new(&path_argument);
        let wacz_object = WACZ::from_file(warc_file_path)?;
        let zipped_wacz: Vec<u8> = wacz_object.as_zip_archive()?;
        std::fs::write("tests/output.wacz", zipped_wacz)?;
    } else {
        eprintln!("argument not suppplied");
    }
    return Ok(());
}
