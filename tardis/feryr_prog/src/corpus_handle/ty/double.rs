use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Copy, Default, Clone, Debug, Deserialize, Serialize)]
pub struct DoubleType {
    tyid: usize,
    val: f64,
    max_val: f64,
    min_val: f64,
    pad: i32,
}
impl DoubleType {
    pub fn new(tyid: usize, val: f64, max_val: f64, min_val: f64, pad: i32) -> Self {
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
    pub fn get_val(&self) -> f64 {
        match self.pad {
            32 => return self.val as f64,
            64 => return self.val,
            _ => {
                panic!("unsupport double type!");
            }
        }
    }
    pub fn get_max_val(&self) -> f64 {
        self.max_val
    }
    pub fn get_min_val(&self) -> f64 {
        self.min_val
    }

    pub fn gen_double(&mut self) -> f64 {
        // generate based on pad
        let mut rng = rand::thread_rng();
        match self.pad {
            32 => {
                self.val = rng.gen::<f32>() as f64;
                self.val
            }
            64 => {
                self.val = rng.gen::<f64>();
                self.val
            }
            _ => {
                panic!("unsupport pad");
            }
        }
    }
}
