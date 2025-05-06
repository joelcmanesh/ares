use crate::mem_stats::*;
use crate::main_memory::*;
use crate::cache::*;

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
    CacheMiss,
}

pub trait MemoryAccess {
    fn read(&self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError>;
    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError>; 
}


#[derive(Debug, Clone)]
pub enum DataTypeSize {
    Byte,
    Halfword,
    Word,
    DoubleWord,
}

impl DataTypeSize {
    pub fn get_size(&self,) -> usize {
        match self {
            DataTypeSize::Byte => return std::mem::size_of::<u8>(),
            DataTypeSize::Halfword => return std::mem::size_of::<u16>(),
            DataTypeSize::Word => return std::mem::size_of::<u32>(),
            DataTypeSize::DoubleWord => return std::mem::size_of::<u64>(),
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    size: usize,
    data: Vec<u8>,
    main: MainMemory,
    l1: Cache,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {
            size, 
            data: vec![0; size], 
            main: MainMemory::new(size) , 
            l1: Cache::DirectMapped(DMCache::new(size, 1))
        }
    }
}

impl MemoryAccess for  Memory {
    fn read(&self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        if addr > self.size {
            return Err(MemoryError::OutOfBounds);
        }

        if addr % size.get_size() != 0 {
            return Err(MemoryError::NotAligned);
        }

        match self.l1.read(addr, size.clone()) {
            Ok(v) => Ok(v),
            Err(MemoryError::CacheMiss) => {
                return self.l1.read(addr, size);
            }
            Err(e) => Err(e)
        }
    }

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        self.main.write(data, addr)
    }
}