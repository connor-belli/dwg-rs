use std::{fs::{self}, path::PathBuf};

use crate::{bitcodes::BitReader, types::CodePage, version::DWGVersion};

pub struct Dwg {
    version: DWGVersion,
}

fn read_obj_free_space<'a, I: Iterator<Item = &'a u8>>(
    bit_reader: &mut BitReader<'a, I>,
) -> Option<()> {
    if bit_reader.get_version() <= DWGVersion::AC1021 {
        let _x = bit_reader.read_raw_long()?;
        let _approx_n_objects = bit_reader.read_raw_long()?;
        let _y = bit_reader.read_raw_longlong()?;
        let _object_section_offset = bit_reader.read_raw_long()?;
        let _n_64b_vals = bit_reader.read_raw_char()?;
        let _max32 = bit_reader.read_raw_longlong()?;
        let _max64 = bit_reader.read_raw_longlong()?;
        let _maxtbl = bit_reader.read_raw_longlong()?;
        let _maxrl = bit_reader.read_raw_longlong()?;
    } else {
        let _ = bit_reader.read_raw_longlong()?;
        let _approx_n_objects = bit_reader.read_raw_longlong()?;
        let _ = bit_reader.read_raw_longlong()?;
        let _max32 = bit_reader.read_raw_longlong()?;
        let _max32hi = bit_reader.read_raw_longlong()?;
        let _max64 = bit_reader.read_raw_longlong()?;
        let _max64hi = bit_reader.read_raw_longlong()?;
        let _maxtbl = bit_reader.read_raw_longlong()?;
        let _maxtblhi = bit_reader.read_raw_longlong()?;
        let _maxrl = bit_reader.read_raw_longlong()?;
        let _maxrlhi = bit_reader.read_raw_longlong()?;
    }
    Some(())
}

fn read_r2000_header<'a, I: Iterator<Item = &'a u8>>(
    bit_reader: &mut BitReader<'a, I>,
) -> Option<()> {
    let version = bit_reader.read_version()?;
    bit_reader.set_version(version);

    // Read 6 bytes, unknown purpose
    for _ in 0..5 {
        let res = bit_reader.read_raw_char()?;
        // Sanity check, find dlls with nonzero elements in these positions
        assert_eq!(res, 0);
    }
    bit_reader.read_raw_char()?;
    // Skip next byte, should be 1
    assert_eq!(bit_reader.read_raw_char(), Some(1));

    // Read image sentinel at 0x0D
    let _image_sentinel_seeker = bit_reader.read_raw_long()?;

    // Two unknown bytes
    bit_reader.read_raw_char()?;

    // Read section-locator record starting at 0x15
    let n_records = bit_reader.read_raw_long()?;
    for _record in 0..n_records {
        let _unused = bit_reader.read_raw_char()?;
        let _seeker = bit_reader.read_raw_long()?;
        let _size = bit_reader.read_raw_long()?;
    }

    // TODO: Verify CRC
    let _crc = bit_reader.read_raw_short()?;
    
    // sentinel after crc
    let sentinel = [
        0x95, 0xA0, 0x4E, 0x28, 0x99, 0x82, 0x1A, 0xE5, 0x5E, 0x41, 0xE0, 0x5F, 0x9D, 0x3A, 0x4D,
        0x00,
    ];

    // Verify that sentinel is equal to expected value
    for byte in sentinel {
        assert_eq!(byte, bit_reader.read_raw_char()? as u8);
    }
    Some(())
}

impl Dwg {
    pub fn read_from_file(file_name: &str) -> Option<Dwg> {
        let bytes = fs::read(file_name).unwrap();
        let mut bit_reader = BitReader::new(bytes.iter());

        read_r2000_header(&mut bit_reader);
        unimplemented!()
    }
}

#[test]
fn test_r2000_header() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("test_data/Line.dwg");

    let bytes = fs::read(d.as_path().to_str().unwrap()).unwrap();
    let mut bit_reader = BitReader::new(bytes.iter());
    // Currently just attempt to read the data
    read_r2000_header(&mut bit_reader);
}
