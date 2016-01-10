pub use self::code::*;
pub use self::ptr::*;

use std::mem::size_of;

pub mod code;
pub mod ptr;

pub fn ptr_width() -> i32 {
    size_of::<*const u8>() as i32
}

pub fn align(value: u32, align: u32) -> u32 {
    ((value + align - 1) / align) * align
}

pub fn align_i32(value: i32, align: i32) -> i32 {
    ((value + align - 1) / align) * align
}

pub fn align_usize(value: usize, align: usize) -> usize {
    ((value + align - 1) / align) * align
}
