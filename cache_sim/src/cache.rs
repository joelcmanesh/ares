use crate::main_memory::MainMemory;
use crate::memory::*;
use crate::mem_stats::*;

const WORDSIZE: usize = std::mem::size_of::<u32>();


#[derive(Debug)]
pub enum EvictionPolicy {
    LRU,
}

#[derive(Debug)]
pub enum Cache {
    DirectMapped(DMCache),
    // SetAssociative(SetAssocCache),
    // FullyAssociative(FAssocCache),
}

impl Cache {
    pub fn fetch_line(&mut self, addr: usize, main_mem: &mut MainMemory) {
        match self {
            Cache::DirectMapped(dm) => dm.fetch_line(addr, main_mem),
            // Later:
            // Cache::SetAssociative(sa) => sa.fetch_line(addr, main_mem),
            // Cache::FullyAssociative(fa) => fa.fetch_line(addr, main_mem),
        }
    }
}


impl MemoryAccess for Cache {
    fn read(&self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        match self {
            Cache::DirectMapped(dm) => dm.read(addr, size),
            // Cache::SetAssociative(sa) => sa.read(addr, size),
            // Cache::FullyAssociative(fa) => fa.read(addr, size),
        }
    }

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        match self {
            Cache::DirectMapped(dm) => dm.write(data, addr),
            // Cache::SetAssociative(sa) => sa.write(data, addr),
            // Cache::FullyAssociative(fa) => fa.write(data, addr),
        }
    }
}


#[derive(Debug, Clone)]
struct CacheLine {
    valid: bool,
    dirty: bool,
    tag: u32,
    data: Vec<u8>,
}

impl CacheLine {
    fn new(words_per_line: usize) -> Self {
        CacheLine {
            valid: false,
            dirty: false,
            tag: 0,
            data: vec![0; words_per_line * WORDSIZE]
        }
    }
}

#[derive(Debug)]
pub struct DMCache {
    size: usize,
    words_per_line: usize,
    // line_size: usize,
    lines: Vec<CacheLine>,
    stats: MemStats,
    evict: EvictionPolicy,
}

impl DMCache {
    pub fn new(size: usize, words_per_line: usize) -> Self {
        let num_lines = size / (words_per_line * WORDSIZE);
        DMCache {
            size,
            words_per_line,
            lines: (0..num_lines)
                .map(|_| CacheLine::new(words_per_line))
                .collect(),
            stats: MemStats::new(),
            evict: EvictionPolicy::LRU,
        }
    }

    pub fn fetch_line(&mut self, addr: usize, main_mem: &mut MainMemory) {
    }
}

impl MemoryAccess for DMCache {
    fn read(&self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        Err(MemoryError::CacheMiss)
    }

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        Err(MemoryError::CacheMiss)
    }
}
