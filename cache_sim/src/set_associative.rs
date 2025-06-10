use crate::{
    cache::{CacheAddressing, EvictionPolicy, CacheLine},
    mem_stats::*,
    memory::{DataType, DataTypeSize, MemLevelAccess, MemoryAccess, MemoryError},
};

/* --------------------------------------------------------------------- */

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

#[derive(Debug)]
pub struct SetAssocCache<
    const BYTES: usize,
    const WORDS_PER_LINE: usize,
    const ASSOC: usize,
> {
    /* sets[way][index] */
    sets: Vec<Vec<CacheLine>>,

    /* stats */
    eviction: EvictionPolicy,
    stats:    MemStats,
}

impl<const BYTES: usize, const WPL: usize, const A: usize>
    SetAssocCache<BYTES, WPL, A>
{
    pub const NUM_LINES: usize = BYTES / (A * WORDSIZE * WPL); // indices

    pub fn new(eviction: EvictionPolicy) -> Self {
        assert!(BYTES.is_power_of_two() && WPL.is_power_of_two());
        assert!(Self::NUM_LINES.is_power_of_two() && Self::NUM_LINES > 0);

        let sets = vec![vec![CacheLine::new(WPL); Self::NUM_LINES]; A];

        Self { sets, eviction, stats: MemStats::new() }
    }

    /* ---------------- lookup in a set ---------------- */
    fn find_line(&self, addr: usize) -> Option<(usize /*way*/, usize /*idx*/)> {
        let (tag, idx, ..) = self.decode_addr(addr);
        for way in 0..A {
            let line = &self.sets[way][idx];
            if line.is_valid() && line.tag() == tag {
                return Some((way, idx));
            }
        }
        None
    }

    /* ---------------- victim policy ------------------ */
    fn victim_way(&self, idx: usize) -> usize {
        match self.eviction {
            /* -------- LRU: smallest timestamp ----------- */
            EvictionPolicy::Lru => (0..A)
                .min_by_key(|&w| self.sets[w][idx].time())
                .unwrap(),

            /* -------- NRU: first line whose time == 0 --- */
            EvictionPolicy::Nru => {
                (0..A)
                    .find(|&w| self.sets[w][idx].time() == 0)
                    .unwrap_or_else(|| (0..A)
                        .min_by_key(|&w| self.sets[w][idx].time())
                        .unwrap())
            }

            /* any other variant – fall back to LRU */
            _ => (0..A).min_by_key(|&w| self.sets[w][idx].time()).unwrap(),
        }
    }

    /* ---------------- address helpers ---------------- */
    #[inline(always)] fn byte_bits (&self) -> usize { WORDSIZE.trailing_zeros() as usize }
    #[inline(always)] fn word_bits (&self) -> usize { WPL     .trailing_zeros() as usize }
    #[inline(always)] fn index_bits(&self) -> usize { Self::NUM_LINES.trailing_zeros() as usize }

    fn decode_addr(&self, addr: usize) -> (usize, usize, usize, usize) {
        let bb = self.byte_bits();
        let wb = self.word_bits();
        let ib = self.index_bits();

        let byte  =  addr & ((1 << bb) - 1);
        let word  = (addr >>  bb) & ((1 << wb) - 1);
        let idx   = (addr >> (bb + wb)) & ((1 << ib) - 1);
        let tag   =  addr >> (bb + wb + ib);

        (tag, idx, word, byte)
    }

    #[inline(always)]
    fn base_addr(&self, tag: usize, idx: usize) -> usize {
        let bb = self.byte_bits(); let wb = self.word_bits(); let ib = self.index_bits();
        (tag << (ib + wb + bb)) | (idx << (wb + bb))
    }
}

/* ===================================================================== */
/* ================          MemoryAccess impl            ============== */
/* ===================================================================== */

