use crate::memory::*;
use crate::mem_stats::*;
use crate::direct_mapped_cache::*;
// use crate::set_associative::*;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

// #[warn(dead_code)]
// #[derive(Debug)]
// pub enum EvictionPolicy {
//     LRU,
//     RANDOM
// }

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

// impl<const BYTES: usize, const WORDS_PER_LINE: usize, const ASSOC: usize> Cache<BYTES, WORDS_PER_LINE, ASSOC> {
//     pub fn print_summary(&self) {
//         match self {
//             Cache::DirectMapped(dm) => dm.print_summary(),
//             // Cache::SetAssociative(sa) => sa.print_summary(),
//             // Cache::FullyAssociative(fa) => fa.print_summary(),
//         }
//     }
// }

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

    fn stats(&self) -> &MemStats {
        match self {
            Cache::DirectMapped(dm) => dm.stats(),
            // Cache::SetAssociative(sa) => sa.stats(),
            // Cache::FullyAssociative(fa) => fa.stats(),
        }
    }
}

// impl CacheAddressing for Cache {
//     fn get_tag(&self, addr: usize) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.get_tag(addr),
//             // Cache::SetAssociative(sa)    => sa.get_tag(addr),
//             // Cache::FullyAssociative(fa)  => fa.get_tag(addr),
//         }
//     }

//     fn get_index(&self, addr: usize) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.get_index(addr),
//             // Cache::SetAssociative(sa)    => sa.get_index(addr),
//             // Cache::FullyAssociative(fa)  => fa.get_index(addr),
//         }
//     }

//     fn get_word_offset(&self, addr: usize) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.get_word_offset(addr),
//             // Cache::SetAssociative(sa)    => sa.get_word_offset(addr),
//             // Cache::FullyAssociative(fa)  => fa.get_word_offset(addr),
//         }
//     }

//     fn get_byte_offset(&self, addr: usize) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.get_byte_offset(addr),
//             // Cache::SetAssociative(sa)    => sa.get_byte_offset(addr),
//             // Cache::FullyAssociative(fa)  => fa.get_byte_offset(addr),
//         }
//     }

//     fn is_line_dirty(&self, addr: usize) -> bool {
//         match self {
//             Cache::DirectMapped(dm) => dm.is_line_dirty(addr),
//             // Cache::SetAssociative(sa)    => sa.is_line_dirty(addr),
//             // Cache::FullyAssociative(fa)  => fa.is_line_dirty(addr),
//         }
//     }

//     fn get_evict_line(&self, addr: usize) -> Vec<u8> {
//         match self {
//             Cache::DirectMapped(dm) => dm.get_evict_line(addr),
//             // Cache::SetAssociative(sa)    => sa.get_evict_line(addr),
//             // Cache::FullyAssociative(fa)  => fa.get_evict_line(addr),
//         }
//     }

//     fn get_words_p_line(&self) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.get_words_p_line(),
//             // Cache::SetAssociative(sa)    => sa.get_words_p_line(),
//             // Cache::FullyAssociative(fa)  => fa.get_words_p_line(),
//         }
//     }

//     fn get_tag_shift(&self) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.get_tag_shift(),
//             // Cache::SetAssociative(sa)    => sa.get_tag_shift(),
//             // Cache::FullyAssociative(fa)  => fa.get_tag_shift(),
//         }
//     }

//     fn get_writeback_addr(&self, addr: usize) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.get_writeback_addr(addr),
//             // Cache::SetAssociative(sa)    => sa.get_writeback_addr(addr),
//             // Cache::FullyAssociative(fa)  => fa.get_writeback_addr(addr),
//         } 
//     }

//     fn get_base_addr(&self, addr: usize) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.get_base_addr(addr),
//             // Cache::SetAssociative(sa)    => sa.get_base_addr(addr),
//             // Cache::FullyAssociative(fa)  => fa.get_base_addr(addr),
//         } 
//     }

//     fn index_bits(&self) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.index_bits(),
//             // Cache::SetAssociative(sa)    => sa.index_bits(),
//             // Cache::FullyAssociative(fa)  => fa.index_bits(),
//         }
//     }

//     fn word_bits(&self) -> usize {
//         match self {
//             Cache::DirectMapped(dm) => dm.word_bits(),
//             // Cache::SetAssociative(sa)    => sa.word_bits(),
//             // Cache::FullyAssociative(fa)  => fa.word_bits(),
//         } 
//     }
// }

// impl MemLevelAccess for Cache {
//     fn write_line(&mut self, addr: usize, words_per_lines: usize, data: Vec<u8>) {
//         match self {
//             Cache::DirectMapped(dm) => dm.write_line(addr, words_per_lines, data),
//             // Cache::SetAssociative(sa) => sa.write_line(addr, size),
//             // Cache::FullyAssociative(fa) => fa.write_line(addr, size),
//         }
//     }

//     fn fetch_line(&self, addr: usize, words_per_lines: usize) -> Vec<u8> {
//         match self {
//             Cache::DirectMapped(dm) => dm.fetch_line(addr, words_per_lines),
//             // Cache::SetAssociative(sa) => sa.fetch_line(addr, size),
//             // Cache::FullyAssociative(fa) => fa.fetch_line(addr, size),
//         }
//     }
// }

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

    pub fn read_line_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}