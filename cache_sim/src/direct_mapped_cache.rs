use core::num;

use crate::mem_stats::*;
use crate::memory::*;
use crate::cache::*;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);
pub const ADDR_BITS: usize = 32;

#[derive(Debug)]
pub struct DMCache<
    const BYTES: usize,               
    const WORDS_PER_LINE: usize,      
    > {
    lines: Vec<CacheLine>,
    stats: MemStats,
}


impl<const BYTES: usize, const WORDS_PER_LINE: usize> DMCache<BYTES, WORDS_PER_LINE> {
    pub const NUM_LINES: usize = BYTES / (WORDSIZE * WORDS_PER_LINE);

    pub fn new() -> Self {
        assert!(BYTES.is_power_of_two(), "BYTES must be a power of two");
        assert!(WORDS_PER_LINE.is_power_of_two(), "WORDS_PER_LINE must be a power of two");
        assert!(Self::NUM_LINES > 0, "cache must hold ≥ 1 line");
        assert!(Self::NUM_LINES.is_power_of_two(),"NUM_LINES must be a power of two");

        let lines = vec![CacheLine::new(WORDS_PER_LINE); Self::NUM_LINES];
        
        Self { lines, stats: MemStats::new() }
    }

    pub fn print_summary(&self) {
        self.stats.print_summary();
    }
}

impl<const B: usize, const W: usize> MemoryAccess for DMCache<B, W> {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        let (_, ind, word, byte) = self.decode_addr(addr);

        let line: &CacheLine = &self.lines[ind];

        if !line.is_valid() || line.tag() != self.get_tag(addr) {
            self.stats.record_miss();
            return Err(MemoryError::NotFound);
        }
        
        self.stats.record_hit();
        
        let byte_index = WORDSIZE * word + byte;

        match size {
            DataTypeSize::Byte => {
                let byte = line.read_byte(byte_index);
                let ret_data= DataType::Byte(byte); 
                Ok(ret_data)
            }

            DataTypeSize::Halfword => {
                let val = u16::from_le_bytes([
                    line.read_byte(byte_index), 
                    line.read_byte(byte_index + 1)
                ]);
                let ret_data= DataType::Halfword(val); 
                Ok(ret_data)
            }

            DataTypeSize::Word => {
                let val = u32::from_le_bytes([
                    line.read_byte(byte_index), 
                    line.read_byte(byte_index + 1),
                    line.read_byte(byte_index + 2), 
                    line.read_byte(byte_index + 3)
                ]);
                let ret_data = DataType::Word(val);
                Ok(ret_data)
            }

            DataTypeSize::DoubleWord => {
                let val = u64::from_le_bytes([
                    line.read_byte(byte_index), 
                    line.read_byte(byte_index + 1),
                    line.read_byte(byte_index + 2), 
                    line.read_byte(byte_index + 3),
                    line.read_byte(byte_index + 4), 
                    line.read_byte(byte_index + 5),
                    line.read_byte(byte_index + 6), 
                    line.read_byte(byte_index + 7)
                ]);
                let ret_data = DataType::DoubleWord(val);
                Ok(ret_data)
            }
        }
    }

    fn stats(&self) -> &MemStats {
        &self.stats
    }

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        let (_, ind, word, byte) = self.decode_addr(addr);
        
        let byte_index = WORDSIZE * word + byte;

        let line: &CacheLine = &self.lines[ind];
        if !line.is_valid() || line.tag() != self.get_tag(addr) {
            self.stats.record_miss();
            return Err(MemoryError::NotFound);
        }
        
        self.stats.record_hit();
        
        let line: &mut CacheLine = &mut self.lines[ind];
        match data {
            DataType::Byte(val) => {
                line.write_byte(byte_index, val);
                Ok(())
            }

            DataType::Halfword(val) => {
                let bytes = val.to_le_bytes();
                for i in 0..2 {
                    line.write_byte(byte_index + i, bytes[i]);
                }
                Ok(())
            }


            DataType::Word(val) => {

                let bytes = val.to_le_bytes();
                for i in 0..4 {
                    line.write_byte(byte_index + i, bytes[i]);
                }
                Ok(())
            }

            DataType::DoubleWord(val) => {
                let bytes = val.to_le_bytes();
                for i in 0..8 {
                    line.write_byte(byte_index + i, bytes[i]);
                }
                Ok(())
            }
        }
    }

}

