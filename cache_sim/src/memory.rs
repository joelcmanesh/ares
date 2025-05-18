use crate::mem_stats::*;
use crate::main_memory::*;
use crate::cache::*;
use crate::direct_map::*;
// use crate::set_associative::*;

use std::mem;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Byte(u8),
    Halfword(u16),
    Word(u32),
    DoubleWord(u64),
}

impl From<u8>  for DataType { fn from(v: u8)  -> Self { DataType::Byte(v) } }
impl From<u16> for DataType { fn from(v: u16) -> Self { DataType::Halfword(v) } }
impl From<u32> for DataType { fn from(v: u32) -> Self { DataType::Word(v) } }
impl From<u64> for DataType { fn from(v: u64) -> Self { DataType::DoubleWord(v) } }

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
    fn stats(&self) -> &MemStats;
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

#[derive(Copy, Clone)]
enum WhichL1 { Instr, Data }

#[derive(Debug)]
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

impl<
    const FULL_BYTES: usize,
    const IM_L1_BYTES: usize,
    const IM_L1_WORDS_PER_LINE: usize,
    const DM_L1_BYTES: usize,
    const DM_L1_WORDS_PER_LINE: usize,
    const IM_L1_ASSOC: usize,
    const DM_L1_ASSOC: usize,
    > Memory<
    FULL_BYTES,
    IM_L1_BYTES, IM_L1_WORDS_PER_LINE,
    DM_L1_BYTES, DM_L1_WORDS_PER_LINE,
    IM_L1_ASSOC, DM_L1_ASSOC >  {
    pub fn new(im_addr_start: usize, dm_addr_start: usize) -> Self {
        assert!(FULL_BYTES.is_power_of_two(), "main memory must be power of two");
        assert!(IM_L1_BYTES.is_power_of_two(), "IM L1 size must be power of two");
        assert!(IM_L1_WORDS_PER_LINE.is_power_of_two(), "IM line size must be power of two");
        assert!(DM_L1_BYTES.is_power_of_two(), "DM L1 size must be power of two");
        assert!(DM_L1_WORDS_PER_LINE.is_power_of_two(), "DM line size must be power of two");
        assert!(im_addr_start.is_power_of_two() || im_addr_start == 0, 
            "Instructions addr must start at pow of 2");
        assert!(dm_addr_start.is_power_of_two() || im_addr_start == 0, 
            "Data addr must start at pow of 2");

        // TODO: check addr spaces dont overlap
        // TODO: check that caches arent too big

        Self {
            im_start_addr: im_addr_start, 
            dm_start_addr: dm_addr_start,
            size: FULL_BYTES,
            stats: MemStats::new(),
            im: Cache::DirectMapped(DMCache::<IM_L1_BYTES, IM_L1_WORDS_PER_LINE>::new()),
            dm: Cache::DirectMapped(DMCache::<DM_L1_BYTES, DM_L1_WORDS_PER_LINE>::new()),
            main: MainMemory::new(),
        }
    }

    #[inline(always)]
    fn choose_cache(&self, addr: usize) -> Option<WhichL1> {
        if addr >= self.im_start_addr{
            return Some(WhichL1::Instr);
        } else if addr >= self.dm_start_addr {
            return Some(WhichL1::Instr);
        } else {
            return None;
        }
    }

    pub fn print_summary(&self) {
        println!("Memory");
        self.stats.print_summary();

        println!("IM L1");
        let im_l1_stats = self.im.stats();
        im_l1_stats.print_summary();

        println!("DM L1");
        let dm_l1_stats = self.dm.stats();
        dm_l1_stats.print_summary();

        println!("Main");
        let mm_stats = self.main.stats();
        mm_stats.print_summary();

    }
}

