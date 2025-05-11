use crate::mem_stats::*;
use crate::main_memory::*;
use crate::cache::*;
use crate::direct_mapped_cache::*;

use std::mem;

// const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

#[derive(Debug, Clone)]
pub enum DataType {
    Byte(u8),
    Halfword(u16),
    Word(u32),
    DoubleWord(u64),
}

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
    main: MainMemory,
    l1: Cache,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {
            size, 
            stats: MemStats::new(),
            main: MainMemory::new(size) , 
            l1: Cache::DirectMapped(DMCache::new(size / 4, 8))
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
                let w_p_l = self.l1.get_words_p_line();
                
                self.stats.record_miss();
                if self.l1.is_line_dirty(addr) {
                    // write back line
                    
                    let base_addr = self.l1.get_tag(addr) << (w_p_l + 2);
                    
                    let write_back_line = self.l1.get_evict_line(addr);
                    self.main.write_line(base_addr, w_p_l, write_back_line);
                }

                let new_line = self.main.fetch_line(addr, w_p_l);
                self.l1.write_line(addr, w_p_l, new_line);
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
                
                if self.l1.is_line_dirty(addr) {
                    // write back line
                    let base_addr = self.l1.get_tag(addr) << (w_p_l + 2);
                    
                    let write_back_line = self.l1.get_evict_line(addr);
                    self.main.write_line(base_addr, w_p_l, write_back_line);
                }

                let new_line = self.main.fetch_line(addr, w_p_l);
                self.l1.write_line(addr, w_p_l, new_line);
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
        let _ = Memory::new(1 << 12);
    }

    // #[test]
    // fn write_and_read_back_byte() {
    //     let mut c = Cache::new(4);
    //     c.write(0x10, 0xAB);
    //     assert_eq!(c.read(0x10), Some(0xAB));
    // }
}

