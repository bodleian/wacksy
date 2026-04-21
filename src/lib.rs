//! Reads WARC files and wraps them up into a WACZ archive.
//!
//! ```
//! # use std::error::Error;
//! # use std::fs;
//! # use std::io::{Cursor, Write as _};
//! # use wacksy::WACZ;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let warc_file_path = std::path::Path::new("tests/example.warc.gz"); // set path to your WARC file
//! let wacz_object = WACZ::from_files(&[warc_file_path])?; // index the WARC and create a WACZ object - multiple WARC files can be specified here
//! let zipped_wacz: Vec<u8> = wacz_object.as_zip_archive()?; // zip up the WACZ
//! let mut wacz_file = fs::File::create("tests/example.wacz")?;
//! # let mut wacz_file = Cursor::new(vec![0; 2048]);
//! wacz_file.write_all(&zipped_wacz)?; // write out to file
//! # fs::remove_file("tests/example.wacz")?;
//! # Ok(())
//! # }
//!
#![doc(
    html_logo_url = "https://www.ox.ac.uk/sites/default/themes/custom/oxweb/images/oxweb-logo.gif",
    html_favicon_url = "https://www.bodleian.ox.ac.uk/sites/default/files/styles/favicon-32x32/public/bodreader/site-favicon/bod-favicon.png"
)]

pub mod datapackage;
pub mod indexer;
use std::{error::Error, fmt, path::Path};

use rawzip::{CompressionMethod, ZipArchiveWriter};

use crate::{
    datapackage::{DataPackage, DataPackageDigest, DataPackageError},
    indexer::{IndexRecord, indexer},
};

/// Set the WACZ version of the file being created,
/// deprecated in [WACZ 1.2.0](https://specs.webrecorder.net/wacz/1.2.0/#changes).
pub const WACZ_VERSION: &str = "1.1.1";

/// A WACZ object
pub struct WACZ {
    pub datapackage: DataPackage,
    pub datapackage_digest: DataPackageDigest,
}
impl WACZ {
    /// # Create WACZ from a single WARC file
    ///
    /// Wrapper around from_files for backwards compatability.
    pub fn from_file(warc_file_path: &Path) -> Result<Self, WaczError> {
        WACZ::from_files(&[warc_file_path])
    }

    /// # Create WACZ from one or more WARC files
    ///
    /// This is the main function of the library, it takes a slice of paths to WARC files,
    /// reads through them to produce CDXJ and page.json indexes. Everything is
    /// wrapped into a [datapackage], and then wrapped _again_ into a [WACZ] struct.
    ///
    /// # Errors
    ///
    /// Returns a [`WaczError`], which can be caused by a problem in either the
    /// indexer (structured errors to-do) or the [datapackage](DataPackageError). As the
    /// datapackage depends on the index being complete, any problem with the
    /// indexer will return early without continuing.
    pub fn from_files(warc_file_paths: &[&Path]) -> Result<Self, WaczError> {
        // Check that at least one WARC is provided
        if warc_file_paths.is_empty() {
            return Err(WaczError::WarcFileError("No file".to_string()));
        }

        // Check that all files exist - if not, then return an error
        let missing_paths: Vec<String> = warc_file_paths.iter().filter(|p| !p.exists()).map(|p| p.to_string_lossy().to_string()).collect();
        if missing_paths.len() > 0 {
            return Err(WaczError::WarcFileError(missing_paths.join(", ")))
        }

        // Generate WACZ
        let index: Vec<IndexRecord> = warc_file_paths.into_iter().flat_map(|p| indexer(p)).collect();
        
        let datapackage: DataPackage = match DataPackage::new(warc_file_paths, &index) {
            Ok(datapackage) => datapackage,
            Err(datapackage_error) => {
                return Err(WaczError::DataPackageError(datapackage_error));
            }
        };
        let datapackage_digest = datapackage.digest();

        Ok(Self {
            datapackage,
            datapackage_digest,
        })
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
            let (mut entry, config) = archive
                .new_file(file_path)
                .compression_method(compression_method)
                .start()
                .unwrap();

            let encoder = match compression_method {
                CompressionMethod::Store => &mut entry,
                CompressionMethod::Deflate => todo!(), // provide a deflate encoder
                _ => todo!(),                          // return an error
            };
            // Wrap the file in a ZipDataWriter, which will track information for the
            // Zip data descriptor (like uncompressed size and CRC).
            let mut writer = config.wrap(encoder);

            // Copy the data to the writer.
            std::io::copy(&mut &*file_data, &mut writer).unwrap();

            // Finish the file, which will return the finalized data descriptor
            let (_, descriptor) = writer.finish().unwrap();

            let uncompressed_size = descriptor.uncompressed_size();

            println!("wrote {uncompressed_size} bytes to {file_path}");

            // Write out the data descriptor and return the number of bytes the data compressed to.
            entry.finish(descriptor).unwrap();
        }

        // Create a new Zip archive in memory.
        let mut output = Vec::with_capacity(14_000);
        let mut archive = ZipArchiveWriter::new(&mut output);

        // iterate over every resource in the datapackage
        for datapackage_resource in &self.datapackage.resources {
            add_file_to_archive(
                &mut archive,
                CompressionMethod::Store,
                &datapackage_resource.content,
                &datapackage_resource.path,
            );
        }

        // add datapackage file
        add_file_to_archive(
            &mut archive,
            CompressionMethod::Store,
            self.datapackage.to_string().as_bytes(),
            "datapackage.json",
        );

        // add digest file
        add_file_to_archive(
            &mut archive,
            CompressionMethod::Store,
            self.datapackage_digest.to_string().as_bytes(),
            "datapackage-digest.json",
        );

        // Finish the archive, which will write the central directory.
        archive.finish()?;

        return Ok(output);
    }
}

#[derive(Debug)]
pub enum WaczError {
    WarcFileError(String),
    DataPackageError(DataPackageError),
}
impl fmt::Display for WaczError {
    fn fmt(&self, message: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WarcFileError(file_path) => {
                return write!(message, "No file found at {file_path}");
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
            Self::WarcFileError(_) => return None,
            Self::DataPackageError(datapackage_error) => return Some(datapackage_error),
        }
    }
}
