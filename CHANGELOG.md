# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.3](https://github.com/bodleian/wacksy/compare/v0.0.2...v0.0.3) - 2025-09-30

### Added

- add more error handling, and specifically in the indexer

### Fixed

- *(indexer)* generate short uuids for each page record, fixes #57
- *(indexer)* add placeholder ids to pages.jsonl records, passes frictionless validation and fixes #54
- *(indexer)* add title to pages.jsonl header, closing #51 (again)
- *(indexer)* add formatted header when writing out pages.jsonl, closes #52
- *(indexer)* rename timestamp to ts in page_record.rs, closes #53
- *(datapackage)* the file name has to be named just 'name'
- *(indexer)* shift the record counter forward by 1, fixes #47

### Other

- add id-token permissions to release action
- remove CARGO_REGISTRY_TOKEN to use Crates.io Trusted Publishing
- correct name of function in readme
- upgrade rawzip to 0.4.1 and rewrite as_zip_archive with new api, closes #49 and #55
- *(indexer)* explain CriticalRecordError
- change documentation and doctest example to match function renamed in d7cae2f
- renamed zip function, fixes #56
- allow multiple crate versions
- *(datapackage)* move datapackage validation into unit test
- *(indexer)* formatter moved around the order of the imports
- add json schema validation to integration tests, closes #50
- use pretty assertions
- add setuptools to requirements.txt
- specify python version
- add requirements.txt
- use setup-python in ci
- fix broken test which relied on example
- add output.wacz to gitignore
- replace examples directory with doctest
- *(readme)* add usage example
- document from_file implemented on WACZ
- bump MSRV to use .display() in the indexer
- bump MSRV and cargofile following release of Rust 1.89
- from_file now returns a result with DataPackageError
- initial work on making a simpler API
- new link in readme
- use new git checkout action
- *(indexer)* wrap indexer in Index struct
- *(indexer)* only attempt to create a page record if the cdxj indexing was successful
- pass whole index into datapackage
- update/add badges to readme
- change repository link to https://github.com/bodleian/wacksy/

## [0.0.2](https://github.com/bodleian/wacksy/compare/v0.0.1...v0.0.2) - 2025-08-06

This release involves some refactoring, different parts of the indexer are now in their own modules.
As a result of this, it was easier to write unit tests for each resource, so I've now done that, along with two integration tests.
The tests just cover the basics, I expect to expand these in future to check errors and other things.

The page record indexer now only indexes records according to a set of conditions which _guarantee_ the record is a web document.
Unfortunately the WACZ spec does not define what a page is in terms we can use here, so I have come up with the following conditions:

- The WARC record type is either Response, Revisit, or Resource
- The HTTP content-type is either `text/html`, `application/xhtml+xml`, or `text/plain`.
- The HTTP status code is 200 OK.

This is an imperfect best-guess attempt to pick out things which _might_ be pages from a WARC file.
The reason I filter for successful status codes is I realised that some failed requests return HTML pages in the response along with a 404 error.
Those are definitely _pages_, but I guess they're not what people want out of the `pages.jsonl` index.

I made a brief attempt to replace sha256 with the faster [blake3](https://github.com/BLAKE3-team/BLAKE3) hashing algorithm, but this breaks compatibility with `py-wacz`.
I think this is something which will have to wait until blake3 can be integrated into the python standard library [as part of hashlib](https://github.com/python/cpython/issues/83479).

### Dependencies

- This library now depends on [surt-rs](https://github.com/mijho/surt-rs) to create searchable url strings. It's a fairly minimal library and is more comprehensive than my own attempt to write a surt-ing function.
- Bump [rawzip](https://github.com/nickbabcock/rawzip) to 0.3 ([#41](https://github.com/bodleian/wacksy/pull/41)), thanks [@nickbabcock](https://github.com/nickbabcock)!

## [0.0.1](https://github.com/bodleian/wacksy/compare/v0.0.1-beta...v0.0.1) - 2025-06-20

As of this point, the WACZ and indexer can output (almost) everything needed from a WARC file to a fully spec-compliant WACZ file.
The last thing missing was the pages.jsonl file, which is now produced when reading through the WARC file as part of the indexer.
I want to avoid reading through the WARC twice to produce two files, so have wrapped everything into one indexer, again there's probably a better way of doing this.

The other happy change in this release is removing code duplication from the WARC reader in case of gzipped and non-gzipped files.
First time I've tried using type generics in Rust, the code is messy, but it works.

### Added

- *(indexer)* Use type generics to eliminate code duplication when iterating through records, this finally gets rid of an awkward situation where I was having to maintain two separate iterators .
- add pages indexer to wacz writer, with a struct for page records, this is the main thing in this release.

### Fixed

- add newline to page records, needed for pages.jsonl format, closes #37, nice and easy change
- *(indexer)* skip serialising null fields in page record
- *(datapackage)* pass cdxj_index_bytes through to the datapackage

### Other

Lots more little documentation/readme changes and additions. Code refactoring, etc.

- *(indexer)* use core instead of standard libraries for error formatting
- add serde features to dependencies, update cargofile
- *(datapackage)* move compose_datapackage into datapackage implementation
- *(datapackage)* DataPackageResource::new now returns a result/error rather than panicking
- *(indexer)* use httparse to parse http status code from response and remove the happily redundant cut_http_headers_from_record function

## [0.0.1-beta](https://github.com/bodleian/wacksy/compare/v0.0.1-alpha...v0.0.1-beta) - 2025-05-16

Work on this version was mostly refactoring, adding structured types and error handling, and some documentation (only just started).

Still on my todo list is to use the indexer to also create pages.jsonl files.

### Fixed

- replace wrapping_add in loop counter with enumerate, closes #29
- *(indexer)* return the same error message for gzipped and non-gzipped files. I have tried to simplify the code for processing both gzipped and non-gzipped files. There's still unnecessary duplication but it's the best I can do for the moment.

### Other

- document some DataPackage structs, better documentation coming once this is properly finished!
- as a style change, this now uses explicit returns everywhere, and I have set lints in cargo.toml to enforce this
- *(indexer)* many of the index functons are now implemented on types. The completed index is returned *as a struct*, which has a display implementation to write it out to json(l).
- *(datapackage)* propogate errors upwards, there are still some panics, but structured error handling is a lot more comprehensive now. Happy and unhappy paths are a little clearer to identify.
- update README with link to a funny meme :)

## [0.0.1-alpha](https://github.com/bodleian/wacksy/releases/tag/v0.0.1-alpha) - 2025-04-05

At this stage the library can read a WARC file to produce a CDXJ index, and a datapackage.

### Added

- *(indexer)* types for DataPackage and DataPackageResource
- *(indexer)* various types for CXDJIndexRecord
