//! Reads WARC files and wraps them up into a WACZ archive.
//!
//! ```
//! # use std::error::Error;
//! # use wacksy::WACZ;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let warc_file_path = std::path::Path::new("tests/example.warc.gz"); // set path to your ᴡᴀʀᴄ file
//! let wacz_object = WACZ::from_file(warc_file_path)?; // index the ᴡᴀʀᴄ and create a ᴡᴀᴄᴢ object
//! let zipped_wacz: Vec<u8> = wacz_object.as_zip_archive()?; // zip up the ᴡᴀᴄᴢ
//! std::fs::write("tests/output.wacz", zipped_wacz)?; // write out to file
//! # Ok(())
//! # }
//! ```
#![doc(
    html_logo_url = "https://www.ox.ac.uk/sites/default/themes/custom/oxweb/images/oxweb-logo.gif",
    html_favicon_url = "https://www.bodleian.ox.ac.uk/sites/default/files/styles/favicon-32x32/public/bodreader/site-favicon/bod-favicon.png"
)]

pub mod datapackage;
pub mod indexer;
use std::{error::Error, fmt, path::Path};

use rawzip::{CompressionMethod, ZipArchiveWriter, ZipDataWriter};

use crate::{
    datapackage::{DataPackage, DataPackageDigest, DataPackageError},
    indexer::{CDXJIndex, Index, IndexingError, PageIndex},
};

/// Set the WACZ version of the file being created,
/// deprecated in [WACZ 1.2.0](https://specs.webrecorder.net/wacz/1.2.0/#changes).
pub const WACZ_VERSION: &str = "1.1.1";

/// A WACZ object
pub struct WACZ {
    pub datapackage: DataPackage,
    pub datapackage_digest: DataPackageDigest,
    pub cdxj_index: CDXJIndex,
    pub pages_index: PageIndex,
}
impl WACZ {
    /// # Create WACZ from WARC file
    ///
    /// This is the main function of the library, it takes a path to a WARC file,
    /// reads through it to produce CDXJ and page.json indexes. Everything is
    /// wrapped into a [datapackage], and then wrapped _again_ into a [WACZ] struct.
    ///
    /// # Errors
    ///
    /// Returns a [`WaczError`], which can be caused by a problem in either the
    /// [indexer](IndexingError) or the [datapackage](DataPackageError). As the
    /// datapackage depends on the index being complete, any problem with the
    /// indexer will return early without continuing.
    pub fn from_file(warc_file_path: &Path) -> Result<Self, WaczError> {
        match Index::index_file(warc_file_path) {
            Ok(index) => {
                let datapackage = match DataPackage::new(warc_file_path, &index) {
                    Ok(datapackage) => datapackage,
                    Err(datapackage_error) => {
                        return Err(WaczError::DataPackageError(datapackage_error));
                    }
                };
                let datapackage_digest = match datapackage.digest() {
                    Ok(digest) => digest,
                    Err(digest_error) => return Err(WaczError::DataPackageError(digest_error)),
                };

                return Ok(Self {
                    datapackage,
                    datapackage_digest,
                    cdxj_index: index.cdxj,
                    pages_index: index.pages,
                });
            }
            Err(indexing_error) => return Err(WaczError::IndexingError(indexing_error)),
        }
    }
    /// # Zipper
    ///
    /// Takes a WACZ struct and zips up every element into a zip file.
    /// This function is mostly a wrapper around [rawzip](https://crates.io/crates/rawzip).
    ///
    /// # Errors
    ///
    /// Returns a `rawzip` error if anything goes wrong with adding files
    /// files to the archive.
    pub fn as_zip_archive(&self) -> Result<Vec<u8>, rawzip::Error> {
        fn add_file_to_archive(
            archive: &mut ZipArchiveWriter<&mut Vec<u8>>,
            compression_method: CompressionMethod,
            file_data: &[u8],
            file_path: &str,
        ) {
            // Start a new file in our zip archive.
            let mut file = archive
                .new_file(file_path)
                .compression_method(compression_method)
                .create()
                .unwrap();

            // Wrap the file in a ZipDataWriter, which will track information for the
            // Zip data descriptor (like uncompressed size and crc).
            let mut writer = ZipDataWriter::new(&mut file);

            // Copy the data to the writer.
            std::io::copy(&mut &*file_data, &mut writer).unwrap();

            // Finish the file, which will return the finalized data descriptor
            let (_, descriptor) = writer.finish().unwrap();

            let uncompressed_size = descriptor.uncompressed_size();

            println!("wrote {uncompressed_size} bytes to {file_path}");

            // Write out the data descriptor and return the number of bytes the data compressed to.
            file.finish(descriptor).unwrap();
        }

        // Create a new Zip archive in memory.
        let mut output = Vec::new();
        let mut archive = ZipArchiveWriter::new(&mut output);

        // Set compression method to Store (no compression).
        let compression_method = CompressionMethod::Store;

        // iterate over every resource in the datapackage
        for datapackage_resource in &self.datapackage.resources {
            add_file_to_archive(
                &mut archive,
                compression_method,
                &datapackage_resource.content,
                &datapackage_resource.path,
            );
        }

        // add datapackage file
        add_file_to_archive(
            &mut archive,
            compression_method,
            &serde_json::to_vec(&self.datapackage).unwrap(),
            "datapackage.json",
        );

        // add digest file
        add_file_to_archive(
            &mut archive,
            compression_method,
            &serde_json::to_vec(&self.datapackage_digest).unwrap(),
            "datapackage-digest.json",
        );

        // Finish the archive, which will write the central directory.
        archive.finish()?;

        return Ok(output);
    }
}

#[derive(Debug)]
pub enum WaczError {
    IndexingError(IndexingError),
    DataPackageError(DataPackageError),
}
impl fmt::Display for WaczError {
    fn fmt(&self, message: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexingError(error_message) => {
                return write!(message, "Indexing error: {error_message}");
            }
            Self::DataPackageError(error_message) => {
                return write!(message, "Error when creating datapackage: {error_message}");
            }
        }
    }
}
impl Error for WaczError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IndexingError(error) => return Some(error),
            Self::DataPackageError(error) => return Some(error),
        }
    }
}