impl<
    const FULL_BYTES: usize,
    const IM_L1_BYTES: usize,
    const IM_L1_WORDS_PER_LINE: usize,
    const DM_L1_BYTES: usize,
    const DM_L1_WORDS_PER_LINE: usize,
    const IM_L1_ASSOC: usize,
    const DM_L1_ASSOC: usize,
    > MemoryAccess for Memory<FULL_BYTES, IM_L1_BYTES, IM_L1_WORDS_PER_LINE, IM_L1_ASSOC, DM_L1_BYTES, DM_L1_WORDS_PER_LINE, DM_L1_ASSOC> {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        if addr >= self.size {
            return Err(MemoryError::OutOfBounds);
        }

        let align = DataTypeSize::get_size(size.clone());
        if addr % align != 0 {
            return Err(MemoryError::NotAligned);
        }

        match self.choose_cache(addr) {
            Some(WhichL1::Instr) => {
                match self.im.read(addr, size.clone()) {
                    Ok(data) => {
                        self.stats.record_hit();
                        Ok(data)
                    }

                    Err(MemoryError::NotFound) => {
                        self.stats.record_miss();


                        let fetch_base_addr = self.im.get_base_addr(addr);

                        if self.im.is_line_dirty(addr) {
                            let write_back_addr = self.im.get_writeback_addr(addr);
                            let write_back_line = self.im.get_evict_line_data(addr);
                            self.main.write_line(write_back_addr, IM_L1_WORDS_PER_LINE, write_back_line);
                        }

                        let new_line = self.main.fetch_line(fetch_base_addr, IM_L1_WORDS_PER_LINE);
                        self.im.write_line(fetch_base_addr, IM_L1_WORDS_PER_LINE, new_line);
                        self.im.read(addr, size)
                    }

                    Err(e) => Err(e),
                }
            }
            Some(WhichL1::Data) => {
                match self.dm.read(addr, size.clone()) {
                    Ok(data) => {
                        self.stats.record_hit();
                        Ok(data)
                    }

                    Err(MemoryError::NotFound) => {
                        self.stats.record_miss();


                        let fetch_base_addr = self.dm.get_base_addr(addr);

                        if self.dm.is_line_dirty(addr) {
                            let write_back_addr = self.dm.get_writeback_addr(addr);
                            let write_back_line = self.dm.get_evict_line_data(addr);
                            self.main.write_line(write_back_addr, DM_L1_WORDS_PER_LINE, write_back_line);
                        }

                        let new_line = self.main.fetch_line(fetch_base_addr, DM_L1_WORDS_PER_LINE);
                        self.dm.write_line(fetch_base_addr, DM_L1_WORDS_PER_LINE, new_line);
                        self.dm.read(addr, size)
                    }

                    Err(e) => Err(e),
                }
            }
            _ => Err(MemoryError::NotCompatible),
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

        match self.choose_cache(addr) {
            Some(WhichL1::Instr) => {
                match self.im.write(data, addr) {
                    Ok(()) => {
                        self.stats.record_hit();
                        Ok(())
                    }

                    Err(MemoryError::NotFound) => {
                        self.stats.record_miss();

                        let fetch_base_addr = self.im.get_base_addr(addr);

                        if self.im.is_line_dirty(addr) {
                            let write_back_addr = self.im.get_writeback_addr(addr);
                            let write_back_line = self.im.get_evict_line_data(addr);
                            self.main.write_line(write_back_addr, IM_L1_WORDS_PER_LINE, write_back_line);
                        }

                        let new_line = self.main.fetch_line(fetch_base_addr, IM_L1_WORDS_PER_LINE);
                        self.im.write_line(fetch_base_addr, IM_L1_WORDS_PER_LINE, new_line);
                        self.im.write(data, addr)
                    }

                    Err(e) => Err(e),
                }

            }
            Some(WhichL1::Data) => {
                match self.dm.write(data, addr) {
                    Ok(()) => {
                        self.stats.record_hit();
                        Ok(())
                    }

                    Err(MemoryError::NotFound) => {
                        self.stats.record_miss();

                        let fetch_base_addr = self.dm.get_base_addr(addr);

                        if self.dm.is_line_dirty(addr) {
                            let write_back_addr = self.dm.get_writeback_addr(addr);
                            let write_back_line = self.dm.get_evict_line_data(addr);
                            self.main.write_line(write_back_addr, DM_L1_WORDS_PER_LINE, write_back_line);
                        }

                        let new_line = self.main.fetch_line(fetch_base_addr, DM_L1_WORDS_PER_LINE);
                        self.dm.write_line(fetch_base_addr, DM_L1_WORDS_PER_LINE, new_line);
                        self.dm.write(data, addr)
                    }

                    Err(e) => Err(e),
                }
            }
            _ => Err(MemoryError::NotCompatible),
        }


    }



    fn stats(&self) -> &MemStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 0.001;

    #[test]
    fn new_empty_cache_has_no_data() {
        const MEM_SIZE: usize = 1 << 12;
        const L1_SIZE: usize = 1 << 10;
        const W_P_L: usize = 4;
        type Mem = Memory<MEM_SIZE, L1_SIZE, W_P_L, L1_SIZE, W_P_L>;
        let mem = Mem::new(0, L1_SIZE);
        println!("{:#?}", mem);
    }

    #[test]
    fn write_and_read_back_byte() {
        const MEM_SIZE: usize = 1 << 12;
        const L1_SIZE: usize = 1 << 10;
        const W_P_L: usize = 4;
        type Mem = Memory<MEM_SIZE, L1_SIZE, W_P_L, L1_SIZE, W_P_L>;
        let mut m = Mem::new(0, L1_SIZE);

        let addr = 0x10;
        let byte = DataType::Byte(0xff);

        let _ = m.write(byte.clone(), addr);

        let mut dut_byte: DataType;
        match m.read(addr, DataTypeSize::Byte) {
            Ok(b) => dut_byte = b,  
            Err(MemoryError::NotFound) => panic!("mem error"),
            _ => panic!("idk")
        }
        assert_eq!(dut_byte, byte);

        assert_eq!(m.stats.total_accesses(), 2);
        assert_eq!(m.stats.hit_rate(), 0.5);
        assert_eq!(m.stats.miss_rate(), 0.5);
    }

    #[test]
    fn write_read_cache_line() {
        const MEM_SIZE: usize = 1 << 12;
        const L1_SIZE: usize = 1 << 10;
        const W_P_L: usize = 4;
        type Mem = Memory<MEM_SIZE, L1_SIZE, W_P_L, L1_SIZE, W_P_L>;
        let mut m = Mem::new(0, L1_SIZE);

        // write line to main mem and check values
        let expected_data: Vec<u32> = (0..W_P_L).map(|i| i as u32).collect();

        for i in 0..W_P_L {
            let addr = i * WORDSIZE;
            let _ = m.main.write(DataType::Word(expected_data[i]), addr);
        }

        for i in 0..W_P_L {
            let addr = i * WORDSIZE;
            let _ = m.main.read(addr, DataTypeSize::Word);
        }

        for i in 0..W_P_L {
            let addr = i * WORDSIZE;
            match m.read(addr, DataTypeSize::Word) {
                Ok(DataType::Word(w)) => assert_eq!(w, expected_data[i]),
                _ => panic!("Incorrect read @ {:#?}",addr)
            }
        }

        m.print_summary();

        let expected_accesses = W_P_L;
        let expected_hit = (W_P_L-1) as f64 / m.stats.total_accesses() as f64;
        let expected_miss = ((expected_accesses % W_P_L) + 1) as f64 / m.stats.total_accesses() as f64;

        assert_eq!(m.stats.total_accesses(), expected_accesses, "Incorrect accesses");
        assert!((m.stats.hit_rate() - expected_hit).abs() < EPSILON, "Incorrect Hit Rate");
        assert!((m.stats.miss_rate() - expected_miss).abs() < EPSILON, "Incorrect Miss Rate");

    }

    #[test]
    fn write_read_2cache_line() {
        const MEM_SIZE: usize = 1 << 12;
        const L1_SIZE: usize = 1 << 10;
        const W_P_L: usize = 16;
        type Mem = Memory<MEM_SIZE, L1_SIZE, W_P_L, L1_SIZE, W_P_L>;
        let mut m = Mem::new(0, L1_SIZE);

        let expected_accesses = W_P_L+1;
        for i in 0..expected_accesses {
            let addr = i * WORDSIZE;
            let _ = m.read(addr, DataTypeSize::Word);
        }

        m.print_summary();

        let expected_hit = (W_P_L-1) as f64 / m.stats.total_accesses() as f64;
        let expected_miss = ((expected_accesses % W_P_L) + 1) as f64 / m.stats.total_accesses() as f64;

        assert_eq!(m.stats.total_accesses(), expected_accesses, "Incorrect accesses");
        assert!((m.stats.hit_rate() - expected_hit).abs() < EPSILON, "Incorrect Hit Rate");
        assert!((m.stats.miss_rate() - expected_miss).abs() < EPSILON, "Incorrect Miss Rate");
    }

    #[test]
    fn write_read_whole_cache() {
        const MEM_SIZE: usize = 1 << 12;
        const L1_SIZE: usize = 1 << 10;
        const W_P_L: usize = 16;
        type Mem = Memory<MEM_SIZE, L1_SIZE, W_P_L, L1_SIZE, W_P_L>;
        let mut m = Mem::new(0, L1_SIZE);


        for i in 0..L1_SIZE/WORDSIZE {
            let addr = i * WORDSIZE;
            let _ = m.read(addr, DataTypeSize::Word);
        }

        m.print_summary();

        let expected_hit = (W_P_L-1) as f64 / W_P_L as f64;
        let expected_miss = 1 as f64 / W_P_L as f64;

        assert_eq!(m.stats.total_accesses(), L1_SIZE/WORDSIZE, "Incorrect accesses");
        assert!((m.stats.hit_rate() - expected_hit).abs() < EPSILON, "Incorrect Hit Rate");
        assert!((m.stats.miss_rate() - expected_miss).abs() < EPSILON, "Incorrect Miss Rate");
    }

    #[test]
    fn cache_eviction() {
        const MEM_SIZE: usize = 1 << 12;
        const L1_SIZE: usize = 1 << 10;
        const W_P_L: usize = 16;
        type Mem = Memory<MEM_SIZE, L1_SIZE, W_P_L, L1_SIZE, W_P_L>;
        let mut m = Mem::new(0, 2 * L1_SIZE);

        // cause a miss and write to the cache
        let bb = m.im.byte_bits(); // lowest bits
        let wb = m.im.word_bits(); // next bits
        let ib = m.im.index_bits(); // next bits

        let addr1 = (1 << (ib + wb + bb)) | (0x8 << ib) | 
                            (0x4 << wb) | (0x0 << bb);
        let data1 = DataType::Word(0xcafebabe);
        let _ = m.write(data1.clone(), addr1.clone());

        // get mapped to the same index and evict the old line
        let addr2 = (2 << (ib + wb + bb)) | (0x8 << ib) | 
                            (0x4 << wb) | (0x0 << bb);
        match m.read(addr2, DataTypeSize::Word) {
            Ok(w) => assert_ne!(w, data1),
            _=> panic!("[MEMORY] errror here")
        }

        match m.read(addr1, DataTypeSize::Word) {
            Ok(w) => assert_eq!(w, data1, "[MEMORY] write-back or reload failed"),
            Err(e) => panic!("[MEMORY] read error: {e:?}"),
        }

        m.print_summary();

        assert_eq!(m.stats.total_accesses(), 3);
        assert_eq!(m.stats.hit_rate(), 0.0);
        assert_eq!(m.stats.miss_rate(), 1.0);
    }

    #[test]
    fn write_read_whole_mem() {
        fn hash_func(val: usize) -> DataType {
            let h = ((val * 7) ^ (val * 17) ^ val) as u32;
            DataType::from(h)
        }

        const MEM_SIZE: usize = 1 << 12;
        const L1_SIZE: usize = 1 << 10;
        const W_P_L: usize = 16;
        type Mem = Memory<MEM_SIZE, L1_SIZE, W_P_L, L1_SIZE, W_P_L>;
        let mut m = Mem::new(0, 2 * L1_SIZE);

        let expected_data: Vec<DataType> = (0..MEM_SIZE)
            .map(|i| hash_func(i))
            .collect();

        for i in 0..MEM_SIZE/WORDSIZE {
            let w_data = hash_func(i);
            let _ = m.write(w_data, i * WORDSIZE);
        }

        for i in 0..MEM_SIZE/WORDSIZE {
            match m.read(i * WORDSIZE, DataTypeSize::Word) {
                Ok(w) => assert_eq!(w, expected_data[i], "[MEMORY] write-back or reload failed"),
                Err(e) => panic!("[MEMORY] read error: {e:?}"),
            }  
        }
    }

    // #[test]
    // fn access_im() {

    // }

    // // #[test]
    // // fn access_dm() {

    // // }


    // // #[test]
    // // fn access_im_dm() {

    // // }


    // // #[test]
    // // fn access_im_dm_evictions() {

    // // }


    // // #[test]
    // // fn access_im_dm_whole() {

    // // }

}

