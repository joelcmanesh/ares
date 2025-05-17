use crate::mem_stats::*;
use crate::memory::*;
use crate::cache::*;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);
pub const ADDR_BITS: usize = 32;

#[derive(Debug)]
pub struct SetAssocCache {
    words_per_line: usize,
    pub stats: MemStats,
    lines: Vec<CacheLine>,
    index_mask: usize,
    word_mask: usize,
    index_shift: usize,
    word_shift: usize,
    tag_shift: usize,
}
