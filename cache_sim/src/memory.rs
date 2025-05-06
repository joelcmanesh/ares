use crate::mem_stats::*;
use crate::main_memory::*;

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
}

pub trait MemoryAccess {
    fn read(&self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError>;
    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError>; 
}


#[derive(Debug)]
pub enum DataTypeSize {
    Byte,
    Halfword,
    Word,
    DoubleWord,
}

#[derive(Debug)]
pub struct Memory {
    size: usize,
    data: Vec<u8>,
    main: MainMemory,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Memory {size, data: vec![0; size], main: MainMemory::new(size) }
    }
}

impl MemoryAccess for  Memory {
    fn read(&self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        self.main.read(addr, size)
    }

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        self.main.write(data, addr)
    }
}