use crate::corpus_handle::{
    ty::{Deserialize, Serialize}, 
};
use rand::Rng; 
// here we want to have a struct that store a integer type and value
//     as_kind!(as_int, checked_as_int, IntType);
#[derive(Copy, Default, Clone, Debug, Deserialize, Serialize)]
pub struct CharType {
    tyid: usize,
    val: char,
    max_val: char,
    min_val: char,
}
impl CharType {
    pub fn new(tyid: usize, val: char, max_val: char, min_val: char) -> Self {
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
    pub fn get_val(&self) -> char {
        self.val
    }
    pub fn get_max_val(&self) -> char {
        self.max_val
    }
    pub fn get_min_val(&self) -> char {
        self.min_val
    }

    pub fn gen_char(&mut self) -> char {
        let mut rng = rand::thread_rng();
        self.val = rng.gen::<char>();
        self.val
    }
} 