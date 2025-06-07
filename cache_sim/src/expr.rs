pub struct Memory<
    const FULL_BYTES: usize,
    const IM_L1_BYTES: usize,
    const IM_L1_WORDS_PER_LINE: usize,
    const DM_L1_BYTES: usize,
    const DM_L1_WORDS_PER_LINE: usize,
    const IM_L1_ASSOC: usize = 1,
    const DM_L1_ASSOC: usize = 1,
    > {
        size: usize,                                     
        stats: MemStats,
        im: Cache<IM_L1_BYTES, IM_L1_WORDS_PER_LINE, IM_L1_ASSOC>,
        dm: Cache<DM_L1_BYTES, DM_L1_WORDS_PER_LINE, DM_L1_ASSOC>,
        im_start_addr: usize,
        dm_start_addr: usize,
        main: MainMemory<FULL_BYTES>,
    }


pub enum Cache<
    const BYTES: usize,
    const WORDS_PER_LINE: usize,
    const ASSOC: usize = 1,        // only SA/FA care
> {
    DirectMapped(DMCache<BYTES, WORDS_PER_LINE>),
    SetAssociative(SetAssocCache<BYTES, WORDS_PER_LINE, ASSOC>),
    FullyAssociative(FAssocCache<BYTES, WORDS_PER_LINE, ASSOC>),
}

pub struct CacheLine {
    valid: bool,
    dirty: bool,
    tag: usize,
    time: u128,
    data: Vec<u8>,
}

pub enum DataType {
    Byte(u8),
    Halfword(u16),
    Word(u32),
    DoubleWord(u64),
}

pub enum MemoryError {
    OutOfBounds,
    NotAligned,
    NotFound,
    NotCompatible, 
}

pub enum DataTypeSize {
    Byte,
    Halfword,
    Word,
    DoubleWord,
}

pub trait MemoryAccess {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError>;
    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError>; 
    fn stats(&self) -> &MemStats;
}

pub struct Memory<
    const FULL_BYTES: usize,
    const IM_L1_BYTES: usize,
    const IM_L1_WORDS_PER_LINE: usize,
    const DM_L1_BYTES: usize,
    const DM_L1_WORDS_PER_LINE: usize,
    const IM_L1_ASSOC: usize = 1,
    const DM_L1_ASSOC: usize = 1,
    > {
        size: usize,                                     
        stats: MemStats,
        im: Cache<IM_L1_BYTES, IM_L1_WORDS_PER_LINE, IM_L1_ASSOC>,
        dm: Cache<DM_L1_BYTES, DM_L1_WORDS_PER_LINE, DM_L1_ASSOC>,
        im_start_addr: usize,
        dm_start_addr: usize,
        main: MainMemory<FULL_BYTES>,
    }