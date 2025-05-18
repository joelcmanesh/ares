use std::io::{BufReader, BufRead, Result};
use std::fs::File;

mod mem_stats;
mod memory;
mod main_memory;
mod cache;
mod direct_map;
mod set_associative;

use crate::memory::{Memory, DataTypeSize, DataType, MemoryAccess};

// const MEM_FILE: &str = "mem_files/data_trace.txt";
const MEM_FILE: &str = "mem_files/small_trace.txt";

fn main() -> Result<()> {
    // let f = File::open(MEM_FILE)?;
    // let reader = BufReader::new(f);
    
    // const MEM_SIZE: usize =  1 << 22; // 4 MiB
    // const L1_SIZE: usize =  1 << 14; // 16 KiB
    // const L1_WPL: usize = 8;

    // type N64Mem = Memory<MEM_SIZE, L1_SIZE, L1_WPL>;
    // let mut mem = N64Mem::new();

    // let lines = reader.lines();
     
    // for line in lines.flatten() {
    //     let args : Vec<&str> = line.split_ascii_whitespace().collect();
    //     let op = args[0].chars().next().unwrap();
        
    //     let addr = usize::from_str_radix(args[1], 16).unwrap();
        
    //     print!("op {} @ {:x}\t", op, addr);
    //     match op {
    //         'r' => {
    //             let val = mem.read(addr, DataTypeSize::Word).unwrap();
    //             println!("read @ {} -> {:?}", addr, val);
    //         }
    //         'w' => {
    //             // example: args = ["w", "16", "42"]
    //             let val = args[2].parse::<u8>().unwrap();
    //             mem.write(DataType::Byte(val), addr).unwrap();
    //             println!("write {} @ {}", val, addr);
    //         }
    //         _ => continue,
    //     }
    // }

    // mem.print_summary();

    Ok(())
}
