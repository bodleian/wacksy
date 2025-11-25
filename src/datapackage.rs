//! Structured representation of a datapackage.json file.
//!
//! According to [the spec](https://specs.webrecorder.net/wacz/1.1.1/#datapackage-json):
//!
//! > The file **must** be present at the root of the WACZ which serves as the manifest for the web archive
//! > and is compliant with the [FRICTIONLESS-DATA-PACKAGE](https://specs.frictionlessdata.io/data-package/) specification.

use chrono::Local;
use sha2::{Digest as _, Sha256};
use std::{error::Error, fmt, fs, path::Path};

use crate::{
    WACZ_VERSION,
    indexer::{IndexRecord, to_cdxj_string, to_pages_json_string},
};

/// The main datapackage struct.
#[derive(Debug)]
pub struct DataPackage {
    /// In WACZ 1.1.1 this value is `data-package`.
    pub profile: String,
    /// See [`WACZ_VERSION`] constant.
    pub wacz_version: String,
    /// WACZ creation date, this is set to local datetime in [RFC 3399 format](https://rfc3339.date/).
    pub created: String,
    /// The name of the software used to create the WACZ file, in this case `wacksy 0.2.0`.
    pub software: String,
    /// List of file names, paths, sizes, and fixity for all files contained in the WACZ.
    pub resources: Vec<DataPackageResource>,
}

/// A resource listed in the datapackage.
#[derive(Debug)]
pub struct DataPackageResource {
    pub path: String,
    pub resource_type: ResourceType,
    pub hash: String,
    pub bytes: usize,
    /// The raw content of the resource in bytes,
    /// not passed through to serde when serialising to JSON.
    pub content: Vec<u8>,
}

#[derive(Debug)]
pub enum ResourceType {
    CDXJ,
    Pages,
    Warc,
}

/// A digest of the datapackage file itself.
pub struct DataPackageDigest {
    pub path: String,
    pub hash: String,
}
impl fmt::Display for DataPackageDigest {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return write!(
            message,
            "{{\"path\":\"{}\",\"hash\":\"{}\"}}",
            self.path, self.hash
        );
    }
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
    pub fn new(warc_file_path: &Path, index: &[IndexRecord]) -> Result<Self, DataPackageError> {
        let mut data_package = Self::default();

        let warc_file_bytes = match fs::read(warc_file_path) {
            Ok(bytes) => bytes,
            Err(error) => return Err(DataPackageError::FileReadError(error)),
        };

        let warc_file_name = match warc_file_path.file_name() {
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

        // Add Warc file to datapackage
        Self::add_resource(
            &mut data_package,
            DataPackageResource::new(ResourceType::Warc, &warc_file_name, &warc_file_bytes)?,
        );

        // Add CDXJ file to datapackage
        Self::add_resource(
            &mut data_package,
            DataPackageResource::new(
                ResourceType::CDXJ,
                "index.cdxj",
                &to_cdxj_string(index).into_bytes(),
            )?,
        );

        // Add Pages file to datapackage
        Self::add_resource(
            &mut data_package,
            DataPackageResource::new(
                ResourceType::Pages,
                "pages.jsonl",
                &to_pages_json_string(index).into_bytes(),
            )?,
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
    #[must_use]
    pub fn digest(&self) -> DataPackageDigest {
        return DataPackageDigest {
            path: "datapackage.json".to_owned(),
            hash: format!("sha256:{:x}", Sha256::digest(self.to_string())),
        };
    }
}
impl fmt::Display for DataPackage {
    //! The file should look something like this when serialised to JSON:
    //!
    //! ```json
    //! {
    //!   "profile": "data-package",
    //!   "wacz_version": "1.1.1",
    //!   "created": "2025-05-16T11:03:03.499792020+01:00",
    //!   "software": "wacksy 0.2.0",
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
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        let collected_resources = self
            .resources
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>()
            .join(",");
        // Iterate over each resource here and create datapackage
        return write!(
            message,
            "{{\"profile\":\"data-package\",\"wacz_version\":\"{}\",\"created\":\"{}\",\"software\":\"{}\",\"resources\":[{collected_resources}]}}",
            self.wacz_version, self.created, self.software
        );
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
    pub fn new(
        resource_type: ResourceType,
        file_name: &str,
        file_bytes: &[u8],
    ) -> Result<Self, DataPackageError> {
        // Add resource location to path. This
        // is a pretty convoluted way of doing things
        // but it works fine.
        let mut path = match resource_type {
            ResourceType::CDXJ => "indexes/",
            ResourceType::Pages => "pages/",
            ResourceType::Warc => "archive/",
        }
        .to_owned();
        path.push_str(file_name);

        return Ok(Self {
            path,
            hash: format!("sha256:{:x}", Sha256::digest(file_bytes)),
            bytes: file_bytes.len(),
            content: file_bytes.to_vec(),
            resource_type,
        });
    }
}
impl fmt::Display for DataPackageResource {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        let name = match self.resource_type {
            ResourceType::CDXJ => "crawl_index",
            ResourceType::Pages => "pages_file",
            ResourceType::Warc => "web_archive_file",
        };
        return write!(
            message,
            "{{\"name\":\"{}\",\"path\":\"{}\",\"hash\":\"{}\",\"bytes\":{}}}",
            name, self.path, self.hash, self.bytes
        );
    }
}

#[derive(Debug)]
pub enum DataPackageError {
    FileNameError(String),
    FilePathError(String),
    FileReadError(std::io::Error),
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
        }
    }
}
impl Error for DataPackageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
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
            let index = indexer(warc_file_path);
            return DataPackage::new(&warc_file_path, &index).unwrap();
        }
    }

    /// This test creates a datapackage and validates it against the
    /// Frictionless Datapackage Schema v1
    #[test]
    fn validate_datapackage_schema() -> Result<(), Box<dyn Error>> {
        let datapackage = common::create_datapackage().to_string();

        let instance: Value = serde_json::from_str(&datapackage)?;

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
        let datapackage_digest = datapackage.digest().to_string();
        let instance: Value = serde_json::from_str(&datapackage_digest)?;
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