impl<const B: usize, const W: usize> MemLevelAccess for DMCache<B, W> {
    fn write_line(&mut self, addr: usize, _words_per_lines: usize, data: Vec<u8>) {
        let (tag, ind, _, _) = self.decode_addr(addr);
        let line: &mut CacheLine = &mut self.lines[ind];
        line.write_line(tag, data);
    }

    fn fetch_line(&self, addr: usize, _words_per_lines: usize) -> Vec<u8> {
        self.get_evict_line_data(addr)
    }
}

impl<const BYTES: usize,const WORDS_PER_LINE: usize,> CacheAddressing for DMCache<BYTES, WORDS_PER_LINE> {
    #[inline(always)]
    fn byte_bits(&self) -> usize {
        WORDSIZE.trailing_zeros() as usize
    }
    
    #[inline(always)]
    fn word_bits(&self) -> usize {
        WORDS_PER_LINE.trailing_zeros() as usize
    }

    #[inline(always)]
    fn index_bits(&self) -> usize {
        self.lines.len().trailing_zeros() as usize
    }

    fn decode_addr(&self, addr: usize) -> (usize, usize, usize, usize) {
        let bb = self.byte_bits(); // lowest bits
        let wb = self.word_bits(); // next bits
        let ib = self.index_bits(); // next bits

        let byte  =  addr & ((1 << bb) - 1);
        let word  = (addr >>  bb) & ((1 << wb) - 1);
        let index = (addr >> (bb + wb)) & ((1 << ib) - 1);
        let tag   =  addr >> (bb + wb + ib);
        (tag, index, word, byte)
    }
    
    #[inline(always)]
    fn get_tag(&self, addr: usize) -> usize {
        let (tag, ..) = self.decode_addr(addr); 
        tag
    }

    #[inline(always)]
    fn get_index(&self, addr: usize) -> usize {
        let (_, ind, ..) = self.decode_addr(addr); 
        ind
    }

    #[inline(always)]
    fn get_word_offset(&self, addr: usize) -> usize {
        let (_, _, word, ..) = self.decode_addr(addr); 
        word
    }

    #[inline(always)]
    fn get_byte_offset(&self, addr: usize) -> usize {
        let (_, _, _, byte) = self.decode_addr(addr); 
        byte
    }

    #[inline(always)]
    fn is_line_dirty(&self, addr: usize) -> bool {
        let ind = self.get_index(addr);
        self.lines[ind].is_dirty()
    }

    #[inline(always)]
    fn get_evict_line_data(&self, addr:usize) -> Vec<u8> {
        let ind = self.get_index(addr);
        self.lines[ind].get_data()
    }
    #[inline(always)]
    fn get_writeback_addr(&self, addr: usize) -> usize {
        let ind = self.get_index(addr);
        let tag = self.lines[ind].tag();          // tag currently stored in cache
        let bb = self.byte_bits();
        let wb = self.word_bits();
        let ib = self.index_bits();
        (tag << (ib + wb + bb)) | (ind << (wb + bb))
    }

