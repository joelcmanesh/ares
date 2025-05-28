use std::vec;

use rand::rng;
use rand::Rng;

use crate::mem_stats::*;
use crate::memory::*;
use crate::cache::*;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);
pub const ADDR_BITS: usize = 32;

#[derive(Debug)]
pub struct SetAssocCache<
    const BYTES: usize,               
    const WORDS_PER_LINE: usize,      
    const ASSOC: usize,      
    > {
    sets: Vec<Vec<CacheLine>>,
    eviction: EvictionPolicy,
    stats: MemStats,
}

impl<const BYTES: usize, const WORDS_PER_LINE: usize, const ASSOC: usize> 
    SetAssocCache<BYTES, WORDS_PER_LINE, ASSOC> {
    

    pub const NUM_LINES: usize = BYTES / (ASSOC * WORDSIZE * WORDS_PER_LINE);

    pub fn new(eviction_pol: EvictionPolicy) -> Self {
        assert!(BYTES.is_power_of_two(), "BYTES must be a power of two");
        assert!(WORDS_PER_LINE.is_power_of_two(), "WORDS_PER_LINE must be a power of two");
        assert!(Self::NUM_LINES > 0, "cache must hold ≥ 1 line");
        assert!(Self::NUM_LINES.is_power_of_two(),"NUM_LINES must be a power of two");

        let sets: Vec<Vec<CacheLine>> = vec![vec![CacheLine::new(WORDS_PER_LINE); Self::NUM_LINES]; ASSOC];

        Self {
            sets,
            stats: MemStats::new(),
            eviction: eviction_pol,
        }
    }

    // fn get_line(&self, addr: usize) -> Option<&CacheLine> {
    //     let (tag, ind, _, _) = self.decode_addr(addr);
    //     for i in 0..ASSOC {
    //         let line: &CacheLine = &self.sets[i][ind];
    //         if line.tag() == tag {
    //             return Some(line);
    //         }
    //     }
    //     None
    // }

    fn find_line(&self, addr: usize) -> Option<(usize, usize)> {
        let (tag, ind, _, _) = self.decode_addr(addr);
        for way in 0..ASSOC {
            let line = &self.sets[way][ind];
            if line.is_valid() && line.tag() == tag { 
                return Some((way, ind));
            }
        }
        None
    }
    
}

impl<const B: usize, const W: usize, const A: usize> MemoryAccess for SetAssocCache<B, W, A> {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        let (way, ind) = match self.find_line(addr) {
            Some(pos) => {
                self.stats.record_hit();
                pos
            }
            None => {
                self.stats.record_miss();
                return Err(MemoryError::NotFound);
            }
        };
    
        let (_, _, word_off, byte_off) = self.decode_addr(addr);
        let line = &mut self.sets[way][ind];
        let byte_index = word_off * WORDSIZE + byte_off;
        line.stamp_now();
        
        let data = match size {
            DataTypeSize::Byte => {
                DataType::Byte(line.read_byte(byte_index))
            }

            DataTypeSize::Halfword => {
                let bytes = [
                    line.read_byte(byte_index),
                    line.read_byte(byte_index + 1),
                ];
                DataType::Halfword(u16::from_le_bytes(bytes))
            }

            DataTypeSize::Word => {
                let bytes = [
                    line.read_byte(byte_index),
                    line.read_byte(byte_index + 1),
                    line.read_byte(byte_index + 2),
                    line.read_byte(byte_index + 3),
                ];
                DataType::Word(u32::from_le_bytes(bytes))
            }

            DataTypeSize::DoubleWord => {
                let mut bytes = [0u8; 8];
                for i in 0..8 {
                    bytes[i] = line.read_byte(byte_index + i);
                }
                DataType::DoubleWord(u64::from_le_bytes(bytes))
            }
        };

