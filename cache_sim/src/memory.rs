use crate::mem_stats::*;
use crate::main_memory::*;
use crate::cache::*;
use crate::direct_mapped_cache::*;
use crate::set_associative::*;

use std::mem;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Byte(u8),
    Halfword(u16),
    Word(u32),
    DoubleWord(u64),
}

impl From<u8>  for DataType { fn from(v: u8)  -> Self { DataType::Byte(v) } }
impl From<u16> for DataType { fn from(v: u16) -> Self { DataType::Halfword(v) } }
impl From<u32> for DataType { fn from(v: u32) -> Self { DataType::Word(v) } }
impl From<u64> for DataType { fn from(v: u64) -> Self { DataType::DoubleWord(v) } }

impl DataType {
    pub fn payload_size(&self) -> usize {
        match self {
            DataType::Byte(_)       => mem::size_of::<u8>(),
            DataType::Halfword(_)   => mem::size_of::<u16>(),
            DataType::Word(_)       => mem::size_of::<u32>(),
            DataType::DoubleWord(_) => mem::size_of::<u64>(),
        }
    }
}

#[derive(Debug)]
pub enum MemoryError {
    OutOfBounds,
    NotAligned,
    NotFound,
    NotCompatible, 
}

pub trait MemoryAccess {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError>;
    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError>; 
}

pub trait MemLevelAccess {
    fn write_line(&mut self, addr: usize, words_per_lines: usize, data: Vec<u8>);
    fn fetch_line(&self, addr: usize, words_per_lines: usize) -> Vec<u8>;
}

#[warn(dead_code)]
#[derive(Debug, Clone)]
pub enum DataTypeSize {
    Byte,
    Halfword,
    Word,
    DoubleWord,
}

impl DataTypeSize {
    pub const fn size(self) -> usize {
        match self {
            DataTypeSize::Byte       => std::mem::size_of::<u8>(),
            DataTypeSize::Halfword   => std::mem::size_of::<u16>(),
            DataTypeSize::Word       => std::mem::size_of::<u32>(),
            DataTypeSize::DoubleWord => std::mem::size_of::<u64>(),
        }
    }

    pub const fn get_size(size: DataTypeSize) -> usize {
        size.size()
    }
}

#[derive(Debug)]
pub struct Memory {
    size: usize,
    stats: MemStats,
    pub l1: Cache,
    main: MainMemory,
}

impl Memory {
    pub fn new(size: usize, l1_size: usize, word_per_line: usize) -> Self {
        assert!(l1_size <= size);
        Memory {
            size, 
            stats: MemStats::new(),
            main: MainMemory::new(size) , 
            l1: Cache::DirectMapped(DMCache::new(l1_size, word_per_line))
        }
    }

    pub fn print_summary(&self) {
        println!("Memory");
        self.stats.print_summary();

        println!("L1");
        self.l1.print_summary();

        // println!("Main");
        // self.stats.print_summary();

    }
}

impl MemoryAccess for Memory {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        if addr >= self.size {
            return Err(MemoryError::OutOfBounds);
        }

        let align = DataTypeSize::get_size(size.clone());
        if addr % align != 0 {
            return Err(MemoryError::NotAligned);
        }

