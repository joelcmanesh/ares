use crate::{mem_stats::MemStats, memory::*};

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

#[derive(Debug)]
pub struct MainMemory<
    const BYTES: usize
> {
    data: Vec<u8>,
    stats: MemStats,
}

impl<const BYTES: usize> MainMemory<BYTES> {
    pub fn new() -> Self {
        MainMemory { data: vec![0; BYTES], stats: MemStats::new()}
    }
}

impl<const BYTES: usize> MemLevelAccess for MainMemory<BYTES> {
    fn write_line(&mut self, base_addr: usize, words_per_lines: usize, data: Vec<u8>) {
        let n_bytes: usize = words_per_lines * WORDSIZE; 

        self.data[base_addr..(n_bytes + base_addr)].copy_from_slice(&data[..n_bytes]);
    }

    fn fetch_line(&self, base_addr: usize, words_per_lines: usize) -> Vec<u8> {
        let n_bytes: usize = words_per_lines * WORDSIZE; 
        let mut ret_vec: Vec<u8> = vec![0; n_bytes];
    
        ret_vec[..n_bytes].copy_from_slice(&self.data[base_addr..(n_bytes + base_addr)]);
        ret_vec
    }
}

impl<const BYTES: usize> MemoryAccess for MainMemory<BYTES> {
    fn read(&mut self, addr: usize, size: DataTypeSize, _dont_count: bool) -> Result<DataType, MemoryError> {
        if addr >= BYTES {
            return Err(MemoryError::OutOfBounds);
        }

        match size {
            DataTypeSize::Byte => {
                let byte = self.data[addr];
                // println!("[MAIN MEMORY] reading {} at {:x}", byte, addr);
                Ok(DataType::Byte(byte))
            }

            DataTypeSize::Halfword => {
                if addr % 2 != 0 {
                    return Err(MemoryError::NotAligned);
                }
                if addr + 1 >= BYTES {
                    return Err(MemoryError::OutOfBounds);
                }
                let val = u16::from_le_bytes([
                    self.data[addr], 
                    self.data[addr + 1]
                ]);
                Ok(DataType::Halfword(val))
            }

            DataTypeSize::Word => {
                if addr % 4 != 0 {
                    return Err(MemoryError::NotAligned);
                }
                if addr + 3 >= BYTES {
                    return Err(MemoryError::OutOfBounds);
                }
                let val = u32::from_le_bytes([
                    self.data[addr],
                    self.data[addr + 1],
                    self.data[addr + 2],
                    self.data[addr + 3],
                ]);

                // println!("[MAIN MEMORY] reading {} at {:x}", val, addr);
                Ok(DataType::Word(val))
            }

            DataTypeSize::DoubleWord => {
                if addr % 8 != 0 {
                    return Err(MemoryError::NotAligned);
                }
                if addr + 7 >= BYTES {
                    return Err(MemoryError::OutOfBounds);
                }
                let val = u64::from_le_bytes([
                    self.data[addr],
                    self.data[addr + 1],
                    self.data[addr + 2],
                    self.data[addr + 3],
                    self.data[addr + 4],
                    self.data[addr + 5],
                    self.data[addr + 6],
                    self.data[addr + 7],
                ]);
                Ok(DataType::DoubleWord(val))
            }
        }
    }

    fn write(&mut self, data: DataType, addr: usize, _dont_count: bool) -> Result<(), MemoryError> {
        if addr >= BYTES {
            return Err(MemoryError::OutOfBounds);
        }

        match data {
            DataType::Byte(val) => {
                self.data[addr] = val;
                Ok(())
            }

            DataType::Halfword(val) => {
                if addr % 4 != 0 {
                    return Err(MemoryError::NotAligned);
                }
                if addr + 3 >= BYTES {
                    return Err(MemoryError::OutOfBounds);
                }

                let bytes = val.to_le_bytes();
                self.data[addr..(DataTypeSize::get_size(DataTypeSize::Halfword) + addr)]
                    .copy_from_slice(&bytes);

                Ok(())
            }

            DataType::Word(val) => {
                if addr % 4 != 0 {
                    return Err(MemoryError::NotAligned);
                }
                if addr + 3 >= BYTES {
                    return Err(MemoryError::OutOfBounds);
                }
                
                let bytes = val.to_le_bytes();
                self.data[addr..(DataTypeSize::get_size(DataTypeSize::Word) + addr)]
                    .copy_from_slice(&bytes);

                Ok(())
            }

            DataType::DoubleWord(val) => {
                if addr % 4 != 0 {
                    return Err(MemoryError::NotAligned);
                }
                if addr + 3 >= BYTES {
                    return Err(MemoryError::OutOfBounds);
                }
                
                let bytes = val.to_le_bytes();
                self.data[addr..(DataTypeSize::get_size(DataTypeSize::DoubleWord) + addr)]
                    .copy_from_slice(&bytes);

                Ok(())
            }
        }
    }

    fn stats(&self) -> &MemStats {
        &self.stats
    }

}