        Ok(data)
    }

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        let (way, ind) = match self.find_line(addr) {
            Some(pos) => {
                self.stats.record_hit();
                pos
            }
            None => {
                self.stats.record_miss();
                return Err(MemoryError::NotFound);
            }
        };
    
        let (_, _, word_off, byte_off) = self.decode_addr(addr);
        let line = &mut self.sets[way][ind];
        let byte_index = word_off * WORDSIZE + byte_off;
        line.stamp_now();

        match data {
            DataType::Byte(val) => {
                line.write_byte(byte_index, val);
                Ok(())
            }

            DataType::Halfword(val) => {
                let bytes = val.to_le_bytes();
                for (i, _) in bytes.iter().enumerate() {
                    line.write_byte(byte_index + i, bytes[i]);
                }
                Ok(())
            }


            DataType::Word(val) => {

                let bytes = val.to_le_bytes();
                for (i, _) in bytes.iter().enumerate() {
                    line.write_byte(byte_index + i, bytes[i]);
                }
                Ok(())
            }

            DataType::DoubleWord(val) => {
                let bytes = val.to_le_bytes();
                for (i, _) in bytes.iter().enumerate() {
                    line.write_byte(byte_index + i, bytes[i]);
                }
                Ok(())
            }
        }
    }

    fn stats(&self) -> &MemStats {
        &self.stats
    }
}

impl<const B: usize, const W: usize, const A: usize> MemLevelAccess for SetAssocCache<B, W, A> {
    fn write_line(&mut self, addr: usize, _words_per_lines: usize, data: Vec<u8>) {
        // check sets for invalid lines
        // if no free sets use eviction policy to throw out a set
        // should update 
        let (tag, ind, _, _) = self.decode_addr(addr);
        
        if let Some(line) = self.sets
                                .iter_mut()
                                .map(|way| &mut way[ind])
                                .find(|l| !l.is_valid()) 
        {
            line.write_line(tag, data);
            return;
        }
        
        let way = match self.eviction {
            EvictionPolicy::Random => {
                rng().random_range(0..A)
            }
            EvictionPolicy::Lru => {
                (0..A)
                    .min_by_key(|&w| self.sets[w][ind].time())
                    .unwrap()
            }
            _ => 0
        };
        
        let line = &mut self.sets[way][ind];

        line.write_line(tag, data);
    }

    fn fetch_line(&self, addr: usize, words_per_lines: usize) -> Vec<u8> {
        vec![0; 1]
    }
}

impl<const B: usize, const W: usize, const A: usize> CacheAddressing for SetAssocCache<B, W, A> {
    #[inline(always)]
    fn byte_bits(&self) -> usize {
        WORDSIZE.trailing_zeros() as usize
    }
    
    #[inline(always)]
    fn word_bits(&self) -> usize {
        W.trailing_zeros() as usize
    }

