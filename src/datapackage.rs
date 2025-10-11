//! Structured definition of a datapackage.json file.
//! According to [the spec](https://specs.webrecorder.net/wacz/1.1.1/#datapackage-json):
//!
//! > The file **must** be present at the root of the WACZ which serves as the manifest for the web archive
//! > and is compliant with the [FRICTIONLESS-DATA-PACKAGE](https://specs.frictionlessdata.io/data-package/) specification.
//!
//! The file should look something like this when serialised to json:
//!
//! ```json
//! {
//!   "profile": "data-package",
//!   "wacz_version": "1.1.1",
//!   "created": "2025-05-16T11:03:03.499792020+01:00",
//!   "software": "wacksy 0.1.0",
//!   "resources": [
//!     {
//!       "name": "data.warc",
//!       "path": "archive/data.warc",
//!       "hash": "sha256:210d0810aaf4a4aba556f97bc7fc497d176a8c171d8edab3390e213a41bed145",
//!       "bytes": 4599
//!     },
//!     {
//!       "name": "index.cdxj",
//!       "path": "indexes/index.cdxj",
//!       "hash": "sha256:0494f16f39fbb3744556e1d64be1088109ac35c730f4a30ac3a3b10942340ca3",
//!       "bytes": 543
//!     }
//!   ]
//! }
//! ```

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest as _, Sha256};
use std::{error::Error, ffi::OsStr, fmt, fs, path::Path};

use crate::{WACZ_VERSION, indexer::Index};

/// The main datapackage struct.
#[derive(Serialize, Deserialize)]
pub struct DataPackage {
    /// In WACZ 1.1.1 this value is `data-package`.
    pub profile: String,
    /// See [`WACZ_VERSION`] constant.
    pub wacz_version: String,
    /// WACZ creation date, this is set to local datetime in [RFC 3399 format](https://rfc3339.date/).
    pub created: String,
    /// The name of the software used to create the WACZ file, in this case `wacksy 0.1.0`.
    pub software: String,
    /// List of file names, paths, sizes, and fixity for all files contained in the WACZ.
    pub resources: Vec<DataPackageResource>,
}

/// A resource listed in the datapackage.
#[derive(Serialize, Deserialize, Debug)]
pub struct DataPackageResource {
    #[serde(rename = "name")]
    pub file_name: String,
    pub path: String,
    pub hash: String,
    pub bytes: usize,
    /// The raw content of the resource in bytes,
    /// not passed through to serde when serialising to json.
    #[serde(skip)]
    pub content: Vec<u8>,
}

/// A digest of the datapackage file itself.
#[derive(Serialize, Deserialize)]
pub struct DataPackageDigest {
    pub path: String,
    pub hash: String,
}

impl Default for DataPackage {
    fn default() -> Self {
        return Self {
            profile: "data-package".to_owned(),
            wacz_version: WACZ_VERSION.to_owned(),
            created: Local::now().to_rfc3339(),
            software: format!("wacksy {}", env!("CARGO_PKG_VERSION")),
            resources: Vec::with_capacity(512),
        };
    }
}
impl DataPackage {
    /// # Create datapackage
    ///
    /// Composes the data package and adds resources to it.
    ///
    /// # Errors
    ///
    /// Will return a `DataPackageError` relating to any
    /// resource if there is anything wrong with the filename
    /// or path of a resource.
    pub fn new(warc_file_path: &Path, index: &Index) -> Result<Self, DataPackageError> {
        let mut data_package = Self::default();

        let warc_file_bytes = match fs::read(warc_file_path) {
            Ok(bytes) => bytes,
            Err(error) => return Err(DataPackageError::FileReadError(error)),
        };

        // add warc file to datapackage
        let path: &Path = if warc_file_path.extension() == Some(OsStr::new("gz")) {
            Path::new("archive/data.warc.gz")
        } else {
            Path::new("archive/data.warc")
        };
        Self::add_resource(
            &mut data_package,
            DataPackageResource::new(path, &warc_file_bytes)?,
        );

        // add cdxj file to datapackage
        let path: &Path = Path::new("indexes/index.cdxj");
        Self::add_resource(
            &mut data_package,
            DataPackageResource::new(path, &index.cdxj.to_string().into_bytes())?,
        );

        // add pages file to datapackage
        let path: &Path = Path::new("pages/pages.jsonl");
        Self::add_resource(
            &mut data_package,
            DataPackageResource::new(path, &index.pages.to_string().into_bytes())?,
        );

        return Ok(data_package);
    }

    /// Takes a `DataPackage` struct and pushes a resource to the
    /// 'resources' field.
    fn add_resource(data_package: &mut Self, resource: DataPackageResource) {
        return data_package.resources.push(resource);
    }

