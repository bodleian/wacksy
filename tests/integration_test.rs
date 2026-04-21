use pretty_assertions::assert_eq;
use std::{error::Error, fs, path::Path};
use wacksy::{
    WACZ,
    indexer::{indexer, to_cdxj_string},
};

const WARC_PATH: &str = "tests/example.warc.gz";
const WARC_PATH_2: &str = "tests/example2.warc.gz";

#[cfg(target_family = "windows")]
fn normalize_newlines(s: String) -> String {
    s.replace("\r\n", "\n")
}

#[test]
fn create_cdxj_index() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new(WARC_PATH);
    let index = indexer(warc_file_path);
    let generated_cdxj_index = to_cdxj_string(&index);

    #[cfg(target_family = "windows")]
    let example_cdxj_index = normalize_newlines(fs::read_to_string(Path::new(
        "tests/wacz_example/indexes/index.cdxj",
    ))?);
    #[cfg(not(target_family = "windows"))]
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

#[test]
fn single_warc() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new(WARC_PATH);

    let wacz_result = WACZ::from_files(&[&warc_file_path]);
    assert!(wacz_result.is_ok());

    Ok(())
}

#[test]
fn single_warc_fromfile() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new(WARC_PATH);

    let wacz_result = WACZ::from_file(&warc_file_path);
    assert!(wacz_result.is_ok());

    Ok(())
}

#[test]
fn multiple_warc() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new(WARC_PATH);
    let warc_file_path2: &Path = Path::new(WARC_PATH_2);

    let wacz_result = WACZ::from_files(&[&warc_file_path, &warc_file_path2]);
    assert!(wacz_result.is_ok());

    Ok(())
}