impl<const B: usize, const W: usize, const A: usize>
    MemoryAccess for SetAssocCache<B, W, A>
{
    fn read(&mut self, addr: usize, size: DataTypeSize, dont_count: bool)
        -> Result<DataType, MemoryError>
    {
        /* ---------- hit / miss ---------- */
        let (way, idx) = match self.find_line(addr) {
            Some(hit) => { 
                if !dont_count {self.stats.record_hit();} 
                hit 
            }
            None      => { 
                self.stats.record_miss(); 
                return Err(MemoryError::NotFound);
            }
        };

        /* ---------- mark for NRU ---------- */
        if matches!(self.eviction, EvictionPolicy::Nru) || matches!(self.eviction, EvictionPolicy::Lru) {
            self.sets[way][idx].stamp_now();        // reuse timestamp as ref-bit
        }

        /* ---------- extract bytes ---------- */
        let (_, _, word, byte) = self.decode_addr(addr);
        let line  = &self.sets[way][idx];
        let base  = word * WORDSIZE + byte;
        let read  = |i| line.read_byte(base + i);

        Ok(match size {
            DataTypeSize::Byte       => DataType::Byte(read(0)),
            DataTypeSize::Halfword   => DataType::Halfword(u16::from_le_bytes([read(0), read(1)])),
            DataTypeSize::Word       => DataType::Word(u32::from_le_bytes([read(0), read(1), read(2), read(3)])),
            DataTypeSize::DoubleWord => {
                let mut b = [0u8; 8]; for i in 0..8 { b[i] = read(i); }
                DataType::DoubleWord(u64::from_le_bytes(b))
            }
        })
    }

    fn write(&mut self, data: DataType, addr: usize, dont_count: bool) -> Result<(), MemoryError> {
        let (way, idx) = match self.find_line(addr) {
            Some(hit) => { 
                if !dont_count {self.stats.record_hit();} 
                hit
             }
            None      => { 
                self.stats.record_miss(); 
                return Err(MemoryError::NotFound); 
            }
        };

        if matches!(self.eviction, EvictionPolicy::Nru) || matches!(self.eviction, EvictionPolicy::Lru) {
            self.sets[way][idx].stamp_now();
        }

        let (_, _, word, byte) = self.decode_addr(addr);
        let offset = word * WORDSIZE + byte;
        let line   = &mut self.sets[way][idx];

        match data {
            DataType::Byte(b)       => line.write_byte(offset, b),
            DataType::Halfword(h)   => for (i, &b) in h.to_le_bytes().iter().enumerate() { line.write_byte(offset + i, b) },
            DataType::Word(w)       => for (i, &b) in w.to_le_bytes().iter().enumerate() { line.write_byte(offset + i, b) },
            DataType::DoubleWord(d) => for (i, &b) in d.to_le_bytes().iter().enumerate() { line.write_byte(offset + i, b) },
        }
        Ok(())
    }

    fn stats(&self) -> &MemStats { &self.stats }
}

/* ===================================================================== */
/* ==============     MemLevelAccess & CacheAddressing     ============= */
/* ===================================================================== */

impl<const B: usize, const W: usize, const A: usize>
    MemLevelAccess for SetAssocCache<B, W, A>
{
    fn write_line(&mut self, addr: usize, _wpl: usize, data: Vec<u8>) {
        let (tag, idx, ..) = self.decode_addr(addr);

        /* ---- try invalid slot first ---- */
        if let Some((way, line)) = self.sets
            .iter_mut()
            .enumerate()                      // way index
            .map(|(w, v)| (w, &mut v[idx]))
            .find(|(_, l)| !l.is_valid())
        {
            line.write_line(tag, data);
            /* NRU: reset age counter for the new line */
            if matches!(self.eviction, EvictionPolicy::Nru) {
                self.sets[way][idx].stamp_now(); // initial “used” = false (time == 0)
                self.sets[way][idx].reset_time(); // assume you have reset_time() that sets 0
            }
            return;
        }

        /* ---- evict victim ---- */
        let way  = self.victim_way(idx);
        let line = &mut self.sets[way][idx];
        line.write_line(tag, data);

        if matches!(self.eviction, EvictionPolicy::Nru) {
            line.reset_time();   // clear ref-bit
        }
    }

    fn fetch_line(&self, _addr: usize, _wpl: usize) -> Vec<u8> {
        vec![0; WORDSIZE * W]   // upper levels overwrite
    }
}

/* ---------------- CacheAddressing helpers ---------------- */

