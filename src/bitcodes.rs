use std::mem::size_of;

pub struct BitReader<'a, I: Iterator<Item = &'a u8>> {
    cur_byte: u8,
    cur_bit: usize,
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
    fn read_bits<const N: usize>(&mut self) -> Option<usize> {
        if cfg!(target_endian = "big") {
            panic!("read_bits not supported for big endian architectures")
        }
        // kind of redundant since bytes are 8 bits by default in rust
        const BITS_PER_BYTE: usize = 8;
        
        assert!(size_of::<usize>() * BITS_PER_BYTE >= N);
        assert!(N > 0);


        let mut res: usize = 0;
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

            let bits_read = if n > rem_bits {
                rem_bits
            } else {
                n
            };
            let mask = (1 << bits_read) - 1;
            res |= (mask & (self.cur_byte >> self.cur_bit) as usize) << (N - n);
            n -= bits_read;
            self.cur_bit += bits_read;
        }

        Some(res)
    }

    pub fn read_bit(&mut self) -> Option<u8> {
        self.read_bits::<1>().map(|x| x as u8)
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
