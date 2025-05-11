use crate::mem_stats::*;
use crate::main_memory::*;
use crate::cache::*;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

#[derive(Debug)]
pub enum DataType {
    Byte(u8),
    Halfword(u16),
    Word(u32),
    DoubleWord(u64),
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
    main: MainMemory,
    l1: Cache,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {
            size, 
            main: MainMemory::new(size) , 
            l1: Cache::DirectMapped(DMCache::new(size / 4, 8))
        }
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

        match self.l1.read(addr, size) {
            Ok(data) => {
                Ok(data)
            }

            // refill line
            Err(MemoryError::NotFound) => {
                if self.l1.is_line_dirty(addr) {
                    // write back line
                    let base_addr = 0;
                    let write_back_line = self.l1.get_evict_line(addr);
                    self.main.write_line(base_addr, self.l1.get_words_p_line(), write_back_line);
                }
                // let line = self.main.fetch_line(addr, self.l1.words_per_line)?;  
                // let _ = self.l1.write(data.clone(), addr)?;
                Err(MemoryError::NotCompatible)
            }

            // any other error just propagates
            Err(e) => Err(e),
        }
    }

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        self.main.write(data, addr)
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

