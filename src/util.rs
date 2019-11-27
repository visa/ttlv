// SPDX-License-Identifier: MIT OR Apache-2.0

use core::{convert::AsRef, str::Utf8Error};

use num_traits::{FromPrimitive, ToPrimitive};
use scroll::{Cread, BE};

use crate::*;

pub trait WriteVar {
    fn write_var<T: AsRef<[u8]>>(&mut self, data: T, offset: usize) -> Result<(), Error>;
}

impl WriteVar for [u8] {
    fn write_var<T: AsRef<[u8]>>(&mut self, data: T, offset: usize) -> Result<(), Error> {
        let buf = &mut self[offset..];

        let data_len = data.as_ref().len();
        let padded_len = padded_len(data_len);

        if buf.len() < padded_len {
            return Err(Error::InsufficientBufferSize);
        }
        buf[..data_len].copy_from_slice(data.as_ref());
        for pad in &mut buf[data_len..padded_len] {
            *pad = 0;
        }
        Ok(())
    }
}

pub fn parse_ttlv_len(buf: &[u8]) -> usize {
    assert_eq!(4, buf.len());
    let len = buf.cread_with::<i32>(0, BE);
    padded_len(len as usize)
}

pub fn padded_len(len: usize) -> usize {
    (len + 7) / 8 * 8
}

impl<T: FromPrimitive + ToPrimitive + PartialEq> Tag for T {
    fn from_u16(n: u16) -> Self {
        FromPrimitive::from_u16(n).expect("Could not convert from u16")
    }
    fn to_u16(&self) -> u16 {
        ToPrimitive::to_u16(self).expect("Could not convert to u16")
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::CorruptUtf8
    }
}

pub trait TryFromValue<'a>: Sized {
    fn try_from(value: &'a Value) -> Option<Self>;
}
impl<'a> TryFromValue<'a> for i32 {
    fn try_from(value: &'a Value) -> Option<Self> {
        if let Value::Integer(val) = value {
            Some(*val)
        } else {
            None
        }
    }
}
impl<'a> TryFromValue<'a> for i64 {
    fn try_from(value: &'a Value) -> Option<Self> {
        if let Value::LongInteger(val) = value {
            Some(*val)
        } else {
            None
        }
    }
}
impl<'a> TryFromValue<'a> for u32 {
    fn try_from(value: &'a Value) -> Option<Self> {
        if let Value::Enumeration(val) = value {
            Some(*val)
        } else {
            None
        }
    }
}
impl<'a> TryFromValue<'a> for bool {
    fn try_from(value: &'a Value) -> Option<Self> {
        if let Value::Boolean(val) = value {
            Some(*val)
        } else {
            None
        }
    }
}
impl<'a> TryFromValue<'a> for &'a str {
    fn try_from(value: &'a Value) -> Option<Self> {
        if let Value::TextString(val) = value {
            Some(val)
        } else {
            None
        }
    }
}
impl<'a> TryFromValue<'a> for &'a [u8] {
    fn try_from(value: &'a Value) -> Option<Self> {
        if let Value::ByteString(val) = value {
            Some(val)
        } else {
            None
        }
    }
}
