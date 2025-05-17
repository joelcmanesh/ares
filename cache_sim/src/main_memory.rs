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
        for i in 0..n_bytes {
            self.data[base_addr + i] = data[i];
        }
    }

    fn fetch_line(&self, base_addr: usize, words_per_lines: usize) -> Vec<u8> {
        let n_bytes: usize = words_per_lines * WORDSIZE; 
        let mut ret_vec: Vec<u8> = vec![0; n_bytes];
        for i in 0..n_bytes {
            ret_vec[i] = self.data[base_addr + i].clone(); 
        }
        ret_vec
    }
}

impl<const BYTES: usize> MemoryAccess for MainMemory<BYTES> {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        if addr >= BYTES {
            return Err(MemoryError::OutOfBounds);
        }

        match size {
            DataTypeSize::Byte => {
                let byte = self.data[addr];
                println!("[MAIN MEMORY] reading {} at {:x}", byte, addr);
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

                println!("[MAIN MEMORY] reading {} at {:x}", val, addr);
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

    fn write(&mut self, data: DataType, addr: usize) -> Result<(), MemoryError> {
        if addr >= BYTES {
            return Err(MemoryError::OutOfBounds);
        }

        match data {
            DataType::Byte(val) => {
                // println!("[MAIN MEMORY] write {} at {:x}", val, addr);

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
                for i in 0..2 {
                    self.data[addr + i] = bytes[i];
                }
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
                for i in 0..4 {
                    self.data[addr + i] = bytes[i];
                }

                let mut dut_bytes = [0 as u8; WORDSIZE];
                for i in 0..4 {
                    dut_bytes[i] = self.data[addr + i];
                }

                // println!("[MAIN MEMORY] write {:#?} at {:x}", dut_bytes, addr);
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
                for i in 0..8 {
                    self.data[addr + i] = bytes[i];
                }
                Ok(())
            }
        }
    }

    fn stats(&self) -> &MemStats {
        &self.stats
    }

}