use crate::mem_stats::*;
use crate::memory::*;
use crate::cache::*;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);
pub const ADDR_BITS: usize = 32;

#[derive(Debug)]
pub struct DMCache {
    words_per_line: usize,
    pub stats: MemStats,
    lines: Vec<CacheLine>,
    index_mask: usize,
    word_mask: usize,
    index_shift: usize,
    word_shift: usize,
    tag_shift: usize,
}

impl DMCache {
    pub fn new(size: usize, words_per_line: usize) -> Self {
        assert!(size <= 1 << ADDR_BITS, "MAX ADDR is 0xFFFF_FFFF");
        
        let num_lines = (size / WORDSIZE) / words_per_line;
        assert!(
            num_lines.is_power_of_two() && words_per_line.is_power_of_two(),
            "cache_size/(words_per_line*WORDSIZE) and words_per_line must both be powers of two"
        );
        let byte_bits = WORDSIZE.trailing_zeros() as usize;
        let word_bits = words_per_line.trailing_zeros() as usize;
        let index_bits = num_lines.trailing_zeros() as usize;
        
        let offset_bits = byte_bits + word_bits;

        let word_mask = (1 << offset_bits) - 1;
        let word_shift = byte_bits;

        let index_mask = (1 << index_bits + offset_bits) - 1;
        let index_shift = offset_bits;

        let tag_shift = offset_bits + index_bits;

        println!("Tag shift {:x}, index shift {:x}, word shift {:x}", 
            tag_shift, index_shift, word_shift);

        DMCache {
            words_per_line,
            lines: (0..num_lines)
                .map(|_| CacheLine::new(words_per_line))
                .collect(),
            stats: MemStats::new(),
            index_mask,
            index_shift,
            word_mask, 
            word_shift,
            tag_shift,
        }
    }

    pub fn print_summary(&self) {
        self.stats.print_summary();
    }

    pub fn get_lines(&self) -> usize {
        self.lines.len()
    }
}

impl MemoryAccess for DMCache {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        let ind = self.get_index(addr);
        
        let line: &CacheLine = &self.lines[ind];

        if !line.is_valid() || line.tag() != self.get_tag(addr) {
            self.stats.record_miss();
            return Err(MemoryError::NotFound);
        }
        
        self.stats.record_hit();
        let byte_index = WORDSIZE * self.get_word_offset(addr) + self.get_byte_offset(addr);
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

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        let idx = self.get_index(addr);
        let byte_index = WORDSIZE * self.get_word_offset(addr) + self.get_byte_offset(addr);

        let line: &CacheLine = &self.lines[idx];
        if !line.is_valid() || line.tag() != self.get_tag(addr) {
            self.stats.record_miss();
            return Err(MemoryError::NotFound);
        }
        
        let line: &mut CacheLine = &mut self.lines[idx];
        self.stats.record_hit();
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

impl MemLevelAccess for DMCache {
    fn write_line(&mut self, addr: usize, _words_per_lines: usize, data: Vec<u8>) {
        let ind = self.get_index(addr);
        let tag = self.get_tag(addr);
        let line: &mut CacheLine = &mut self.lines[ind];
        line.write_line(tag, data);
    }

    fn fetch_line(&self, addr: usize, _words_per_lines: usize) -> Vec<u8> {
        self.get_evict_line(addr)
    }
}

impl CacheAddressing for DMCache {
    fn get_tag(&self, addr: usize) -> usize {
        addr >> self.tag_shift
    }

    fn get_index(&self, addr: usize) -> usize {
        (addr & self.index_mask) >> self.index_shift
    }

    fn get_word_offset(&self, addr: usize) -> usize {
        (addr & self.word_mask) >> self.word_shift
    }

    fn get_byte_offset(&self, addr: usize) -> usize {
        addr & 0x3
    }

    fn is_line_dirty(&self, addr: usize) -> bool {
        let ind = self.get_index(addr);
        self.lines[ind].is_dirty()
    }

    fn get_evict_line(&self, addr:usize) -> Vec<u8> {
        let ind = self.get_index(addr);
        let line = self.lines[ind].clone();
        line.get_data()
    }

    fn get_words_p_line(&self) -> usize {
        self.words_per_line
    }

    fn get_tag_shift(&self) -> usize {
        self.tag_shift
    }

    fn get_writeback_addr(&self, addr: usize) -> usize {
        let ind = self.get_index(addr);
        let tag = self.lines[ind].tag();
        (tag << self.tag_shift) + (ind << self.index_shift) 
    }

    fn get_base_addr(&self, addr: usize) -> usize {
        (self.get_tag(addr) << self.tag_shift) + (self.get_index(addr) << self.index_shift)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new () {
        let _ = DMCache::new(1 << 10, 8);
    }

    #[test]
    fn parse_addr() {
        let c = DMCache::new(1 << 12, 8);

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
        let mut c = DMCache::new(1 << 12, 8);

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
        let mut c = DMCache::new(1 << 12, 8);

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
        let mut c = DMCache::new(1 << 12, 8);

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
        let size: usize = 1 << 16;
        let words_p_line = 8;

        let mut c = DMCache::new(size, words_p_line);
        
        for i in 0..words_p_line {
            let i = i * WORDSIZE;
            let _ = c.read(i, DataTypeSize::Word);
        }

        c.stats.print_summary();

        assert_eq!(c.stats.total_accesses(), words_p_line);
        assert_eq!(c.stats.miss_rate(), 1.0);
    }

    #[test]
    fn write_read_whole_cache () {
        let size: usize = 1 << 6;
        let words_p_line = 4;
        let vec: Vec<u8> = (0..size).map(|i| i as u8).collect();

        let mut c = DMCache::new(size, words_p_line);

            for i in 0..vec.len() {
            if i % (words_p_line * WORDSIZE) == 0 {
                let slice: Vec<u8> = (i..i+words_p_line*WORDSIZE).map(|j| vec[j]).collect();
                c.write_line(i, words_p_line, slice);
            }
        }

        for i in 0..vec.len() {
            match c.read(i, DataTypeSize::Byte) {
                Ok(DataType::Byte(d)) => assert_eq!(d, vec[i]),
                _ => panic!("Incorrect Read")
            }
        }

        c.stats.print_summary();

        assert_eq!(c.stats.total_accesses(), size);
        assert_eq!(c.stats.hit_rate(), 1.0);
        assert_eq!(c.stats.miss_rate(), 0.0);
    }
}