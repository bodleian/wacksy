# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/bodleian/wacksy/compare/v0.1.0...v0.1.1) - 2025-09-30

This first minor release includes a new API, made up of two functions:

1. `from_file()` takes a WARC file, indexes it and produces a structured representation of a WACZ object.
2. `as_zip_archive()` takes that structured representation and writes it out to a zip archive.

> [!NOTE]
> Despite the more serious commitment to a stable API in this version, this library is still not ready for professional use. I'm still not sure the indexer is correctly calculating the byte offset of each record in the WARC file. Output WACZes do not replay properly with [ReplayWeb.page](https://replayweb.page/).

I've been trying to model the construction pattern to fit the conditions of the format. For example, all the resources in a collection must be defined in the datapackage, or a page record cannot be created without a matching CDX record. These conditions should be satisfied by a carefully ordered process, with each step flowing logically into the next. The indexer and the new API were (re)written with this goal in mind.

### Pages.jsonl

Each page listed in pages.jsonl now gets assigned a short uuid, I am [not sure if this is necessary](https://github.com/webrecorder/specs/issues/167), but without an id the page records don't pass validation against the [frictionless datapackage schema](https://specs.frictionlessdata.io/schemas/data-package.json).
The pages file also includes a header line in the form `{"format":"json-pages-1.0","id":"pages","title":"All Pages"}"`.

### Unit test coverage and JSON schemas

I've converted most of the integration tests into unit tests, and where possible replaced string comparison with validation against a JSON Schema.
Serialisation of values to JSON still feels too ✨magical✨ to me; I don't necessarily want to change any of the code here, but I would like to use serde more _confidently_.

### Fixes

- *(indexer)* move WARC record counter forward by 1 because the iterator enumeration is zero-indexed. Easy mistake.

### Other

- Renamed `zip()` to `as_zip_archive()`, thanks to @anna-hope for [the suggestion](https://github.com/bodleian/wacksy/issues/43#issuecomment-3233606691) and @eviejmorris for [the fix](https://github.com/bodleian/wacksy/commit/d7cae2fcee0656eb01eecd26b20a447ecc01cabd).
- Replaced example code with a doctest, and added usage example to readme.
- Bumped the MSRV to 1.87.
- Moved the repository to the Bodleian organisation on GitHub.
- Use pretty assertions in tests.
- Wrote this changelog.

### Dependencies

- Updated [rawzip](https://lib.rs/crates/rawzip) to version 0.4.1, and refactored `as_zip_archive` to handle the new API.
- Added [short uuid](https://lib.rs/crates/short-uuid) for generating page ids.

## [0.1.0](https://github.com/bodleian/wacksy/compare/v0.0.2...v0.1.0) - 2025-09-26

Accidental early release of v0.1.0 without a changelog.
Changelog included in patch version.

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
- Bump [rawzip](https://github.com/nickbabcock/rawzip) to 0.3 ([#41](https://github.com/bodleian/wacksy/pull/41)), thanks @nickbabcock!

## [0.0.1](https://github.com/bodleian/wacksy/compare/v0.0.1-beta...v0.0.1) - 2025-06-20

As of this point, the WACZ and indexer can output (almost) everything needed from a WARC file to a fully spec-compliant WACZ file.
The last thing missing was the pages.jsonl file, which is now produced when reading through the WARC file as part of the indexer.
I want to avoid reading through the WARC twice to produce two files, so have wrapped everything into one indexer, again there's probably a better way of doing this.

The other happy change in this release is removing code duplication from the WARC reader in case of gzipped and non-gzipped files.
First time I've tried using type generics in Rust, the code is messy, but it works.

### Added

- *(indexer)* Use type generics to eliminate code duplication when iterating through records, this finally gets rid of an awkward situation where I was having to maintain two separate iterators.
- Add pages indexer to wacz writer, with a struct for page records, this is the main thing in this release.

### Fixed

- add newline to page records, needed for pages.jsonl format, closes #37, nice and easy change
- *(indexer)* skip serialising null fields in page record
- *(datapackage)* pass cdxj_index_bytes through to the datapackage

### Other

Lots more little documentation/readme changes and additions. Code refactoring, etc.

- *(indexer)* use core instead of standard libraries for error formatting
- Add serde features to dependencies, update cargofile.
- *(datapackage)* move compose_datapackage into datapackage implementation
- *(datapackage)* `DataPackageResource::new` now returns a result/error rather than panicking
- *(indexer)* use httparse to parse http status code from response and remove the happily redundant cut_http_headers_from_record function

## [0.0.1-beta](https://github.com/bodleian/wacksy/compare/v0.0.1-alpha...v0.0.1-beta) - 2025-05-16

Work on this version was mostly refactoring, adding structured types and error handling, and some documentation (only just started).

Still on my to-do list is to use the indexer to also create pages.jsonl files.

### Fixed

- Replace wrapping_add in loop counter with enumerate, closes #29.
- *(indexer)* return the same error message for gzipped and non-gzipped files. I have tried to simplify the code for processing both gzipped and non-gzipped files. There's still unnecessary duplication but it's the best I can do for the moment.

### Other

- Document some DataPackage structs, better documentation coming once this is properly finished!
- As a style change, this now uses explicit returns everywhere, and I have set lints in cargo.toml to enforce this
- *(indexer)* many of the index functions are now implemented on types. The completed index is returned *as a struct*, which has a display implementation to write it out to json.
- *(datapackage)* propagate errors upwards, there are still some panics, but structured error handling is a lot more comprehensive now. Happy and unhappy paths are a little clearer to identify.
- Update README with link to a funny meme 🙂

## [0.0.1-alpha](https://github.com/bodleian/wacksy/releases/tag/v0.0.1-alpha) - 2025-04-05

At this stage the library can read a WARC file to produce a CDXJ index, and a datapackage.

### Added

- *(indexer)* types for `DataPackage` and `DataPackageResource`
- *(indexer)* various types for `CXDJIndexRecord`
