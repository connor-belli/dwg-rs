use std::mem::size_of;

pub struct BitReader<'a, I: Iterator<Item = &'a u8>> {
    cur_byte: u8,
    cur_bit: u32,
    iter: I,
}

impl<'a, I: Iterator<Item = &'a u8>> BitReader<'a, I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            cur_byte: 0,
            cur_bit: 8,
        }
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
            }, 
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

    pub fn read_raw_double(&mut self) -> Option<f64> {
        let x1 = self.read_bits::<32>()? as u64;
        let x2 = self.read_bits::<32>()? as u64;
        Some(f64::from_bits(x2 << 32 | x1))
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
