#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

// pub mod c_cover {
//     extern "C" {
//         pub fn hello();
//         pub fn count_branch(virgin_bits: *mut u8) -> u32;
//         pub fn classify_counts(mem: *mut u32);
//         pub fn has_new_bits(virgin_map: &mut Box<[u8]>, trace_bit: &mut Box<[u8]>) -> u8;
//         pub fn new_path(virgin_bits: &mut Box<[u8]>, trace_bits: &mut Box<[u8]>) -> u32;
//     }
// }