impl<const B: usize, const W: usize, const A: usize>
    CacheAddressing for SetAssocCache<B, W, A>
{
    #[inline] fn byte_bits (&self) -> usize { self.byte_bits() }
    #[inline] fn word_bits (&self) -> usize { self.word_bits() }
    #[inline] fn index_bits(&self) -> usize { self.index_bits() }

    fn decode_addr(&self, a: usize) -> (usize, usize, usize, usize) { self.decode_addr(a) }

    fn is_line_dirty(&self, a: usize) -> bool {
        self.find_line(a)
            .map(|(w, idx)| self.sets[w][idx].is_dirty())
            .unwrap_or(false)
    }

    fn get_evict_line_data(&self, a: usize) -> Vec<u8> {
        let idx = self.get_index(a);
        let way = self.victim_way(idx);
        self.sets[way][idx].get_data()
    }

    fn get_writeback_addr(&self, a: usize) -> usize {
        let idx = self.get_index(a);
        let way = self.victim_way(idx);
        let tag = self.sets[way][idx].tag();
        self.base_addr(tag, idx)
    }

    fn get_base_addr(&self, a: usize) -> usize {
        let (tag, idx, ..) = self.decode_addr(a);
        self.base_addr(tag, idx)
    }

    #[inline] fn get_tag        (&self, a: usize) -> usize { let (t, ..) = self.decode_addr(a); t }
    #[inline] fn get_index      (&self, a: usize) -> usize { let (_, i, ..) = self.decode_addr(a); i }
    #[inline] fn get_word_offset(&self, a: usize) -> usize { let (_, _, w, ..) = self.decode_addr(a); w }
    #[inline] fn get_byte_offset(&self, a: usize) -> usize { let (_, _, _, b) = self.decode_addr(a); b }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new () {
        type L1 = SetAssocCache<1024, 8, 2>;
        let l1 = L1::new(EvictionPolicy::Random);
        println!("{:#?}", l1);
    }

    #[test]
    fn parse_addr() {
        type L1 = SetAssocCache<4096, 8, 2>;
        let c = L1::new(EvictionPolicy::Random);

        let addr = 0x385;
        let tag = c.get_tag(addr);
        let index = c.get_index(addr);
        let word_offset = c.get_word_offset(addr);
        let byte_offset = c.get_byte_offset(addr);

        assert_eq!(tag, 0x0);
        assert_eq!(index, 0x1c);
        assert_eq!(word_offset, 0x1);
        assert_eq!(byte_offset, 0x1);

        assert_eq!(c.stats.total_accesses(), 0);
    }

    #[test]
    fn compulsory_miss () {
        const L1_SIZE: usize = 1 << 13;
        const WORD_P_LINE: usize = 8;
        const ASSOC: usize = 2;
        type L1 = SetAssocCache<L1_SIZE, WORD_P_LINE, ASSOC>;
        let mut c = L1::new(EvictionPolicy::Random);

        let addr = 0x385;

        let result = c.read(addr, DataTypeSize::Byte);
        
        assert!(
            matches!(result, Err(MemoryError::NotFound)),
            "expected Err(NotFound), got: {:?}",
            result
        );

        assert_eq!(c.stats.total_accesses(), 1);
    }

    #[test]
    fn single_write () {
        const L1_SIZE: usize = 1 << 13;
        const WORD_P_LINE: usize = 8;
        const ASSOC: usize = 2;
        type L1 = SetAssocCache<L1_SIZE, WORD_P_LINE, ASSOC>;
        let mut c = L1::new(EvictionPolicy::Random);

        let addr = 0x385;
        c.write_line(addr, 8, vec![0xff; WORDSIZE * 8]);

        let _ = c.write(DataType::Byte(0x11), addr);
        match c.read(addr, DataTypeSize::Byte) {
            Ok(DataType::Byte(d)) => assert_eq!(d, 0x11),
            _ => panic!("Incorrect Read")
        }

        let _ = c.write(DataType::Halfword(0x1234), addr);
        match c.read(addr, DataTypeSize::Halfword) {
            Ok(DataType::Halfword(d)) => assert_eq!(d, 0x1234),
            _ => panic!("Incorrect Read")
        }

        let _ = c.write(DataType::Word(0xcafebabe), addr);
        match c.read(addr, DataTypeSize::Word) {
            Ok(DataType::Word(d)) => assert_eq!(d, 0xcafebabe),
            _ => panic!("Incorrect Read")
        }
        
        let _ = c.write(DataType::DoubleWord(0x87654321cafebabe), addr);
        match c.read(addr, DataTypeSize::DoubleWord) {
            Ok(DataType::DoubleWord(d)) => assert_eq!(d, 0x87654321cafebabe),
            _ => panic!("Incorrect Read")
        }

        assert_eq!(c.stats.total_accesses(), 8);
        assert_eq!(c.stats.hit_rate(), (8 / 8) as f64);
        assert_eq!(c.stats.miss_rate(), (0 / 8) as f64);
    }

    #[test]
    fn write_mult_ways() {
        const L1_SIZE: usize = 1 << 13;
        const WORD_P_LINE: usize = 8;
        const ASSOC: usize = 2;
        type L1 = SetAssocCache<L1_SIZE, WORD_P_LINE, ASSOC>;
        let mut c = L1::new(EvictionPolicy::Random);

        let bb = c.byte_bits(); // lowest bits
        let wb = c.word_bits(); // next bits
        let ib = c.index_bits(); // next bits

        let addr1 = (1 << (ib + wb + bb)) | (0x8 << ib) | 
                            (0x2 << wb) | (0x0 << bb);
        let data1 = DataType::Word(0xcafebabe);
        let _ = c.write(data1.clone(), addr1.clone());

        let addr2 = (2 << (ib + wb + bb)) | (0x8 << ib) | 
                            (0x2 << wb) | (0x0 << bb);
        let data2 = DataType::Word(0xbabecafe);
        let _ = c.write(data2.clone(), addr2.clone());

        match c.read(addr1, DataTypeSize::Word) {
            Ok(w) => assert_eq!(w, data1),
            _=> panic!("[MEMORY] errror here")
        }

        match c.read(addr2, DataTypeSize::Word) {
            Ok(w) => assert_eq!(w, data2, "[MEMORY] write-back or reload failed"),
            Err(e) => panic!("[MEMORY] read error: {e:?}"),
        }

        assert_eq!(c.stats.total_accesses(), 4);
    }

    // #[test]
    // fn read () {
    //     const L1_SIZE: usize = 1 << 12;
    //     const WORD_P_LINE: usize = 8;
    //     type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
    //     let mut c = L1::new();

    //     let addr = 0x385;
    //     c.write_line(addr, 8, vec![0xa5; WORDSIZE * 8]);

    //     match c.read(addr, DataTypeSize::Byte) {
    //         Ok(DataType::Byte(d)) => assert_eq!(d, 0xa5),
    //         _ => panic!("Incorrect Read")
    //     }

    //     match c.read(addr, DataTypeSize::Halfword) {
    //         Ok(DataType::Halfword(d)) => assert_eq!(d, 0xa5a5),
    //         _ => panic!("Incorrect Read")
    //     }

    //     match c.read(addr, DataTypeSize::Word) {
    //         Ok(DataType::Word(d)) => assert_eq!(d, 0xa5a5a5a5),
    //         _ => panic!("Incorrect Read")
    //     }

    //     match c.read(addr, DataTypeSize::DoubleWord) {
    //         Ok(DataType::DoubleWord(d)) => assert_eq!(d, 0xa5a5a5a5a5a5a5a5),
    //         _ => panic!("Incorrect Read")
    //     }

    //     assert_eq!(c.stats.total_accesses(), 4);
    //     assert_eq!(c.stats.hit_rate(), (4 / 4) as f64);
    //     assert_eq!(c.stats.miss_rate(), (0 / 4) as f64);
    // }

    // #[test]
    // fn write_read_cache_line () {
    //     const L1_SIZE: usize = 1 << 12;
    //     const WORD_P_LINE: usize = 8;
    //     type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
    //     let mut c = L1::new();

    //     for i in 0..WORD_P_LINE {
    //         let i = i * WORDSIZE;
    //         let _ = c.read(i, DataTypeSize::Word);
    //     }

    //     c.stats.print_summary();

    //     assert_eq!(c.stats.total_accesses(), WORD_P_LINE);
    //     assert_eq!(c.stats.miss_rate(), 1.0);
    // }

    // #[test]
    // fn write_read_whole_cache () {
    //     const L1_SIZE: usize = 1 << 12;
    //     const WORD_P_LINE: usize = 8;
    //     type L1 = DMCache<L1_SIZE, WORD_P_LINE>;
    //     let mut c = L1::new();

    //     let vec: Vec<u8> = (0..L1_SIZE).map(|i| i as u8).collect();

    //     // change to step
    //     for i in 0..L1_SIZE {
    //         if i % (WORD_P_LINE * WORDSIZE) == 0 {
    //             let slice: Vec<u8> = (i..i+WORD_P_LINE*WORDSIZE)
    //                 .map(|j| vec[j])
    //                 .collect();
    //             c.write_line(i, WORD_P_LINE, slice);
    //         }
    //     }

    //     for i in 0..L1_SIZE {
    //         match c.read(i, DataTypeSize::Byte) {
    //             Ok(DataType::Byte(d)) => assert_eq!(d, vec[i]),
    //             _ => panic!("Incorrect Read")
    //         }
    //     }

    //     c.stats.print_summary();

    //     assert_eq!(c.stats.total_accesses(), L1_SIZE);
    //     assert_eq!(c.stats.hit_rate(), 1.0);
    //     assert_eq!(c.stats.miss_rate(), 0.0);
    // }
}