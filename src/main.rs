
#![feature(try_trait)]
extern crate image;

 use image::*;

extern crate byteorder;
extern crate imagewithmetadata;



use imagewithexif::metadata;
use imagewithexif::metadata::{JPEGDecoderWithMetadata, ImageWithMetadataError};
use ImageWithMetadataError::*;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use image::ImageDecoder;
use image::DecodingResult::*;
use image::jpeg;
use std::convert::From;
use std::error::Error;
use std::result::Result;
use std::io::{Read, Seek, Cursor};
use std::ffi::{CString, NulError};
use std::option::NoneError;

impl From<ImageWithMetadataError> for MyError {
    fn from(err: ImageWithMetadataError) -> MyError {
        match err {
            ImageWithMetadataError::Internal(string) => MyError::Str(string),
            DecoderError(err) => MyError::Str(String::from(err.description())),
        }
    }
}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> MyError {
        MyError::Str(String::from(err.description()))
    }
}

impl From<NoneError> for MyError {
    fn from(err: NoneError) -> MyError {
        MyError::Str(format!("{:?}", err))
    }
}

impl From<NulError> for MyError {
    fn from(err: NulError) -> MyError {
        MyError::Str(String::from(err.description()))
    }
}

impl From<image::ImageError> for MyError {
    fn from(err: image::ImageError) -> MyError {
        match err {
            image::ImageError::FormatError(string) => MyError::Str(string),
            _ => MyError::Str(String::from("Unsupported error")),
        }
    }
}

#[derive(Debug)]
enum MyError {
    Str(String),
    None,
}

//use imagewithexif::exifwriter::jpeg as jpeg_writer;

fn resize(path: &PathBuf, output: &PathBuf) -> Result<(), MyError> {
    println!("trying open {:?}", path);
    // let mut input = OpenOptions::new()
    //     .write(true)
    //     .read(true)
    //     .open(path)?;

//    let exif = jpeg_writer::extract_exif(File::open(path)?)?;
    //  let mut img = jpeg::JPEGDecoder::new(input);
    let mut decoder = metadata::JPEGDecoderWithMetadata::new_from_path(path)?;
    let mut file = File::create(output)?;
    let mut encoder = jpeg::JPEGEncoder::new_with_quality(&mut file, 25);
    let (w, h) = decoder.dimensions()?;
    
    println!("resizing {:?}", path);
    match decoder.read_image()? {
        U8(vec) => encoder.encode(vec.as_slice(), w, h, decoder.colortype()?)?,
        U16(_) => panic!("U16 does not managed"),
    };
    println!("Metadata");
    decoder.save_metadata_to_file(output)?;
    println!("done {:?}", path);
    Ok(())
}


fn main() {
    use std::path::Path;
    let args: Vec<String> = std::env::args().collect();
    for arg in args.iter().skip(1) {
        let mut new_path = arg.clone();
            new_path.push_str("_resized");
        match resize(&Path::new(arg.as_str()).to_path_buf(), &Path::new(new_path.as_str()).to_path_buf()) {
            Ok(_) => (),
            Err(err) => println!("{:?}", err),
        }
    }
}
