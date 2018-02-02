use std::fs::File;
use std::path::{Path, PathBuf};
use std::convert::From;
use std::result::Result;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std;
use std::boxed::Box;
use self::png::*;
use self::png;
use self::jpeg::*;
use self::jpeg;
// use self::pnm::*;
// use self::pnm;
// use self::ico::*;
// use self::ico;
// use self::tiff::*;
// use self::tiff;
// use self::tga::*;
// use self::tga;
// use self::bmp::*;
// use self::bmp;
// use self::gif::*;
// use self::gif;
use image::*;
use image::ColorType;
use exif::jpeg as exif_jpeg;


#[derive(Debug)]
pub enum ImageWithMetadataError {
    //Error from image crate
    DecoderError(ImageError),
    //Internal error: described by String
    Internal(String),
}

pub struct Exif {
    pub buf: Vec<u8>, // Vector containing the whole exif segment
    pub seg_size: usize, // Size of the exif segment
}

impl Exif {
    pub fn new(buf: Vec<u8>, seg_size: usize) -> Exif {
        Exif {
            buf,
            seg_size,
        }
    }
}

pub struct JPEGMetadata {
    exif: Exif,
}

impl JPEGMetadata {
    pub fn new_from_path(path: &PathBuf) -> std::io::Result<JPEGMetadata> {
        Ok(JPEGMetadata {
            exif: exif_jpeg::extract_exif(File::open(path)?)?,
        })
    }
    
    // /// Extract Metadata from the file at path 'path' returning Result<()> If successful
    // pub fn extract_from_path(path: &PathBuf) -> Result<()> {
        
    // }

    // /// Extract Metadata from the file 'file' returning Result<()> If successful
    // pub fn extract_from_file(file: &mut File) -> Result<()> {
        
    // }
    
    /// Insert Metadata to the file at path 'path' returning Result<()> If sucessful
    pub fn insert_to_path(&self, path: &PathBuf) -> std::io::Result<()> {
        exif_jpeg::insert_exif_at_path(path, &self.exif)?;
        Ok(())
    }
    
    // /// Insert Metadata to the file 'File' returning Result<()> If sucessful
    // pub fn insert_to_file(&self, file: &mut file) -> Result<()> {
        
    // }
}

pub struct JPEGDecoderWithMetadata {
    /// Only exif is actually supported
    pub metadata: JPEGMetadata,
    decoder: JPEGDecoder<File>,
}

impl JPEGDecoderWithMetadata  {
    /// Create new instance from 'path'
    pub fn new_from_path(path: &PathBuf) -> Result<JPEGDecoderWithMetadata, ImageWithMetadataError> {
        let input_file = File::open(path)?;
        let decoder = JPEGDecoder::new(input_file);
        let metadata = JPEGMetadata::new_from_path(path)?;
        Ok(JPEGDecoderWithMetadata {
            metadata,
            decoder,
        })
    }
    /// Save Metadata to file at 'path'
    pub fn save_metadata_to_file(&self, path: &PathBuf) -> std::io::Result<()> {
        self.metadata.insert_to_path(path)?;
        Ok(())
    }
}


impl ImageDecoder for JPEGDecoderWithMetadata {
    fn dimensions(&mut self) -> ImageResult<(u32, u32)> {
        self.decoder.dimensions()
    }
    
    fn colortype(&mut self) -> ImageResult<ColorType> {
        self.decoder.colortype()
    }
    
    fn row_len(&mut self) -> ImageResult<usize> {
        self.decoder.row_len()
    }
    
    fn read_scanline(&mut self, buf: &mut [u8]) -> ImageResult<u32> {
        self.decoder.read_scanline(buf)
    }
    
    fn read_image(&mut self) -> ImageResult<DecodingResult> {
        self.decoder.read_image()
    }
    
    fn is_animated(&mut self) -> ImageResult<bool> {
        self.decoder.is_animated()
    }
    
    fn into_frames(self) -> ImageResult<Frames> {
        self.decoder.into_frames()
    }
    
    fn load_rect(&mut self, x: u32, y: u32, length: u32, width: u32) -> ImageResult<Vec<u8>> {
        self.decoder.load_rect(x, y, length, width)
    }
}

impl From<ImageError> for ImageWithMetadataError {
    fn from(error: ImageError) -> ImageWithMetadataError {
        ImageWithMetadataError::DecoderError(error)
    }
}

impl From<std::io::Error> for ImageWithMetadataError {
    fn from(error: std::io::Error) -> ImageWithMetadataError {
        ImageWithMetadataError::Internal(error.description().to_string())
    }
}

impl Display for ImageWithMetadataError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ImageWithMetadataError::Internal(ref err_string) => write!(f, "{}", err_string),
             ImageWithMetadataError::DecoderError(ref err) => err.fmt(f),
        }
    }
}

impl Error for ImageWithMetadataError {
    fn description(&self) -> &str {
        match *self {
            ImageWithMetadataError::DecoderError(ref err) => err.description(),
            ImageWithMetadataError::Internal(ref err) => err.as_str(),
        }
    }
    fn cause(&self) -> Option<&Error> {
        match *self {
            ImageWithMetadataError::DecoderError(ref err) => Some(err),
            ImageWithMetadataError::Internal(_) => None,
        }
    }
}
