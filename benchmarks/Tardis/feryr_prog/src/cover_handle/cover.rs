use failure;
use likely_stable::unlikely;
use memmap2::MmapOptions;
use std::fs::OpenOptions;

// static COUNT_CLASS_LOOKUP16: [u16; 65536];

static COUNT_LOOKUP: [u8; 256] = [
    0, 1, 2, 4, 8, 8, 8, 8, 16, 16, 16, 16, 16, 16, 16, 16, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    64, 64, 64, 64, 64, 64, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
];

// macro_rules! cast {
//     ($ptr:expr) => {{
//         unsafe { std::mem::transmute($ptr) }
//     }};
// }

pub const MAP_SIZE_POW2: usize = 20;
pub const BRANCHES_SIZE: usize = 1 << MAP_SIZE_POW2;
pub type BranchBuf = [u8];
#[cfg(target_pointer_width = "32")]
type BranchEntry = u32;
#[cfg(target_pointer_width = "64")]
// type BranchEntry = u64;
#[cfg(target_pointer_width = "32")]
const ENTRY_SIZE: usize = 4;
#[cfg(target_pointer_width = "64")]
// const ENTRY_SIZE: usize = 8;
// type BranchBufPlus = [BranchEntry; BRANCHES_SIZE / ENTRY_SIZE];
#[derive(Debug)]
pub struct Cover {
    pub branch: usize,
    pub last_branch: usize,
    pub virgin_bits: Box<[u8]>, // untouched
}

impl Default for Cover {
    fn default() -> Self {
        Self::new()
    }
}
impl Cover {
    pub fn new() -> Cover {
        let virgin = vec![255u8; BRANCHES_SIZE];
        let cover = Cover {
            branch: 0,
            last_branch: 0,
            virgin_bits: virgin.into_boxed_slice(),
        };
        return cover;
    }

    // read trace_bits from shared memory, called when exec a prog
    // also reset bits_map
    pub fn handle_bits(&self, tid: &usize) -> Result<Box<[u8]>, failure::Error> {
        let mut buf = vec![0u8; BRANCHES_SIZE];
        // open coverage file
        let file_name = "/dev/shm/cover-".to_string() + &tid.to_string();
        // let file = OpenOptions::new().read(true).write(true).open(file_name);
        let file = OpenOptions::new()
                                                    .read(true)
                                                    .write(true)
                                                    .open(file_name);
        match file {
            Err(e) => {
                dbg!("open coverage shm file failed");
                return Err(failure::format_err!("{:?}", e));
            }
            Ok(file) => {
                let mut cover_shm = unsafe { MmapOptions::new().map_mut(&file)? };
                buf.copy_from_slice(&cover_shm[0..BRANCHES_SIZE]);
                let mut trace_bits = buf.into_boxed_slice();
                self.classify_counts(&mut trace_bits).unwrap();

                // clear the bits
                cover_shm[0..BRANCHES_SIZE].copy_from_slice(&vec![0u8; BRANCHES_SIZE]);
                Ok(trace_bits)
            }
        }
    }

    pub fn classify_counts(&self, mem: &mut Box<[u8]>) -> Result<(), failure::Error> {
        for i in 0..BRANCHES_SIZE {
            if unlikely(mem[i] != 0) {
                let mem8 = mem[i];
                mem[i] = COUNT_LOOKUP[mem8 as usize];
            }
        }

        Ok(())
    }

    //    Check if the current execution path brings anything new to the table.
    //    Update virgin bits to reflect the finds. Returns 1 if the only change is
    //    the hit-count for a particular tuple; 2 if there are new tuples seen.
    //    Updates the map, so subsequent calls will always return 0.

    pub fn has_new_bits(&mut self, trace_bit: Box<[u8]>) -> Result<u8, failure::Error> {
        let mut ret = 0;
        let mut add_branch = 0;
        let mut zero_exist = false;
        for i in 0..BRANCHES_SIZE {
            if unlikely(trace_bit[i] != 0) && unlikely(trace_bit[i] & self.virgin_bits[i] != 0) {
                if trace_bit[i] != 0 && self.virgin_bits[i] == 0xff {
                    add_branch += 1;
                    if unlikely(ret < 2) {
                        ret = 2;
                    }
                }
                self.virgin_bits[i] &= !trace_bit[i];
            } else if unlikely(zero_exist == false) && trace_bit[i] == 0 {
                zero_exist = true;
            }
        }

        if zero_exist {
            self.branch += add_branch;
        } else {
            ret = 3;
        }

        Ok(ret)
    }

    pub fn count_bits(&self, mem: Box<[u8]>) -> Result<u64, failure::Error> {
        let mut i = BRANCHES_SIZE >> 2;
        let mut ret: u64 = 0;
        let mut idx = 0;
        while i != 0 {
            if idx + 3 >= BRANCHES_SIZE {
                break;
            }
            let mut v = u32::from_le_bytes([mem[idx], mem[idx + 1], mem[idx + 2], mem[idx + 3]]);
            if v == 0xffffffff {
                ret += 32;
                continue;
            }

            v -= (v >> 1) & 0x55555555;
            v = (v & 0x33333333) + ((v >> 2) & 0x33333333);
            let tmp_ret = (((v + (v >> 4)) & 0xF0F0F0F) * 0x01010101) >> 24;
            ret += tmp_ret as u64;
            i -= 1;
            idx += 3;
        }
        Ok((BRANCHES_SIZE << 3 - ret) as u64)
    }

    // called when normally exec a prog, check if is interesting
    // pub fn check_if_interesting(&self, trace_cover: &mut Cover) -> Result<bool, failure::Error> {
    //     let trace_bits = self.read_bits();
    //     let hnb = self
    //         .has_new_bits(trace_bits.virgin_bits, trace_bits)
    //         .unwrap();
    //     println!("hnb = %{}\n", &hnb);

    //     let mut cp_trace_bits = trace_cover
    //         .bit_map
    //         .write()
    //         .map_err(|_| failure::format_err!("Failed to lock!"))?;

    //     self.classify_counts(&mut cp_trace_bits).unwrap();
    //     let mut virgin_bits = self
    //         .bit_map
    //         .write()
    //         .map_err(|_| failure::format_err!("Failed to lock!"))?;

    //     if hnb != 0 {
    //         println!("new path found !!!\n");
    //         Ok(true)
    //     } else {
    //         println!("opps! No path found !!!\n");
    //         Ok(false)
    //     }
    // }
}
