use std;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::io::{Read, Write, SeekFrom, Result, Seek, Error};
use std::path::PathBuf;
use byteorder::*;
use byteorder::{LE, BE, NativeEndian};
use metadata::Exif;

/// Extract the first exif segment from file, consuming it.
pub fn extract_exif(mut file: File) -> Result<Exif> {
    let (seg_size, offset) = find_exif(&mut file)?;
    if offset == 0 || seg_size == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to extract exif segment: Failed to find exif segment"));
    }
    const HDR_SIZE: usize = 6;
    let mut segment = vec![0u8; seg_size];
    file.read_exact(&mut segment)?;
    return Ok(Exif::new(segment, seg_size));
}


/// Position the internal file offset to the beginning of the exif(first) segment i.e at the size field, returning the size field and the current internal position of the file.
pub fn find_exif(file: &mut File) -> Result<(usize, usize)> {
    let rdr = file;
    let mut position: usize = 0;
    loop {
        if rdr.read_u8()? != 0xFF {
            rdr.seek(SeekFrom::Start(0))?;
            return Ok((0, 0));
        }
        let seg_id = rdr.read_u8()?; //Found segment, get his id
        position += 2;
        match seg_id {
            0x0 => {}, // Byte stuffing, skip this
            0xD8 => {}, //Image start id
            0xE1 => {
                let seg_size = (rdr.read_u16::<BE>()? - 2) as usize; //Need to subtract 2 to size because the size of the field is counted in it
                const HDR_SIZE: usize = 6;
                let mut hdr = [0u8; HDR_SIZE];
                rdr.read_exact(&mut hdr)?; // Read Exif HDR
                if &hdr == &[b'E', b'x', b'i', b'f', 0x00, 0x00] {
                    rdr.seek(SeekFrom::Current((-(HDR_SIZE as i64 + 2)) as i64))?; //Reposition internal file offset to size field of the Exif segment 
                    return Ok((seg_size, position));
                }
                rdr.seek(SeekFrom::Current((seg_size - HDR_SIZE) as i64))?;
            }
            _ => {
                // skip segment
                let len = rdr.read_u16::<BE>()? - 2;
                rdr.seek(SeekFrom::Current(len as i64))?;
                position += len as usize + 2;
            }
            
        }
    }
}

/// Insert exif segment provided as the argument exif: Vec<u8>, remplacing the whole old segment.
pub fn insert_exif_at_path(path: &PathBuf, exif: &Exif) -> Result<()> {

    let mut old_file_vec = Vec::new();
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(path)?;
    let old_file_size = file.read_to_end(&mut old_file_vec)?;
    // Reset internal position of file
    file.seek(SeekFrom::Start(0))?;
    let (seg_size, mut exif_index) = find_exif(&mut file)?;
    file.set_len(0)?;
    let mut tmp_exif = exif.buf.clone();
    if exif_index == 0 {
        println!("No exif segment already existing to be found: Inserting new marker");
        exif_index = 2;
        tmp_exif.insert(0, 0xE1);
        tmp_exif.insert(0, 0xFF);
    }
    old_file_vec.splice(exif_index..(exif_index + seg_size), tmp_exif);
    file.seek(SeekFrom::Start(0))?;
    file.write_all(old_file_vec.as_slice())?;
    Ok(())
}
