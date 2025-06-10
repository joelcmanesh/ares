use std::io::{BufReader, BufRead, Result};
use std::fs::File;

mod mem_stats;
mod memory;
mod main_memory;
mod cache;
mod direct_map;
mod set_associative;

use crate::memory::{Memory, DataTypeSize, DataType, MemoryAccess};

// const TRACE_FILE: &str = "mem_files/big_minecraft_log2.txt";
const TRACE_FILE: &str = "mem_files/big_flappy_log2.txt";
// const TRACE_FILE: &str = "mem_files/med_flappy.txt";
// const TRACE_FILE: &str = "mem_files/small_flappy.txt";

/* ── compile-time cache geometry ─────────────────────────────────────── */
const FULL_BYTES         : usize = 1 << 22;   // 4 MiB main memory
const IM_L1_BYTES        : usize = 1 << 14;   // 16 KiB I-cache
const IM_L1_WORDS_PER_LN : usize = 8;         // 8 words / line
const DM_L1_BYTES        : usize = 1 << 13;   // 8 KiB D-cache
const DM_L1_WORDS_PER_LN : usize = 4;         // 4 words / line

type SimMem = Memory<
    FULL_BYTES,
    IM_L1_BYTES, IM_L1_WORDS_PER_LN,
    DM_L1_BYTES, DM_L1_WORDS_PER_LN,
    2, 2
>;

const DM_BASE  : usize = 0x0060_0000;  // start of data region
const MMIO_BASE: usize = 0xA000_0000;  // start of MMIO region

/* ─────────────────────────────────────────────────────────────────────── */

fn main() -> Result<()> {
    let reader = BufReader::new(File::open(TRACE_FILE)?);

    let mut mem = SimMem::new(MMIO_BASE, DM_BASE);

    let mut counter = 0;
    for (line_no, line) in reader.lines().flatten().enumerate() {
        if line.trim().is_empty() { continue; }

        let cols: Vec<&str> = line.split_ascii_whitespace().collect();
        let op   = cols[0].chars().next().unwrap();
        let addr = usize::from_str_radix(cols[1], 16)
            .expect("address must be hex");
        let sz_b = cols[2].parse::<usize>()
            .expect("size must be decimal (1/2/4/8)");

        /* map byte-count → enum */
        let size_enum = match sz_b {
            1 => DataTypeSize::Byte,
            2 => DataTypeSize::Halfword,
            4 => DataTypeSize::Word,
            8 => DataTypeSize::DoubleWord,
            _ => { eprintln!("L{line_no}: unsupported size {sz_b}"); continue; }
        };

        counter += 1;
        match op {
            /* ---------------- READ ---------------- */
            'r' => {
                let val = mem.read(addr, size_enum, false)
                    .unwrap_or_else(|e| panic!("L{line_no}: {e:?}"));
                // println!("read {sz_b}B @ 0x{addr:x} → {:?}", val);
            }

            /* ---------------- WRITE --------------- */
            'w' => {
                if cols.len() != 4 {
                    eprintln!("L{line_no}: write line needs a value");
                    continue;
                }
                let raw = cols[3].parse::<u64>()
                    .expect("value must be decimal");

                let data = match sz_b {
                    1 => DataType::Byte(raw as u8),
                    2 => DataType::Halfword(raw as u16),
                    4 => DataType::Word(raw as u32),
                    8 => DataType::DoubleWord(raw),
                    _ => unreachable!(),
                };

                mem.write(data, addr, false)
                    .unwrap_or_else(|e| panic!("L{line_no}: {e:?}"));
                // println!("write {:?} @ 0x{addr:x}", data);
            }

            _ => eprintln!("L{line_no}: unknown op '{op}'"),
        }
    }

    /* optional: show cache & memory statistics */
    mem.print_summary();
    
    println!("Completed {counter} operations");
    Ok(())
}

