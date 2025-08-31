use pretty_assertions::assert_eq;
use std::{error::Error, fs, path::Path};
use wacksy::indexer;

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

// the datapackage cannot be easily tested because it contains
// a local timestamp, how do I mock this?
// #[test]
// fn create_datapackage() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
//     let warc_file_path: &Path = Path::new(WARC_PATH);
//     let index = indexer::index_file(warc_file_path)?;
//     let warc_file = fs::read(warc_file_path)?;

//     let cdxj_index_bytes = index.0.to_string().into_bytes();
//     let pages_index_bytes = index.1.to_string().into_bytes();

//     let data_package =
//         datapackage::DataPackage::new(&warc_file, &cdxj_index_bytes, &pages_index_bytes)?;

//     let generated_data_package = serde_json::to_string(&data_package)?;
//     let example_data_package =
//         fs::read_to_string(Path::new("tests/wacz_example/datapackage.json"))?;
//     assert_eq!(generated_data_package, example_data_package);
//     Ok(())
// }
