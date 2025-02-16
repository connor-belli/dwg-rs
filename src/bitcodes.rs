//! A struct to read DWG datatypes from a byte stream
//!
//! See chapter 2 of the ODS for details on the structure of the datatypes that can be read
//!
//! This module currently is fairly unoptimized; however, given the bitwise nature of DWGs,
//! the API should stay the same and can't really be made any faster
use std::mem::size_of;

use crate::version::DWGVersion;

/// A structure that wraps a `Iterator<&u8>` that enables reading DWG datatypes from a byte stream
///
/// This struct does not allow for modification or writing of the DWG and instead will be
/// performed by a future struct instead
///
/// This struct does no buffering and this functionality needs to be implemented from the iterator
pub struct BitReader<'a, I: Iterator<Item = &'a u8>> {
    cur_byte: u8,
    cur_bit: u32,
    iter: I,
    version: DWGVersion,
}

impl<'a, I: Iterator<Item = &'a u8>> BitReader<'a, I> {
    /// Creates a new `BitReader` by wrapping an `Iterator<&u8>`
    ///
    /// Assumes a Version of AC1015 (R2000) initially  
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            cur_byte: 0,
            cur_bit: 8,
            version: DWGVersion::AC1015,
        }
    }

    pub fn get_version(&self) -> DWGVersion {
        self.version
    }

    pub fn set_version(&mut self, version: DWGVersion) {
        self.version = version
    }

    /// Read 6 byte magic number and return the DWG version
    ///
    /// This will not update the version of the reader automatically
    pub fn read_version(&mut self) -> Option<DWGVersion> {
        let mut bytes = [0u8; 6];
        for byte in bytes.iter_mut() {
            *byte = self.read_bits::<8>()? as u8;
        }
        DWGVersion::from_magic(&bytes)
    }

    /// Reads N bits to a usize and returns the results
    ///
    /// This will return None if there are less than N bits in the stream
    fn read_bits<const N: u32>(&mut self) -> Option<u32> {
        if cfg!(target_endian = "big") {
            panic!("read_bits not supported for big endian architectures")
        }
        // kind of redundant since bytes are 8 bits by default in rust
        const BITS_PER_BYTE: u32 = 8;

        assert!(size_of::<u32>() * BITS_PER_BYTE as usize >= N as usize);
        assert!(N > 0);

        let mut res: u32 = 0;
        let mut n = N;
        while n > 0 {
            let mut rem_bits = BITS_PER_BYTE - self.cur_bit;
            if rem_bits == 0 {
                if let Some(byte) = self.iter.next() {
                    self.cur_byte = *byte;
                    self.cur_bit = 0;
                    rem_bits = BITS_PER_BYTE - self.cur_bit;
                } else {
                    return None;
                }
            }

            let bits_read = if n > rem_bits { rem_bits } else { n };
            let mask = (1 << bits_read) - 1;
            res |= (mask & (self.cur_byte >> self.cur_bit) as u32) << (N - n);
            n -= bits_read;
            self.cur_bit += bits_read;
        }

        Some(res)
    }

    pub fn read_bit(&mut self) -> Option<u8> {
        self.read_bits::<1>().map(|x| x as u8)
    }

    pub fn read_bit_triplet(&mut self) -> Option<u8> {
        let mut byte = 0;
        for _ in 0..3 {
            let bit = self.read_bit()?;
            byte = byte << 1 | bit;
            if bit == 0 {
                break;
            }
        }
        Some(byte)
    }

    pub fn read_bitshort(&mut self) -> Option<i16> {
        let flag = self.read_bits::<2>()?;
        match flag {
            0x0 => self.read_raw_short(),
            0x1 => self.read_bits::<8>().map(|x| x as i16),
            0x2 => Some(0),
            0x3 => Some(256),
            _ => unreachable!(),
        }
    }

    pub fn read_bitlong(&mut self) -> Option<i32> {
        let flag = self.read_bits::<2>()?;
        match flag {
            0x0 => self.read_raw_long(),
            0x1 => self.read_bits::<8>().map(|x| x as i32),
            0x2 => Some(0),
            0x3 => Some(256),
            _ => unreachable!(),
        }
    }

    pub fn read_bitlonglong(&mut self) -> Option<i64> {
        let flag = self.read_bits::<2>()?;
        match flag {
            0x0 => {
                let x1 = self.read_raw_long()? as u64;
                let x2 = self.read_raw_long()? as u64;
                Some((x2 << 32 | x1) as i64)
            }
            0x1 => self.read_bits::<8>().map(|x| x as i64),
            0x2 => Some(0),
            0x3 => Some(256),
            _ => unreachable!(),
        }
    }

    pub fn read_bitdouble(&mut self) -> Option<f64> {
        let flag = self.read_bits::<2>()?;
        match flag {
            0x0 => self.read_raw_double(),
            0x1 => Some(1.0),
            0x2 => Some(0.0),
            _ => unreachable!(),
        }
    }

    pub fn read_modular_char(&mut self) -> Option<i32> {
        let mut res = 0i32;
        let mut i = 0;
        loop {
            let byte = self.read_bits::<8>()? as u8;
            res |= ((byte & !(1 << 7)) as i32) << (i * 7);
            if byte & (1 << 7) == 0 {
                break;
            }
            i += 1;
        }
        Some(res)
    }

    pub fn read_modular_short(&mut self) -> Option<i32> {
        let mut res = 0i32;
        let mut i = 0;
        loop {
            let byte = self.read_bits::<16>()? as u16;
            res |= ((byte & !(1 << 15)) as i32) << (i * 15);
            if byte & (1 << 15) == 0 {
                break;
            }
            i += 1;
        }
        Some(res)
    }

    pub fn read_raw_char(&mut self) -> Option<i8> {
        self.read_bits::<8>().map(|x| x as i8)
    }

    pub fn read_raw_short(&mut self) -> Option<i16> {
        self.read_bits::<16>().map(|x| x as i16)
    }

    pub fn read_raw_long(&mut self) -> Option<i32> {
        self.read_bits::<32>().map(|x| x as i32)
    }

    pub fn read_raw_longlong(&mut self) -> Option<i64> {
        let x1 = self.read_bits::<32>()? as u64;
        let x2 = self.read_bits::<32>()? as u64;
        Some((x2 << 32 | x1) as i64)
    }

    pub fn read_raw_double(&mut self) -> Option<f64> {
        let x1 = self.read_bits::<32>()? as u64;
        let x2 = self.read_bits::<32>()? as u64;
        Some(f64::from_bits(x2 << 32 | x1))
    }

    pub fn read_bit_extrusion(&mut self) -> Option<(f64, f64, f64)> {
        if self.version >= DWGVersion::AC1015 {
            // NOTE: ODS does not specifically say that post R16 versions use this method,
            // only that R16 uses this method
            let bit = self.read_bit()?;
            if bit == 1 {
                return Some((0.0, 0.0, 1.0));
            }
        }
        let x1 = self.read_bitdouble()?;
        let x2 = self.read_bitdouble()?;
        let x3 = self.read_bitdouble()?;
        Some((x1, x2, x3))
    }

    pub fn read_bitdouble_with_default(&mut self) -> Option<f64> {
        if self.version >= DWGVersion::AC1015 {
            let bit = self.read_bit()?;
            if bit == 1 {
                return Some(0.0);
            }
        }
        self.read_bitdouble()
    }

    pub fn read_cm_color_short(&mut self) -> Option<i16> {
        self.read_bitshort()
    }

    pub fn read_object_type(&mut self) -> Option<i16> {
        if self.version <= DWGVersion::AC1021 {
            self.read_bitshort()
        } else {
            let flags = self.read_bits::<2>()?;
            match flags {
                0x0 => self.read_raw_char().map(|x| x as i16),
                0x1 => self.read_raw_char().map(|x| x as i16 + 0x1f0),
                0x2 => self.read_raw_short(),
                0x3 => self.read_raw_short(),
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn test_read_bits() {
    let buf: [_; 4] = [0xFF, 0xDD, 0xCC, 0xBB];
    let mut reader = BitReader::new(buf.iter());
    assert_eq!(reader.read_bits::<8>(), Some(0xFF));
    assert_eq!(reader.read_bits::<16>(), Some(0xCCDD));
    assert_eq!(reader.read_bits::<5>(), Some(0x1B));
    assert_eq!(reader.read_bits::<3>(), Some(0x5));
    assert_eq!(reader.read_bits::<1>(), None);
}

#[test]
fn test_read_raw_long() {
    let buf: [_; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
    let mut reader = BitReader::new(buf.iter());
    assert_eq!(reader.read_raw_long(), Some(-1));

    let buf: [_; 4] = [0x01, 0x00, 0x00, 0x00];
    let mut reader = BitReader::new(buf.iter());
    assert_eq!(reader.read_raw_long(), Some(1));
}

#[test]
fn test_read_raw_longlong() {
    let buf: [_; 8] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let mut reader = BitReader::new(buf.iter());
    assert_eq!(reader.read_raw_longlong(), Some(-1));

    let buf: [_; 8] = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut reader = BitReader::new(buf.iter());
    assert_eq!(reader.read_raw_longlong(), Some(1));
}

#[test]
fn test_read_modular_char() {
    // Opendesign specification example
    let buf: [_; 2] = [0b10000010, 0b00100100];
    let mut reader = BitReader::new(buf.iter());
    assert_eq!(reader.read_modular_char(), Some(4610));
}

#[test]
fn test_read_modular_short() {
    // Opendesign specification example
    // NOTE: First byte of example in PDF is wrong
    let buf: [_; 4] = [0b00110001, 0b11110100, 0b10001101, 0b00000000];
    let mut reader = BitReader::new(buf.iter());
    assert_eq!(reader.read_modular_short(), Some(4650033));
}