        match self.l1.read(addr, size.clone()) {
            Ok(data) => {
                self.stats.record_hit();
                Ok(data)
            }
            
            Err(MemoryError::NotFound) => {
                self.stats.record_miss();

                let w_p_l = self.l1.get_words_p_line();

                let fetch_base_addr = self.l1.get_base_addr(addr);
                
                if self.l1.is_line_dirty(addr) {
                    let write_back_addr = self.l1.get_writeback_addr(addr);
                    let write_back_line = self.l1.get_evict_line(addr);
                    self.main.write_line(write_back_addr, w_p_l, write_back_line);
                }

                let new_line = self.main.fetch_line(fetch_base_addr, w_p_l);
                self.l1.write_line(fetch_base_addr, w_p_l, new_line);
                self.l1.read(addr, size)
            }

            Err(e) => Err(e),
        }
    }

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        if addr >= self.size {
            return Err(MemoryError::OutOfBounds);
        }

        let align = data.payload_size();
        if addr % align != 0 {
            return Err(MemoryError::NotAligned);
        }

        match self.l1.write(data.clone(), addr) {
            Ok(()) => {
                self.stats.record_hit();
                Ok(())
            }

            Err(MemoryError::NotFound) => {
                let w_p_l = self.l1.get_words_p_line();
                self.stats.record_miss();

                let fetch_base_addr = self.l1.get_base_addr(addr);
                
                if self.l1.is_line_dirty(addr) {
                    let write_back_addr = self.l1.get_writeback_addr(addr);
                    let write_back_line = self.l1.get_evict_line(addr);
                    self.main.write_line(write_back_addr, w_p_l, write_back_line);
                }

                let new_line = self.main.fetch_line(fetch_base_addr, w_p_l);
                self.l1.write_line(fetch_base_addr, w_p_l, new_line);
                self.l1.write(data, addr)
            }

            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty_cache_has_no_data() {
        let _ = Memory::new(1 << 12, 1<<10, 4);
    }

    #[test]
    fn write_and_read_back_byte() {
        let tot_size = 1 << 16 as usize;
        let l1_size = 1 << 10 as usize;
        let w_p_l = 4 as usize;

        let mut m = Memory::new(tot_size, l1_size, w_p_l);
        
        let addr = 0x10;
        let byte = DataType::Byte(0xff);

        let _ = m.write(byte.clone(), addr);

        let mut dut_byte = DataType::Byte(0);
        match m.read(addr, DataTypeSize::Byte) {
            Ok(b) => dut_byte = b,  
            Err(MemoryError::NotFound) => panic!("mem error"),
            _ => panic!("idk")
        }
        assert_eq!(dut_byte, byte);

        // TODO: check cache stats
        assert_eq!(m.stats.total_accesses(), 2);
        assert_eq!(m.stats.hit_rate(), 0.5);
        assert_eq!(m.stats.miss_rate(), 0.5);
    }

    #[test]
    fn write_read_cache_line() {
        let tot_size = 1 << 16 as usize;
        let l1_size = 1 << 10 as usize;
        let w_p_l = 8 as usize;

        let mut m = Memory::new(tot_size, l1_size, w_p_l);

        // write line to main mem and check values
        let expected_data: Vec<u32> = (0..w_p_l).map(|i| i as u32).collect();
        for i in 0..w_p_l {
            let addr = i * WORDSIZE;
            let _ = m.main.write(DataType::Word(expected_data[i]), addr);
        }

        for i in 0..w_p_l {
            let addr = i * WORDSIZE;
            let _ = m.main.read(addr, DataTypeSize::Word);
        }

        for i in 0..w_p_l {
            let addr = i * WORDSIZE;
            match m.read(addr, DataTypeSize::Word) {
                Ok(DataType::Word(w)) => assert_eq!(w, expected_data[i]),
                _ => panic!("Incorrect read @ {:#?}",addr)
            }
        }

        m.print_summary();

        assert_eq!(m.stats.total_accesses(), 8);
        assert_eq!(m.stats.hit_rate(), 7 as f64 / 8 as f64);
        assert_eq!(m.stats.miss_rate(), 1 as f64 / 8 as f64);
    }

    #[test]
    fn write_read_2cache_line() {
        let tot_size = 1 << 16 as usize;
        let l1_size = 1 << 10 as usize;
        let w_p_l = 8 as usize;

        let mut m = Memory::new(tot_size, l1_size, w_p_l);

        for i in 0..w_p_l+1 {
            let addr = i * WORDSIZE;
            let _ = m.read(addr, DataTypeSize::Word);
        }

        m.print_summary();

        let expected_hit = 7 as f64 / m.stats.total_accesses() as f64;
        let expected_miss =2 as f64 / m.stats.total_accesses() as f64;
        
        assert_eq!(m.stats.total_accesses(), 9);
        assert_eq!(m.stats.hit_rate(), expected_hit as f64);
        assert_eq!(m.stats.miss_rate(), expected_miss as f64);
    }

    #[test]
    fn write_read_whole_cache() {
        let tot_size = 1 << 16 as usize;
        let l1_size = 1 << 10 as usize;
        let w_p_l = 8 as usize;

        let mut m = Memory::new(tot_size, l1_size, w_p_l);

        for i in 0..l1_size/WORDSIZE {
            let addr = i * WORDSIZE;
            let _ = m.read(addr, DataTypeSize::Word);
        }

        m.print_summary();

        let hit_ratio = (w_p_l - 1) as f64 / w_p_l as f64;
        let miss_ratio = 1 as f64 / w_p_l as f64;
        
        assert_eq!(m.stats.total_accesses(), l1_size / WORDSIZE);
        assert_eq!(m.stats.hit_rate(), hit_ratio as f64);
        assert_eq!(m.stats.miss_rate(), miss_ratio as f64);
    }

    #[test]
    fn cache_eviction() {
        let tot_size = 1 << 16 as usize;
        let l1_size = 1 << 10 as usize;
        let w_p_l = 8 as usize;

        let mut m = Memory::new(tot_size, l1_size, w_p_l);

        // bits 
        // 2 - byte sel 
        // 3 - word sel
        // 7 - index
        // 4 - tag

        // cause a miss and write to the cache
        let addr1 = (1 << 12) | (0x8 << 5) | (0x4 << 2) | (0x0);
        println!("addr1 {:x} = {}", addr1, addr1);
        let data1 = DataType::Word(0xcafebabe);
        let _ = m.write(data1.clone(), addr1.clone());

        // get mapped to the same index and evict the old line
        let addr2 = (2 << 12) | (0x8 << 5) | (0x4 << 2) | (0x0);
        match m.read(addr2, DataTypeSize::Word) {
            Ok(w) => assert_ne!(w, data1),
            _=> panic!("[MEMORY] errror here")
        }

        // bring line back in and check that it wrote back
        match m.read(addr1, DataTypeSize::Word) {
            Ok(w) => assert_eq!(w, data1, "[MEMORY] write-back or reload failed"),
            Err(e) => panic!("[MEMORY] read error: {e:?}"),
        }

        m.print_summary();

        assert_eq!(m.stats.total_accesses(), 3);
        assert_eq!(m.stats.hit_rate(), 0.0);
        assert_eq!(m.stats.miss_rate(), 1.0);
    }

    #[test]
    fn write_read_whole_mem() {
    }

    // #[test]
    // fn name() {
    // }

    // #[test]
    // fn name() {
    // }

    // #[test]
    // fn name() {
    // }

    // #[test]
    // fn name() {
    // }

}

