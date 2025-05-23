use crate::memory::*;

const WORDSIZE: usize = DataTypeSize::get_size(DataTypeSize::Word);

#[derive(Debug)]
pub struct MainMemory {
    size: usize,
    data: Vec<u8>,
}

impl MainMemory {
    pub fn new(size: usize) -> Self {
        MainMemory {size, data: vec![0; size] }
    }
}

impl MemLevelAccess for MainMemory {
    fn write_line(&mut self, base_addr: usize, words_per_lines: usize, data: Vec<u8>) {
        let last_addr = base_addr + (words_per_lines * WORDSIZE); 
        for i in base_addr..last_addr {
            self.data[i] = data[i];
        }
    }

    fn fetch_line(&self, base_addr: usize, words_per_lines: usize) -> Vec<u8> {
        let n_bytes: usize = words_per_lines * WORDSIZE; 
        let mut ret_vec: Vec<u8> = vec![0; n_bytes];
        for i in 0..n_bytes {
            ret_vec[i] = self.data[base_addr + n_bytes].clone(); 
        }
        ret_vec
    }
}

// TODO: I dont think this is needed anymore
impl MemoryAccess for MainMemory {
    fn read(&mut self, addr: usize, size: DataTypeSize) -> Result<DataType, MemoryError> {
        if addr >= self.size {
            return Err(MemoryError::OutOfBounds);
        }

        match size {
            DataTypeSize::Byte => {
                Ok(DataType::Byte(self.data[addr]))
            }

            DataTypeSize::Halfword => {
                if addr % 2 != 0 {
                    return Err(MemoryError::NotAligned);
                }
                if addr + 1 >= self.size {
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
                if addr + 3 >= self.size {
                    return Err(MemoryError::OutOfBounds);
                }
                let val = u32::from_le_bytes([
                    self.data[addr],
                    self.data[addr + 1],
                    self.data[addr + 2],
                    self.data[addr + 3],
                ]);
                Ok(DataType::Word(val))
            }

            DataTypeSize::DoubleWord => {
                if addr % 8 != 0 {
                    return Err(MemoryError::NotAligned);
                }
                if addr + 7 >= self.size {
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
        if addr >= self.size {
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
                if addr + 3 >= self.size {
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
                if addr + 3 >= self.size {
                    return Err(MemoryError::OutOfBounds);
                }
                let bytes = val.to_le_bytes();
                for i in 0..4 {
                    self.data[addr + i] = bytes[i];
                }
                Ok(())
            }

            DataType::DoubleWord(val) => {
                if addr % 4 != 0 {
                    return Err(MemoryError::NotAligned);
                }
                if addr + 3 >= self.size {
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

}