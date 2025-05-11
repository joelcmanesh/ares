use crate::memory::*;
use crate::direct_mapped_cache::*;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

#[warn(dead_code)]
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
    pub fn print_summary(&self) {
        match self {
            Cache::DirectMapped(dm) => dm.print_summary(),
            // Cache::SetAssociative(sa) => sa.print_summary(),
            // Cache::FullyAssociative(fa) => fa.print_summary(),
        }
    }
}

pub trait CacheAddressing {
    fn get_tag(&self, addr: usize) -> usize;
    fn get_index(&self, addr: usize) -> usize;
    fn get_word_offset(&self, addr: usize) -> usize;
    fn get_byte_offset(&self, addr: usize) -> usize;
    fn is_line_dirty(&self, addr: usize) -> bool;
    fn get_evict_line(&self, addr:usize) -> Vec<u8>;
    fn get_words_p_line(&self) -> usize;
}

impl MemoryAccess for Cache {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
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

impl CacheAddressing for Cache {
    fn get_tag(&self, addr: usize) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.get_tag(addr),
            // Cache::SetAssociative(sa)    => sa.get_tag(addr),
            // Cache::FullyAssociative(fa)  => fa.get_tag(addr),
        }
    }

    fn get_index(&self, addr: usize) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.get_index(addr),
            // Cache::SetAssociative(sa)    => sa.get_index(addr),
            // Cache::FullyAssociative(fa)  => fa.get_index(addr),
        }
    }

    fn get_word_offset(&self, addr: usize) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.get_word_offset(addr),
            // Cache::SetAssociative(sa)    => sa.get_word_offset(addr),
            // Cache::FullyAssociative(fa)  => fa.get_word_offset(addr),
        }
    }

    fn get_byte_offset(&self, addr: usize) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.get_byte_offset(addr),
            // Cache::SetAssociative(sa)    => sa.get_byte_offset(addr),
            // Cache::FullyAssociative(fa)  => fa.get_byte_offset(addr),
        }
    }

    fn is_line_dirty(&self, addr: usize) -> bool {
        match self {
            Cache::DirectMapped(dm) => dm.is_line_dirty(addr),
            // Cache::SetAssociative(sa)    => sa.is_line_dirty(addr),
            // Cache::FullyAssociative(fa)  => fa.is_line_dirty(addr),
        }
    }

    fn get_evict_line(&self, addr: usize) -> Vec<u8> {
        match self {
            Cache::DirectMapped(dm) => dm.get_evict_line(addr),
            // Cache::SetAssociative(sa)    => sa.get_evict_line(addr),
            // Cache::FullyAssociative(fa)  => fa.get_evict_line(addr),
        }
    }

    fn get_words_p_line(&self) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.get_words_p_line(),
            // Cache::SetAssociative(sa)    => sa.get_words_p_line(),
            // Cache::FullyAssociative(fa)  => fa.get_words_p_line(),
        }
    }
}

impl MemLevelAccess for Cache {
    fn write_line(&mut self, addr: usize, words_per_lines: usize, data: Vec<u8>) {
        match self {
            Cache::DirectMapped(dm) => dm.write_line(addr, words_per_lines, data),
            // Cache::SetAssociative(sa) => sa.write_line(addr, size),
            // Cache::FullyAssociative(fa) => fa.write_line(addr, size),
        }
    }

    fn fetch_line(&self, addr: usize, words_per_lines: usize) -> Vec<u8> {
        match self {
            Cache::DirectMapped(dm) => dm.fetch_line(addr, words_per_lines),
            // Cache::SetAssociative(sa) => sa.fetch_line(addr, size),
            // Cache::FullyAssociative(fa) => fa.fetch_line(addr, size),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheLine {
    valid: bool,
    dirty: bool,
    tag: usize,
    data: Vec<u8>,
}

impl CacheLine {
    pub fn new(words_per_line: usize) -> Self {
        CacheLine {
            valid: false,
            dirty: false,
            tag: 0,
            data: vec![0; words_per_line * WORDSIZE]
        }
    }

    // getters
    pub fn is_valid(&self) -> bool { self.valid }
    pub fn is_dirty(&self) -> bool { self.dirty }
    pub fn tag(&self) -> usize     { self.tag }
    pub fn get_data(&self) -> Vec<u8> { self.data.clone()}


    pub fn read_byte(&self, offset: usize) -> u8 {
        self.data[offset]
    }

    pub fn write_byte(&mut self, offset: usize, value: u8) {
        self.data[offset] = value;
        self.dirty = true;
    }

    pub fn write_line(&mut self, tag: usize, new_data: Vec<u8>) {
        self.tag = tag;
        self.data = new_data.clone();
        self.valid = true;
        self.dirty = false;
    }
}