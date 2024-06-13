use rand::Rng;
use serde::{Deserialize, Serialize};

// here we want to have a struct that store a integer type and value
//     as_kind!(as_int, checked_as_int, IntType);
#[derive(Copy, Default, Clone, Debug, Deserialize, Serialize)]
pub struct IntType {
    tyid: usize,
    val: u64,
    max_val: u64,
    min_val: u64,
    pad: i32,
}
impl IntType {
    pub fn new(tyid: usize, val: u64, max_val: u64, min_val: u64, pad: i32) -> Self {
        Self {
            tyid,
            val,
            max_val,
            min_val,
            pad,
        }
    }
    pub fn get_tyid(&self) -> usize {
        self.tyid
    }
    pub fn get_val(&self) -> u64 {
        match self.pad {
            -8 => return self.val as u64,
            -16 => return self.val as u64,
            -32 => return self.val as u64,
            -64 => return self.val as u64,
            8 => return self.val as u64,
            16 => return self.val as u64,
            32 => return self.val as u64,
            64 => return self.val as u64,
            _ => {
                panic!("unsupport int type!");
            }
        }
    }
    pub fn get_max_val(&self) -> u64 {
        self.max_val
    }
    pub fn get_min_val(&self) -> u64 {
        self.min_val
    }
    pub fn get_pad(&self) -> i32 {
        self.pad
    }

    pub fn gen_i16_string() -> String {
        let mut rng = rand::thread_rng();
        let val = rng.gen::<i16>() as i16;
        val.to_string()
    }

    pub fn gen_integer(&mut self) -> u64 {
        let mut rng = rand::thread_rng();

        // generate val based on pad value:
        match self.pad {
            -8 => {
                // generate a u8 value
                self.val = rng.gen::<i8>() as u64;
                self.val
            }
            -16 => {
                self.val = rng.gen::<i16>() as u64;
                self.val
            }
            -32 => {
                self.val = rng.gen::<i32>() as u64;
                self.val
            }
            -64 => {
                self.val = rng.gen::<i64>() as u64;
                self.val
            }
            8 => {
                // generate a u8 value
                self.val = rng.gen::<u8>() as u64;
                self.val
            }
            16 => {
                self.val = rng.gen::<u16>() as u64;
                self.val
            }
            32 => {
                self.val = rng.gen::<u32>() as u64;
                self.val
            }
            64 => {
                self.val = rng.gen::<u64>() as u64;
                self.val
            }
            _ => {
                panic!("unsupport int pad");
            }
        }
    }
}

#[derive(Copy, Default, Clone, Debug, Deserialize, Serialize)]
pub struct BoolType {
    tyid: usize,
    val: u64,
    max_val: u64,
    min_val: u64,
}
impl BoolType {
    pub fn new(tyid: usize, val: u64, max_val: u64, min_val: u64) -> Self {
        Self {
            tyid,
            val,
            max_val,
            min_val,
        }
    }
    pub fn get_tyid(&self) -> usize {
        self.tyid
    }
    pub fn get_val(&self) -> i32 {
        match self.val {
            0 => 0,
            1 => 1,
            _ => panic!("unsupport bool type"),
        }
    }
    pub fn get_max_val(&self) -> u64 {
        self.max_val
    }
    pub fn get_min_val(&self) -> u64 {
        self.min_val
    }

    pub fn gen_bool(&self) -> u64 {
        let mut rng = rand::thread_rng();
        let val = rng.gen_range(0..2);
        val
    }
}
