use pretty_assertions::assert_eq;
use std::{error::Error, fs, path::Path};
use wacksy::{WACZ, indexer};

const WARC_PATH: &str = "tests/example.warc.gz";

#[test]
fn create_cdxj_index() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new(WARC_PATH);
    let index = indexer::Index::index_file(warc_file_path)?;
    let generated_cdxj_index = index.cdxj.to_string();
    let example_cdxj_index =
        fs::read_to_string(Path::new("tests/wacz_example/indexes/index.cdxj"))?;
    assert_eq!(generated_cdxj_index, example_cdxj_index);
    Ok(())
}

// test more error here?
#[test]
fn bad_file_path() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new("bad/file/path");
    assert!(WACZ::from_file(warc_file_path).is_err());
    Ok(())
}