    #[inline(always)]
    fn get_base_addr(&self, addr: usize) -> usize {
        let (tag, ind, _, _) = self.decode_addr(addr);
        let bb = self.byte_bits();
        let wb = self.word_bits();
        let ib = self.index_bits();
        (tag << (ib + wb + bb)) | (ind << (wb + bb)) 
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new () {
        type L1 = DMCache<1024, 8>;           // Direct-mapped, 1 KiB, 32-B lines
        let l1 = L1::new();
        println!("{:#?}", l1);
    }

    #[test]
    fn parse_addr() {
        type L1 = DMCache<1024, 8>;           // Direct-mapped, 1 KiB, 32-B lines
        let c = L1::new();

        let addr = 0x385;
        let tag = c.get_tag(addr);
        let index = c.get_index(addr);
        let word_offset = c.get_word_offset(addr);
        let byte_offset = c.get_byte_offset(addr);

        assert_eq!(tag, 0x0);
        assert_eq!(index, 0x1c);
        assert_eq!(word_offset, 0x1);
        assert_eq!(byte_offset, 0x1);

        assert_eq!(c.stats.total_accesses(), 0);
    }

    #[test]
    fn compulsory_miss () {
        const L1_SIZE: usize = 1 << 12;
        const WORD_P_LINE: usize = 8;
        type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
        let mut c = L1::new();

        let addr = 0x385;

        let result = c.read(addr, DataTypeSize::Byte);
        
        assert!(
            matches!(result, Err(MemoryError::NotFound)),
            "expected Err(NotFound), got: {:?}",
            result
        );

        assert_eq!(c.stats.total_accesses(), 1);
    }

    #[test]
    fn single_write () {
        const L1_SIZE: usize = 1 << 12;
        const WORD_P_LINE: usize = 8;
        type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
        let mut c = L1::new();

        let addr = 0x385;
        c.write_line(addr, 8, vec![0xff; WORDSIZE * 8]);

        let _ = c.write(DataType::Byte(0x11), addr);
        match c.read(addr, DataTypeSize::Byte) {
            Ok(DataType::Byte(d)) => assert_eq!(d, 0x11),
            _ => panic!("Incorrect Read")
        }

        let _ = c.write(DataType::Halfword(0x1234), addr);
        match c.read(addr, DataTypeSize::Halfword) {
            Ok(DataType::Halfword(d)) => assert_eq!(d, 0x1234),
            _ => panic!("Incorrect Read")
        }

        let _ = c.write(DataType::Word(0xcafebabe), addr);
        match c.read(addr, DataTypeSize::Word) {
            Ok(DataType::Word(d)) => assert_eq!(d, 0xcafebabe),
            _ => panic!("Incorrect Read")
        }
        
        let _ = c.write(DataType::DoubleWord(0x87654321cafebabe), addr);
        match c.read(addr, DataTypeSize::DoubleWord) {
            Ok(DataType::DoubleWord(d)) => assert_eq!(d, 0x87654321cafebabe),
            _ => panic!("Incorrect Read")
        }

        assert_eq!(c.stats.total_accesses(), 8);
        assert_eq!(c.stats.hit_rate(), (8 / 8) as f64);
        assert_eq!(c.stats.miss_rate(), (0 / 8) as f64);

    }

    #[test]
    fn read () {
        const L1_SIZE: usize = 1 << 12;
        const WORD_P_LINE: usize = 8;
        type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
        let mut c = L1::new();

        let addr = 0x385;
        c.write_line(addr, 8, vec![0xa5; WORDSIZE * 8]);

        match c.read(addr, DataTypeSize::Byte) {
            Ok(DataType::Byte(d)) => assert_eq!(d, 0xa5),
            _ => panic!("Incorrect Read")
        }

        match c.read(addr, DataTypeSize::Halfword) {
            Ok(DataType::Halfword(d)) => assert_eq!(d, 0xa5a5),
            _ => panic!("Incorrect Read")
        }

        match c.read(addr, DataTypeSize::Word) {
            Ok(DataType::Word(d)) => assert_eq!(d, 0xa5a5a5a5),
            _ => panic!("Incorrect Read")
        }

        match c.read(addr, DataTypeSize::DoubleWord) {
            Ok(DataType::DoubleWord(d)) => assert_eq!(d, 0xa5a5a5a5a5a5a5a5),
            _ => panic!("Incorrect Read")
        }

        assert_eq!(c.stats.total_accesses(), 4);
        assert_eq!(c.stats.hit_rate(), (4 / 4) as f64);
        assert_eq!(c.stats.miss_rate(), (0 / 4) as f64);
    }

    #[test]
    fn write_read_cache_line () {
        const L1_SIZE: usize = 1 << 12;
        const WORD_P_LINE: usize = 8;
        type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
        let mut c = L1::new();

        for i in 0..WORD_P_LINE {
            let i = i * WORDSIZE;
            let _ = c.read(i, DataTypeSize::Word);
        }

        c.stats.print_summary();

        assert_eq!(c.stats.total_accesses(), WORD_P_LINE);
        assert_eq!(c.stats.miss_rate(), 1.0);
    }

    #[test]
    fn write_read_whole_cache () {
        const L1_SIZE: usize = 1 << 12;
        const WORD_P_LINE: usize = 8;
        type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
        let mut c = L1::new();

        let vec: Vec<u8> = (0..L1_SIZE).map(|i| i as u8).collect();

        // change to step
        for i in 0..L1_SIZE {
            if i % (WORD_P_LINE * WORDSIZE) == 0 {
                let slice: Vec<u8> = (i..i+WORD_P_LINE*WORDSIZE)
                    .map(|j| vec[j])
                    .collect();
                c.write_line(i, WORD_P_LINE, slice);
            }
        }

        for i in 0..L1_SIZE {
            match c.read(i, DataTypeSize::Byte) {
                Ok(DataType::Byte(d)) => assert_eq!(d, vec[i]),
                _ => panic!("Incorrect Read")
            }
        }

        c.stats.print_summary();

        assert_eq!(c.stats.total_accesses(), L1_SIZE);
        assert_eq!(c.stats.hit_rate(), 1.0);
        assert_eq!(c.stats.miss_rate(), 0.0);
    }
}