use crate::corpus_handle::{
    ty::{common::CommonInfo, BinaryFormat, Dir},
    value::{FloatValue, Value},
};
use std::{fmt::Display, ops::RangeInclusive};

/// Integer format.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy, Default)]
pub struct FloatFormat {
    pub fmt: BinaryFormat,
    pub bitfield_off: u64,
    pub bitfield_len: u64,
    pub bitfield_unit: u64,
    pub bitfield_unit_off: u64,
}

macro_rules! float_format_attr_getter {
    () => {
        #[inline(always)]
        pub fn format(&self) -> crate::corpus_handle::ty::BinaryFormat {
            self.float_fmt.fmt
        }

        #[inline(always)]
        pub fn bitfield_off(&self) -> u64 {
            self.float_fmt.bitfield_off
        }

        #[inline(always)]
        pub fn bitfield_len(&self) -> u64 {
            self.float_fmt.bitfield_len
        }

        #[inline(always)]
        pub fn bitfield_unit(&self) -> u64 {
            // self.float_fmt.bitfield_unit
            if self.bitfield_len() != 0 {
                // self.bitfield_unit()
                self.float_fmt.bitfield_unit
            } else {
                self.size()
            }
        }

        // pub fn unit_size(&self) -> u64 {
        //     if self.bitfield_len() != 0 {
        //         self.bitfield_unit()
        //     } else {
        //         self.size()
        //     }
        // }

        #[inline(always)]
        pub fn bitfield_unit_off(&self) -> u64 {
            self.float_fmt.bitfield_unit_off
        }

        pub fn bit_size(&self) -> u64 {
            if let crate::corpus_handle::ty::BinaryFormat::Native
            | crate::corpus_handle::ty::BinaryFormat::BigEndian = self.format()
            {
                if self.bitfield_len() != 0 {
                    self.bitfield_len()
                } else {
                    self.size() * 8
                }
            } else {
                64
            }
        }

        #[inline(always)]
        pub fn is_bitfield(&self) -> bool {
            self.bitfield_len() != 0
        }
    };
}

macro_rules! default_float_value {
    () => {
        #[inline]
        pub fn default_value(&self, dir: Dir) -> Value {
            FloatValue::new(self.id(), dir, self.default_special_value()).into()
        }

        #[inline(always)]
        pub fn is_default(&self, val: &Value) -> bool {
            val.checked_as_float().val == self.default_special_value()
        }
    };
}

#[derive(Debug, Clone)]
pub struct FloatType {
    comm: CommonInfo,
    float_fmt: FloatFormat,
    range: Option<RangeInclusive<u64>>,
    align: u64,
}

impl FloatType {
    common_attr_getter! {}

    float_format_attr_getter! {}

    default_float_value! {}

    #[inline(always)]
    pub fn float_fmt(&self) -> &FloatFormat {
        &self.float_fmt
    }

    #[inline(always)]
    pub fn range(&self) -> Option<&RangeInclusive<u64>> {
        self.range.as_ref()
    }

    #[inline(always)]
    pub fn float_align(&self) -> u64 {
        self.align
    }

    #[inline(always)]
    pub fn default_special_value(&self) -> u64 {
        0
    }
}

impl Display for FloatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "float")?;
        if self.is_bitfield() {
            write!(f, ":{}", self.bitfield_len())?;
        } else if let Some(range) = self.range.as_ref() {
            write!(f, "[{}:{}", range.start(), range.end())?;
            if self.align != 0 {
                write!(f, ", {}", self.align)?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

eq_ord_hash_impl!(FloatType);

#[derive(Debug, Clone)]
pub struct FloatTypeBuilder {
    comm: CommonInfo,
    float_fmt: FloatFormat,
    range: Option<RangeInclusive<u64>>,
    align: u64,
}

impl FloatTypeBuilder {
    #[inline(always)]
    pub fn new(comm: CommonInfo) -> Self {
        Self {
            comm,
            float_fmt: FloatFormat::default(),
            range: None,
            align: 0,
        }
    }

    #[inline(always)]
    pub fn comm(&mut self, comm: CommonInfo) -> &mut Self {
        self.comm = comm;
        self
    }

    #[inline(always)]
    pub fn float_fmt(&mut self, fmt: FloatFormat) -> &mut Self {
        self.float_fmt = fmt;
        self
    }

    #[inline(always)]
    pub fn range(&mut self, range: RangeInclusive<u64>) -> &mut Self {
        self.range = Some(range);
        self
    }

    #[inline(always)]
    pub fn align(&mut self, align: u64) -> &mut Self {
        self.align = align;
        self
    }

    pub fn build(self) -> FloatType {
        FloatType {
            comm: self.comm,
            float_fmt: self.float_fmt,
            range: self.range,
            align: self.align,
        }
    }
}
