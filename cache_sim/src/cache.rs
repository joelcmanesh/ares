use crate::memory::*;
use crate::mem_stats::*;
use crate::direct_map::*;
// use crate::set_associative::*;

use std::time::{SystemTime, UNIX_EPOCH};

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

#[derive(Debug)]
pub enum EvictionPolicy {
    Lru,
    Nru,
    Random
}

#[derive(Debug)]
pub enum Cache<
    const BYTES: usize,
    const WORDS_PER_LINE: usize,
    const ASSOC: usize = 1,        // only SA/FA care
> {
    DirectMapped(DMCache<BYTES, WORDS_PER_LINE>),
    // SetAssociative(SetAssocCache<BYTES, WORDS_PER_LINE, ASSOC>),
    // FullyAssociative(FAssocCache<BYTES, WORDS_PER_LINE>),
}

pub trait CacheAddressing {
    fn is_line_dirty(&self, addr: usize) -> bool;
    fn get_base_addr(&self, addr: usize) -> usize;
    fn get_writeback_addr(&self, addr: usize) -> usize;
    fn decode_addr(&self, addr: usize) -> (usize, usize, usize, usize); // tag, index, word, byte
    
    fn get_tag(&self, addr: usize) -> usize;
    fn get_index(&self, addr: usize) -> usize;
    fn get_word_offset(&self, addr: usize) -> usize;
    fn get_byte_offset(&self, addr: usize) -> usize;
    fn get_evict_line_data(&self, addr:usize) -> Vec<u8>;

    fn byte_bits(&self) -> usize;
    fn word_bits(&self) -> usize;
    fn index_bits(&self) -> usize;
}

impl<const B: usize, const W: usize, const A: usize> MemoryAccess for Cache<B, W, A> {
    fn read(&mut self, addr: usize, size: DataTypeSize, dont_count: bool) -> Result<DataType, MemoryError> {
        match self {
            Cache::DirectMapped(dm) => dm.read(addr, size, dont_count),
            // Cache::SetAssociative(sa) => sa.read(addr, size),
            // Cache::FullyAssociative(fa) => fa.read(addr, size),
        }
    }

    fn write(&mut self, data: DataType, addr: usize, dont_count: bool) -> Result<(), MemoryError> {
        match self {
            Cache::DirectMapped(dm) => dm.write(data, addr, dont_count),
            // Cache::SetAssociative(sa) => sa.write(data, addr),
            // Cache::FullyAssociative(fa) => fa.write(data, addr),
        }
    }

    fn stats(&self) -> &MemStats {
        match self {
            Cache::DirectMapped(dm) => dm.stats(),
            // Cache::SetAssociative(sa) => sa.stats(),
            // Cache::FullyAssociative(fa) => fa.stats(),
        }
    }
}

impl<const B: usize, const W: usize, const A: usize> CacheAddressing for Cache<B, W, A> {
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

    fn get_evict_line_data(&self, addr: usize) -> Vec<u8> {
        match self {
            Cache::DirectMapped(dm) => dm.get_evict_line_data(addr),
            // Cache::SetAssociative(sa)    => sa.get_evict_line_data(addr),
            // Cache::FullyAssociative(fa)  => fa.get_evict_line_data(addr),
        }
    }

    fn get_writeback_addr(&self, addr: usize) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.get_writeback_addr(addr),
            // Cache::SetAssociative(sa)    => sa.get_writeback_addr(addr),
            // Cache::FullyAssociative(fa)  => fa.get_writeback_addr(addr),
        } 
    }

    fn get_base_addr(&self, addr: usize) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.get_base_addr(addr),
            // Cache::SetAssociative(sa)    => sa.get_base_addr(addr),
            // Cache::FullyAssociative(fa)  => fa.get_base_addr(addr),
        } 
    }

    fn decode_addr(&self, addr: usize) -> (usize, usize, usize, usize) {
        match self {
            Cache::DirectMapped(dm) => dm.decode_addr(addr),
            // Cache::SetAssociative(sa)    => sa.decode_addr(addr),
            // Cache::FullyAssociative(fa)  => fa.decode_addr(addr),
        }
    }

    fn index_bits(&self) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.index_bits(),
            // Cache::SetAssociative(sa)    => sa.index_bits(),
            // Cache::FullyAssociative(fa)  => fa.index_bits(),
        }
    }

    fn word_bits(&self) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.word_bits(),
            // Cache::SetAssociative(sa)    => sa.word_bits(),
            // Cache::FullyAssociative(fa)  => fa.word_bits(),
        } 
    }

    fn byte_bits(&self) -> usize {
        match self {
            Cache::DirectMapped(dm) => dm.byte_bits(),
            // Cache::SetAssociative(sa)    => sa.byte_bits(),
            // Cache::FullyAssociative(fa)  => fa.byte_bits(),
        } 
    }
}

impl<const B: usize, const W: usize, const A: usize> MemLevelAccess for Cache<B, W, A> {
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
    time: u128,
    data: Vec<u8>,
}

impl CacheLine {
    pub fn new(words_per_line: usize) -> Self {
        CacheLine {
            valid: false,
            dirty: false,
            tag: 0,
            time: 0,
            data: vec![0; words_per_line * WORDSIZE],
        }
    }

    // getters
    pub fn is_valid(&self) -> bool { self.valid }
    pub fn is_dirty(&self) -> bool { self.dirty }
    pub fn tag(&self) -> usize     { self.tag }
    pub fn time(&self) -> u128     { self.time }
    pub fn get_data(&self) -> Vec<u8> { self.data.clone()}

    pub fn stamp_now(&mut self) {
        self.time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before 1970")
            .as_nanos();
    }

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
        self.stamp_now();
    }

    pub fn read_line_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}