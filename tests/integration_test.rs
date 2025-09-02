use pretty_assertions::assert_eq;
use serde_json::Value;
use std::{
    error::Error,
    fs::{self, File},
    path::Path,
};
use wacksy::{datapackage, indexer};

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

#[test]
fn create_pages_index() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new(WARC_PATH);
    let index = indexer::Index::index_file(warc_file_path)?;
    let generated_pages_index = index.pages.to_string();
    let example_pages_index =
        fs::read_to_string(Path::new("tests/wacz_example/pages/pages.jsonl"))?;
    assert_eq!(generated_pages_index, example_pages_index);
    Ok(())
}

/// This test creates a datapackage and validates it against the
/// Frictionless Datapackage Schema v1
#[test]
fn valid_datapackage_json_schema() -> Result<(), Box<dyn Error>> {
    let warc_file_path: &Path = Path::new(WARC_PATH);
    let index = indexer::Index::index_file(warc_file_path)?;
    let data_package = datapackage::DataPackage::new(&warc_file_path, &index)?;
    // here we are casting the datapackage to string, then back out to a JSON Value.
    // is there a better way of doing this?
    let data_package_string = serde_json::to_string(&data_package)?;
    let instance: Value = serde_json::from_str(&data_package_string)?;
    let schema: Value = serde_json::from_reader(File::open("tests/data-package-schema.json")?)?;

    // Build & reuse (faster)
    let validator = jsonschema::validator_for(&schema)?;

    // Iterate over errors
    for error in validator.iter_errors(&instance) {
        eprintln!("Error: {error}");
        eprintln!("Location: {}", error.instance_path);
    }

    // Boolean result
    assert!(jsonschema::draft4::is_valid(&schema, &instance));

    Ok(())
}
