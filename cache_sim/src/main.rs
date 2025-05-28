use std::io::{BufReader, BufRead, Result};
use std::fs::File;

mod mem_stats;
mod memory;
mod main_memory;
mod cache;
mod direct_map;
mod set_associative;

use crate::memory::{Memory, DataTypeSize, DataType, MemoryAccess};

// const MEM_FILE: &str = "mem_files/small_minecraft.txt";
// const MEM_FILE: &str = "mem_files/small_flappy.txt";
// const MEM_FILE: &str = "mem_files/big_minecraft_log1.txt";
const MEM_FILE: &str = "mem_files/big_flappy_log1.txt";
// const MEM_FILE: &str = "mem_files/big_data_trace.txt";

fn main() -> Result<()> {
    let f = File::open(MEM_FILE)?;
    let reader = BufReader::new(f);
    
    const MEM_SIZE: usize =  1 << 32; // 4 MiB
    // const MEM_SIZE: usize =  1 << 22; // 4 MiB
    const IM_L1_SIZE: usize = 1 << 14; // 16 KiB
    const IM_W_P_L: usize = 16;
    const DM_L1_SIZE: usize = 1 << 13; // 8 KiB
    const DM_W_P_L: usize = 8;
    
    type N64Mem = Memory<MEM_SIZE, IM_L1_SIZE, IM_W_P_L, DM_L1_SIZE, DM_W_P_L>;
    const IM_BASE: usize = 0;
    const DM_BASE: usize = 0;

    let mut mem = N64Mem::new(IM_BASE, DM_BASE);

    let lines = reader.lines();
     
    for line in lines.flatten() {
        let args : Vec<&str> = line.split_ascii_whitespace().collect();
        
        let Some(op) = args[0].chars().next() else {
            continue;
        };

        let Ok(addr) = usize::from_str_radix(args[1], 16) else {
            continue;
        };

        // println!("op {} @ {:x}({}) \t", op, addr, addr);
        
        match op {
            'r' => {
                let size = match args[2].parse::<usize>().unwrap() {
                    1 => DataTypeSize::Byte,
                    2 => DataTypeSize::Halfword,
                    4 => DataTypeSize::Word,
                    8 => DataTypeSize::DoubleWord,
                    _ => DataTypeSize::Word,
                };
                let val = mem.read(addr, size, false).unwrap();
                // println!("read @ {} -> {:?}", addr, val);
            }
            'w' => {

                // let val = args[2].parse::<u8>().unwrap();
                mem.write(DataType::Byte(255), addr, false).unwrap();
                // println!("write {} @ {}", val, addr);
            }
            _ => continue,
        }
    }

    mem.print_summary();

    Ok(())
}
