// use rand::Rng;
// use std::process::Command;

use serde::Deserialize;
use serde::Serialize;

pub use super::character::*;
pub use super::double::*;
pub use super::integer::*;
use super::TYPE;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ArrayElement {
    Int(u64),
    Float(f64),
    Char(char),
    // Add other types as needed
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArrayType {
    tyid: usize,
    inner_type: TYPE,
    len: u64,
    ptr: Box<[ArrayElement]>,
    int_array: Vec<IntType>,
    float_array: Vec<DoubleType>,
    char_array: Vec<CharType>,
    bool_array: Vec<BoolType>, 
}
impl ArrayType {
    pub fn new(tyid: usize, inner_type: TYPE, len: u64) -> Self {
        Self {
            tyid,
            inner_type,
            len,
            ptr: Box::new([]),
            int_array: Vec::new(),
            float_array: Vec::new(),
            char_array: Vec::new(),
            bool_array: Vec::new(), 
        }
    }
    pub fn get_tyid(&self) -> usize {
        self.tyid
    }

    pub fn get_array_box(&mut self) -> Box<[ArrayElement]> {
        let mut elements = Vec::new();
        match self.inner_type {
            TYPE::Int8 | TYPE::Int16 | TYPE::Int32 | TYPE::Int64
            | TYPE::UInt8 | TYPE::UInt16 | TYPE::UInt32 | TYPE::UInt64 => {
                for int in &self.int_array {
                    // Assuming IntType has a method to get its value as i32
                    elements.push(ArrayElement::Int(int.get_val()));
                }
            },
            TYPE::Float => {
                for float in &self.float_array {
                    // Assuming DoubleType has a method to get its value as f64
                    elements.push(ArrayElement::Float(float.get_val()));
                }
            },
            TYPE::Char => {
                for char in &self.char_array {
                    // Assuming CharType has a method to get its value as char
                    elements.push(ArrayElement::Char(char.get_val()));
                }
            },
            // Handle other types as needed
            _ => panic!("Unsupported array type"),
        }
        self.ptr = elements.into_boxed_slice();
        self.ptr.clone()

    }
 
    pub fn gen_array(&mut self) -> Result<(), failure::Error> {
        // generate based on pad
        match self.inner_type {
            TYPE::Char => {
                for _ in 0..self.len {
                    let mut val = CharType::new(0, 0 as char, char::MAX, 0 as char);
                    val.gen_char();
                    self.char_array.push(val);
                }
            }
            TYPE::Float => {
                for _ in 0..self.len {
                    let mut val =
                        DoubleType::new(0, 0 as f64, f32::MAX as f64, f32::MIN as f64, 32);
                    val.gen_double();
                    self.float_array.push(val);
                }
            }
            TYPE::Int8 => {
                for _ in 0..self.len {
                    let mut val = IntType::new(0, 0 as u64, i8::MAX as u64, i8::MIN as u64, 8);
                    val.gen_integer();
                    self.int_array.push(val);
                }
            }
            TYPE::Int16 => {
                for _ in 0..self.len {
                    let mut val = IntType::new(0, 0 as u64, i16::MAX as u64, i16::MIN as u64, 16);
                    val.gen_integer();
                    self.int_array.push(val);
                }
            }
            TYPE::Int32 => {
                for _ in 0..self.len {
                    let mut val = IntType::new(0, 0 as u64, i32::MAX as u64, i32::MIN as u64, 32);
                    val.gen_integer();
                    self.int_array.push(val);
                }
            }
            TYPE::Int64 => {
                for _ in 0..self.len {
                    let mut val = IntType::new(0, 0 as u64, i64::MAX as u64, i64::MIN as u64, 64);
                    val.gen_integer();
                    self.int_array.push(val);
                }
            }
            TYPE::UInt8 => {
                for _ in 0..self.len {
                    let mut val = IntType::new(0, 0 as u64, u8::MAX as u64, u8::MIN as u64, 8);
                    val.gen_integer();
                    self.int_array.push(val);
                }
            }
            TYPE::UInt16 => {
                for _ in 0..self.len {
                    let mut val = IntType::new(0, 0 as u64, u16::MAX as u64, u16::MIN as u64, 16);
                    val.gen_integer();
                    self.int_array.push(val);
                }
            }
            TYPE::UInt32 => {
                for _ in 0..self.len {
                    let mut val = IntType::new(0, 0 as u64, u32::MAX as u64, u32::MIN as u64, 32);
                    val.gen_integer();
                    self.int_array.push(val);
                }
            }
            TYPE::UInt64 => {
                for _ in 0..self.len {
                    let mut val = IntType::new(0, 0 as u64, u64::MAX, u64::MIN, 64);
                    val.gen_integer();
                    self.int_array.push(val);
                }
            }
            TYPE::Bool => {
                for _ in 0..self.len {
                    let val = BoolType::new(0, 0 as u64, 1 as u64, 0 as u64);
                    val.gen_bool();
                    self.bool_array.push(val);
                }
            }
            _ => {
                panic!("Not supported type");
            }
        }
        Ok(())
    }
}
