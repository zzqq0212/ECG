use ahash::{AHashMap, AHashSet};

#[macro_use]
pub mod verbose;
pub mod alloc;
pub mod context;
pub mod corpus;
pub mod gen;
pub mod len;
pub mod mutation;
pub mod parse;
pub mod prog;
pub mod relation;
pub mod select;
pub mod serialization;
pub mod syscall;
pub mod target;
pub mod ty;
pub mod value;

pub const IN_SHM_SZ: usize = 1 << 16;
pub type HashMap<K, V> = AHashMap<K, V>;
pub type HashSet<V> = AHashSet<V>;
pub type RngType = rand::rngs::SmallRng;
pub const IN_MAGIC: u64 = 0xBADC0FFEEBADFACE;
