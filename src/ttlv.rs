// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::vec::Vec;
use core::{slice::Iter, str::from_utf8};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use scroll::{Cread, Cwrite, BE};

use crate::util::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Ttlv<'a> {
    tag: u16,
    value: Value<'a>,
}

pub trait Tag: Sized + PartialEq {
    fn from_u16(n: u16) -> Self;
    fn to_u16(&self) -> u16;
}

#[derive(Debug, Clone, FromPrimitive, ToPrimitive)]
enum Type {
    Structure = 0x01,
    Integer,
    LongInteger,
    BigInteger,
    Enumeration,
    Boolean,
    TextString,
    ByteString,
    DateTime,
    Interval,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Structure(Vec<Ttlv<'a>>),
    Integer(i32),
    LongInteger(i64),
    BigInteger(&'a [u8]), // Not fully supported
    Enumeration(u32),
    Boolean(bool),
    TextString(&'a str),
    ByteString(&'a [u8]),
    DateTime(i64), // POSIX Time, as described in IEEE Standard 1003.1 [FIPS202]
    Interval(u32),
}

const START_BYTE: u8 = 0x42;

/// The common error type returned for all TTLV-related failures. Variants can be used for more targetted error-handling.
#[derive(Debug)]
pub enum Error {
    UnsupportedType,
    TypeMismatch,
    ChildNotFound,
    MissingStartByte,
    InsufficientBufferSize,
    CorruptUtf8,
}

impl<'a> Ttlv<'a> {
    pub fn new<T: Tag>(tag: T, value: Value<'a>) -> Self {
        Ttlv {
            tag: tag.to_u16(),
            value,
        }
    }
    pub fn tag<T: Tag>(&self) -> T {
        T::from_u16(self.tag)
    }
    pub fn value<T: TryFromValue<'a>>(&'a self) -> Result<T, Error> {
        T::try_from(&self.value).ok_or(Error::TypeMismatch)
    }
    pub fn child_iter(&self) -> Result<Iter<Ttlv>, Error> {
        if let Value::Structure(val) = &self.value {
            Ok(val.iter())
        } else {
            Err(Error::TypeMismatch)
        }
    }
    pub fn path<T: Tag>(&self, tags: &[T]) -> Result<&Ttlv, Error> {
        self.child_iter()?
            .find(|c| {
                let child_tag: T = c.tag();
                child_tag == tags[0]
            })
            .ok_or(Error::ChildNotFound)
            .and_then(|c| {
                if tags.len() == 1 {
                    Ok(c)
                } else {
                    c.path(&tags[1..])
                }
            })
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, Error> {
        if buf.len() < 16 {
            return Err(Error::InsufficientBufferSize);
        }
        buf.cwrite_with::<u8>(START_BYTE, 0, BE);
        buf.cwrite_with::<u16>(self.tag, 1, BE);
        let (type_, len) = match &self.value {
            Value::Structure(children) => {
                let mut cursor = 8;
                for c in children {
                    cursor += c.encode(&mut buf[cursor..])?;
                }
                (Type::Structure, cursor - 8)
            }
            Value::Integer(val) => {
                buf.cwrite_with::<i32>(*val, 8, BE);
                buf.cwrite_with::<u32>(0, 12, BE);
                (Type::Integer, 4)
            }
            Value::LongInteger(val) => {
                buf.cwrite_with::<i64>(*val, 8, BE);
                (Type::LongInteger, 8)
            }
            // Big Integers are padded with leading sign-extended bytes (which are included in the length).
            Value::BigInteger(_) => return Err(Error::UnsupportedType),
            Value::Enumeration(val) => {
                buf.cwrite_with::<u32>(*val, 8, BE);
                buf.cwrite_with::<u32>(0, 12, BE);
                (Type::Enumeration, 4)
            }
            Value::Boolean(val) => {
                buf.cwrite_with::<u64>(if *val { 1 } else { 0 }, 8, BE);
                (Type::Boolean, 8)
            }
            Value::TextString(val) => {
                buf.write_var(val, 8)?;
                (Type::TextString, val.len())
            }
            Value::ByteString(val) => {
                buf.write_var(val, 8)?;
                (Type::ByteString, val.len())
            }
            Value::DateTime(val) => {
                buf.cwrite_with::<i64>(*val, 8, BE);
                (Type::DateTime, 8)
            }
            Value::Interval(val) => {
                buf.cwrite_with::<u32>(*val, 8, BE);
                buf.cwrite_with::<u32>(0, 12, BE);
                (Type::Interval, 4)
            }
        };
        buf.cwrite_with::<u8>(type_ as u8, 3, BE);
        buf.cwrite_with::<u32>(len as u32, 4, BE);
        Ok(8 + padded_len(len))
    }

    pub fn decode(buf: &'a [u8]) -> Result<(Self, usize), Error> {
        if buf.len() < 8 {
            return Err(Error::InsufficientBufferSize);
        }
        if buf.cread_with::<u8>(0, BE) != START_BYTE {
            return Err(Error::MissingStartByte);
        }

        let tag = buf.cread_with::<u16>(1, BE);
        let type_ = Type::from_u8(buf.cread_with::<u8>(3, BE)).ok_or(Error::UnsupportedType)?;
        let len = buf.cread_with::<u32>(4, BE) as usize;
        let padded_len = padded_len(len);
        if buf.len() < 8 + padded_len {
            return Err(Error::InsufficientBufferSize);
        }

        let value = match type_ {
            Type::Structure => {
                let mut cursor = 8;
                let mut children = Vec::new();
                while let Ok((c, c_len)) = Ttlv::decode(&buf[cursor..8 + len]) {
                    cursor += c_len;
                    children.push(c);
                }
                Value::Structure(children)
            }
            Type::Integer => Value::Integer(buf.cread_with::<i32>(8, BE)),
            Type::LongInteger => Value::LongInteger(buf.cread_with::<i64>(8, BE)),
            Type::BigInteger => Value::BigInteger(&buf[8..8 + len]),
            Type::Enumeration => Value::Enumeration(buf.cread_with::<u32>(8, BE)),
            Type::Boolean => Value::Boolean(buf.cread_with::<u64>(8, BE) != 0),
            Type::TextString => Value::TextString(from_utf8(&buf[8..8 + len])?),
            Type::ByteString => Value::ByteString(&buf[8..8 + len]),
            Type::DateTime => Value::DateTime(buf.cread_with::<i64>(8, BE)),
            Type::Interval => Value::Interval(buf.cread_with::<u32>(8, BE)),
        };
        Ok((Ttlv::new(tag, value), 8 + padded_len))
    }
}
