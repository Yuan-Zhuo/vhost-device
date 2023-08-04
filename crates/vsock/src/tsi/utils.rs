use crate::vhu_vsock::{Error, Result};
use std::mem::size_of;
use vm_memory::{bitmap::BitmapSlice, VolatileSlice};

macro_rules! define_read {
    ($name:ident, $from_bytes:ident, $ty:ty, $size:expr) => {
        pub fn $name<B: BitmapSlice>(slice: &VolatileSlice<B>, offset: usize) -> Result<$ty> {
            assert!(slice.len() >= offset + $size);

            let mut buffer = [0u8; $size];
            slice
                .offset(offset)
                .map_err(|_| Error::InvalidPktBuf)?
                .copy_to(&mut buffer);

            Ok(<$ty>::$from_bytes(buffer))
        }
    };
}

define_read!(read_le_u8, from_le_bytes, u8, 1);
define_read!(read_le_u16, from_le_bytes, u16, 2);
define_read!(read_le_u32, from_le_bytes, u32, 4);
define_read!(read_le_i32, from_le_bytes, i32, 4);
define_read!(read_be_u16, from_be_bytes, u16, 2);

macro_rules! define_write {
    ($name:ident, $to_bytes:ident, $t:ty) => {
        pub fn $name<B: BitmapSlice>(
            slice: &VolatileSlice<B>,
            offset: usize,
            data: $t,
        ) -> Result<u32> {
            let buffer = <$t>::$to_bytes(data);
            slice
                .offset(offset)
                .map_err(|_| Error::InvalidPktBuf)?
                .copy_from(&buffer);

            Ok(size_of::<$t>() as u32)
        }
    };
}

define_write!(write_le_u16, to_le_bytes, u16);
define_write!(write_le_i32, to_le_bytes, i32);
define_write!(write_be_u32, to_be_bytes, u32);