    /// # Digest datapackage
    ///
    /// Takes a `DataPackage` struct and returns a `DataPackageDigest`
    /// containing a Sha256 hash of the datapackage.
    ///
    /// # Errors
    ///
    /// Will return a `serde_json` error if there's any problem
    /// deserialising the data package to a vector.
    pub fn digest(&self) -> Result<DataPackageDigest, DataPackageError> {
        match serde_json::to_vec(&self) {
            Ok(datapackage_as_vec) => {
                return Ok(DataPackageDigest {
                    path: "datapackage.json".to_owned(),
                    hash: format!("sha256:{:x}", Sha256::digest(datapackage_as_vec)),
                });
            }
            Err(serde_error) => {
                return Err(DataPackageError::SerialisationError(serde_error));
            }
        }
    }
}

impl DataPackageResource {
    /// # Instantiate datapackage resource
    ///
    /// This is for serialising a single resource to
    /// a struct to pass through to the `DataPackage`.
    ///
    /// # Errors
    ///
    /// Will return a `DataPackageError` mainly in case the
    /// resource file path or file name are missing or cannot
    /// be converted to string.
    pub fn new(path: &Path, file_bytes: &[u8]) -> Result<Self, DataPackageError> {
        let file_name = match path.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => file_name.to_owned(),
                None => {
                    return Err(DataPackageError::FileNameError(format!(
                        "unable to convert {} to string",
                        file_name.display()
                    )));
                }
            },
            None => {
                return Err(DataPackageError::FileNameError(
                    "file name is empty".to_owned(),
                ));
            }
        };
        let path = match path.to_str() {
            Some(path) => path.to_owned(),
            None => {
                return Err(DataPackageError::FilePathError(format!(
                    "unable to convert {file_name:?} to string"
                )));
            }
        };

        return Ok(Self {
            file_name,
            path,
            hash: format!("sha256:{:x}", Sha256::digest(file_bytes)),
            bytes: file_bytes.len(),
            content: file_bytes.to_vec(),
        });
    }
}

#[derive(Debug)]
pub enum DataPackageError {
    FileNameError(String),
    FilePathError(String),
    FileReadError(std::io::Error),
    SerialisationError(serde_json::Error),
}
impl fmt::Display for DataPackageError {
    fn fmt(&self, message: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNameError(error_message) => {
                return write!(message, "Filename error: {error_message}");
            }
            Self::FilePathError(error_message) => {
                return write!(message, "File path error: {error_message}");
            }
            Self::FileReadError(error_message) => {
                return write!(message, "Could not read WARC file: {error_message}");
            }
            Self::SerialisationError(error_message) => {
                return write!(message, "Serialisation error: {error_message}");
            }
        }
    }
}
impl Error for DataPackageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SerialisationError(parse_error) => return Some(parse_error),
            Self::FileReadError(read_error) => return Some(read_error),
            Self::FilePathError(_) | Self::FileNameError(_) => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DataPackage;
    use serde_json::Value;
    use std::{error::Error, fs::File};

    mod common {
        use super::DataPackage;
        use crate::indexer;
        use std::path::Path;

        pub fn create_datapackage() -> DataPackage {
            let warc_file_path: &Path = Path::new("tests/example.warc.gz");
            let index = indexer::Index::index_file(warc_file_path).unwrap();
            return DataPackage::new(&warc_file_path, &index).unwrap();
        }
    }

    /// This test creates a datapackage and validates it against the
    /// Frictionless Datapackage Schema v1
    #[test]
    fn validate_datapackage_schema() -> Result<(), Box<dyn Error>> {
        let datapackage: DataPackage = common::create_datapackage();
        let instance: Value = serde_json::to_value(&datapackage)?;
        let schema: Value =
            serde_json::from_reader(File::open("tests/schemas/datapackage.schema.json")?)?;

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

    /// This test creates a datapackage digest and validates it against a schema I've made up.
    #[test]
    fn validate_datapackage_digest_schema() -> Result<(), Box<dyn Error>> {
        let datapackage: DataPackage = common::create_datapackage();
        // create the digest
        let datapackage_digest = datapackage.digest()?;
        let instance: Value = serde_json::to_value(&datapackage_digest)?;
        let schema: Value =
            serde_json::from_reader(File::open("tests/schemas/datapackage-digest.schema.json")?)?;

        // Build & reuse (faster)
        let validator = jsonschema::validator_for(&schema)?;

        // Iterate over errors
        for error in validator.iter_errors(&instance) {
            eprintln!("Error: {error}");
            eprintln!("Location: {}", error.instance_path);
        }

        // Boolean result
        assert!(jsonschema::draft202012::is_valid(&schema, &instance));

        Ok(())
    }
}