    #[inline(always)]
    fn index_bits(&self) -> usize {
        self.sets[0].len().trailing_zeros() as usize
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

    // TODO:
    #[inline(always)]
    fn is_line_dirty(&self, addr: usize) -> bool {
        match self.find_line(addr) {
            Some(pos) => {
                let line = &self.sets[pos.0][pos.1]; 
                line.is_dirty()
            }
            None => {
                false
            }
        };
        false
    }

    // TODO
    #[inline(always)]
    fn get_evict_line_data(&self, addr:usize) -> Vec<u8> {
        match self.eviction {
            EvictionPolicy::Lru => vec![],
            EvictionPolicy::Nru => vec![],
            EvictionPolicy::Random => vec![],
        }
    }
    #[inline(always)]
    fn get_writeback_addr(&self, addr: usize) -> usize {
        0
    }

    #[inline(always)]
    fn get_base_addr(&self, addr: usize) -> usize {
        0
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new () {
        type L1 = SetAssocCache<1024, 8, 2>;
        let l1 = L1::new(EvictionPolicy::Random);
        println!("{:#?}", l1);
    }

    #[test]
    fn parse_addr() {
        type L1 = SetAssocCache<4096, 8, 2>;
        let c = L1::new(EvictionPolicy::Random);

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
        const L1_SIZE: usize = 1 << 13;
        const WORD_P_LINE: usize = 8;
        const ASSOC: usize = 2;
        type L1 = SetAssocCache<L1_SIZE, WORD_P_LINE, ASSOC>;
        let mut c = L1::new(EvictionPolicy::Random);

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
        const L1_SIZE: usize = 1 << 13;
        const WORD_P_LINE: usize = 8;
        const ASSOC: usize = 2;
        type L1 = SetAssocCache<L1_SIZE, WORD_P_LINE, ASSOC>;
        let mut c = L1::new(EvictionPolicy::Random);

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
    fn write_mult_ways() {
        const L1_SIZE: usize = 1 << 13;
        const WORD_P_LINE: usize = 8;
        const ASSOC: usize = 2;
        type L1 = SetAssocCache<L1_SIZE, WORD_P_LINE, ASSOC>;
        let mut c = L1::new(EvictionPolicy::Random);

        let bb = c.byte_bits(); // lowest bits
        let wb = c.word_bits(); // next bits
        let ib = c.index_bits(); // next bits

        let addr1 = (1 << (ib + wb + bb)) | (0x8 << ib) | 
                            (0x2 << wb) | (0x0 << bb);
        let data1 = DataType::Word(0xcafebabe);
        let _ = c.write(data1.clone(), addr1.clone());

        let addr2 = (2 << (ib + wb + bb)) | (0x8 << ib) | 
                            (0x2 << wb) | (0x0 << bb);
        let data2 = DataType::Word(0xbabecafe);
        let _ = c.write(data2.clone(), addr2.clone());

        match c.read(addr1, DataTypeSize::Word) {
            Ok(w) => assert_eq!(w, data1),
            _=> panic!("[MEMORY] errror here")
        }

        match c.read(addr2, DataTypeSize::Word) {
            Ok(w) => assert_eq!(w, data2, "[MEMORY] write-back or reload failed"),
            Err(e) => panic!("[MEMORY] read error: {e:?}"),
        }

        assert_eq!(c.stats.total_accesses(), 4);
    }

    // #[test]
    // fn read () {
    //     const L1_SIZE: usize = 1 << 12;
    //     const WORD_P_LINE: usize = 8;
    //     type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
    //     let mut c = L1::new();

    //     let addr = 0x385;
    //     c.write_line(addr, 8, vec![0xa5; WORDSIZE * 8]);

    //     match c.read(addr, DataTypeSize::Byte) {
    //         Ok(DataType::Byte(d)) => assert_eq!(d, 0xa5),
    //         _ => panic!("Incorrect Read")
    //     }

    //     match c.read(addr, DataTypeSize::Halfword) {
    //         Ok(DataType::Halfword(d)) => assert_eq!(d, 0xa5a5),
    //         _ => panic!("Incorrect Read")
    //     }

    //     match c.read(addr, DataTypeSize::Word) {
    //         Ok(DataType::Word(d)) => assert_eq!(d, 0xa5a5a5a5),
    //         _ => panic!("Incorrect Read")
    //     }

    //     match c.read(addr, DataTypeSize::DoubleWord) {
    //         Ok(DataType::DoubleWord(d)) => assert_eq!(d, 0xa5a5a5a5a5a5a5a5),
    //         _ => panic!("Incorrect Read")
    //     }

    //     assert_eq!(c.stats.total_accesses(), 4);
    //     assert_eq!(c.stats.hit_rate(), (4 / 4) as f64);
    //     assert_eq!(c.stats.miss_rate(), (0 / 4) as f64);
    // }

    // #[test]
    // fn write_read_cache_line () {
    //     const L1_SIZE: usize = 1 << 12;
    //     const WORD_P_LINE: usize = 8;
    //     type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
    //     let mut c = L1::new();

    //     for i in 0..WORD_P_LINE {
    //         let i = i * WORDSIZE;
    //         let _ = c.read(i, DataTypeSize::Word);
    //     }

    //     c.stats.print_summary();

    //     assert_eq!(c.stats.total_accesses(), WORD_P_LINE);
    //     assert_eq!(c.stats.miss_rate(), 1.0);
    // }

    // #[test]
    // fn write_read_whole_cache () {
    //     const L1_SIZE: usize = 1 << 12;
    //     const WORD_P_LINE: usize = 8;
    //     type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
    //     let mut c = L1::new();

    //     let vec: Vec<u8> = (0..L1_SIZE).map(|i| i as u8).collect();

    //     // change to step
    //     for i in 0..L1_SIZE {
    //         if i % (WORD_P_LINE * WORDSIZE) == 0 {
    //             let slice: Vec<u8> = (i..i+WORD_P_LINE*WORDSIZE)
    //                 .map(|j| vec[j])
    //                 .collect();
    //             c.write_line(i, WORD_P_LINE, slice);
    //         }
    //     }

    //     for i in 0..L1_SIZE {
    //         match c.read(i, DataTypeSize::Byte) {
    //             Ok(DataType::Byte(d)) => assert_eq!(d, vec[i]),
    //             _ => panic!("Incorrect Read")
    //         }
    //     }

    //     c.stats.print_summary();

    //     assert_eq!(c.stats.total_accesses(), L1_SIZE);
    //     assert_eq!(c.stats.hit_rate(), 1.0);
    //     assert_eq!(c.stats.miss_rate(), 0.0);
    // }
}