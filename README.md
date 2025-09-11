# Wacksy

[![Software Heritage Archive](https://archive.softwareheritage.org/badge/origin/https://github.com/bodleian/wacksy/)](https://archive.softwareheritage.org/browse/origin/?origin_url=https://github.com/bodleian/wacksy)
![Deps.rs Crate Dependencies (latest)](https://img.shields.io/deps-rs/wacksy/latest)
![Crates.io Total Downloads](https://img.shields.io/crates/d/wacksy)

An experimental Rust library for ~~reading and~~ writing ᴡᴀᴄᴢ files.

## Install

With cargo installed, run the following command in your project directory:

```
cargo add wacksy
```

## Example

This library provides two main ᴀᴘɪ functions.
`from_file()` takes a ᴡᴀʀᴄ file and returns a structured representation of a ᴡᴀᴄᴢ object.
`zip()` takes a ᴡᴀᴄᴢ object and zips it up to a byte array using [rawzip](https://github.com/nickbabcock/rawzip).

```rust
fn main() -> Result<(), Box<dyn Error>> {
    let warc_file_path = Path::new("example.warc.gz"); // set path to your ᴡᴀʀᴄ file
    let wacz_object = WACZ::from_file(warc_file_path)?; // index the ᴡᴀʀᴄ and create a ᴡᴀᴄᴢ object
    let zipped_wacz: Vec<u8> = wacz_object.as_zip_archive()?; // zip up the ᴡᴀᴄᴢ
    fs::write("example.wacz", zipped_wacz)?; // write out to file
    Ok(())
}
```

See [the documentation](https://docs.rs/wacksy/latest/wacksy/) for more details.

## Background

According to [Ed Summers](https://inkdroid.org/2022/07/09/wacz-images/), a ᴡᴀᴄᴢ file is "really just [a ᴢɪᴘ file](https://chaos.social/@ki/111680421462204605) that contains ᴡᴀʀᴄ data and metadata at predicatble file locations."[^code4lib_talk]

The [example in the spec](https://specs.webrecorder.net/wacz/1.1.1/) outlines what a ᴡᴀᴄᴢ file should contain:

```
archive
└── data.warc.gz
datapackage.json
datapackage-digest.json
indexes
└── index.cdx.gz
pages
└── pages.jsonl
```

[^code4lib_talk]: For more discussion of the concept, see the talk "[Web Archives in Digital Repositories](https://www.youtube.com/watch?v=dtd5Os5t0Io&t=1513s)" by Ilya Kremer and Ed Summers at Code4Lib 2022.

### Similar libraries

* [py-wacz](https://github.com/webrecorder/py-wacz) for python
* [js-wacz](https://github.com/harvard-lil/js-wacz) for javascript

## License

[MIT](https://github.com/bodleian/wacksy/blob/main/LICENSE) © [Bodleian Libraries](https://www.bodleian.ox.ac.uk/) and contributors
