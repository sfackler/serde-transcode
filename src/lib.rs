//! Transcode from one Serde format to another.
//!
//! This crate provides functionality to "transcode" from an arbitrary Serde
//! `Deserializer` to an arbitrary Serde `Serializer` without needing to
//! collect the entire input into an intermediate form in memory. For example,
//! you could translate a stream of JSON data into a stream of CBOR data, or
//! translate JSON into its pretty-printed form.
//!
//! # Examples
//!
//! Translate a JSON file to a pretty-printed version.
//!
//! ```no_run
//! extern crate serde;
//! extern crate serde_json;
//! extern crate serde_transcode;
//!
//! use serde::Serialize;
//! use serde_json::{Serializer, Deserializer};
//! use std::io::{Read, Write, BufReader, BufWriter};
//! use std::fs::File;
//!
//! fn main() {
//!     let reader = BufReader::new(File::open("input.json").unwrap());
//!     let writer = BufWriter::new(File::create("output.json").unwrap());
//!
//!     let mut deserializer = Deserializer::from_reader(reader);
//!     let mut serializer = Serializer::pretty(writer);
//!     serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();
//!     serializer.into_inner().flush().unwrap();
//! }
//! ```
#![warn(missing_docs)]
#![doc(html_root_url="https://docs.rs/serde-transcode/1.0.1")]

#[macro_use]
extern crate serde;

use serde::de;
use serde::ser::{self, Serialize, SerializeSeq, SerializeMap};
use std::cell::RefCell;
use std::fmt;

#[cfg(test)]
mod test;

/// Transcodes from a Serde `Deserializer` to a Serde `Serializer`.
pub fn transcode<'de, D, S>(d: D, s: S) -> Result<S::Ok, Error<D::Error, S::Error>>
    where D: de::Deserializer<'de>,
          S: ser::Serializer
{
    let transcoder = Transcoder::new(d);
    let result = transcoder.serialize(s);
    match transcoder.take_last_deserializer_error() {
        Some(de_err) => Err(Error::DeserializerError(de_err)),
        None => match result {
            Ok(v) => Ok(v),
            Err(ser_err) => Err(Error::SerializerError(ser_err)),
        },
    }
}

/// A wrapper for errors that can occur while transcoding.
#[derive(Debug)]
pub enum Error<D, S>
    where D: de::Error,
          S: ser::Error
{
    /// Error from the `Deserializer` side.
    DeserializerError(D),
    /// Error from the `Serializer` side.
    SerializerError(S),
}

impl<D, S> fmt::Display for Error<D, S>
    where D: de::Error,
          S: ser::Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializerError(e) => fmt::Display::fmt(&e, f),
            Self::SerializerError(e) => fmt::Display::fmt(&e, f),
        }
    }
}

impl<D, S> std::error::Error for Error<D, S>
    where D: de::Error,
          S: ser::Error
{
}

/// A Serde transcoder.
///
/// In most cases, the `transcode` function should be used instead of this
/// type.
///
/// # Note
///
/// Unlike traditional serializable types, `Transcoder`'s `Serialize`
/// implementation is *not* idempotent, as it advances the state of its
/// internal `Deserializer`. It should only ever be serialized once.
pub struct Transcoder<'de, D>
    where D: de::Deserializer<'de>
{
    de: RefCell<Option<D>>,
    de_err: RefCell<Option<D::Error>>,
}

impl<'de, D> Transcoder<'de, D>
    where D: de::Deserializer<'de>
{
    /// Constructs a new `Transcoder`.
    pub fn new(de: D) -> Transcoder<'de, D> {
        Transcoder {
            de: RefCell::new(Some(de)),
            de_err: RefCell::new(None),
        }
    }

    /// Returns the last error that has occured on the `Deserializer` side
    /// while transcoding. Errors on the `Serializer` side are returned by the
    /// `Serialize` implementation of the `Transcoder`.
    ///
    /// # Note
    ///
    /// This function should only be called once - right after the `Transcoder`
    /// has been serialized, because it removes the error stored internally.
    pub fn take_last_deserializer_error(&self) -> Option<D::Error> {
        self.de_err.borrow_mut().take()
    }
}

impl<'de, D> ser::Serialize for Transcoder<'de, D>
    where D: de::Deserializer<'de>
{
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        let result = self
            .de
            .borrow_mut()
            .take()
            .expect("Transcoder may only be serialized once")
            .deserialize_any(Visitor(ser));
        match result {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(ser_err)) => Err(ser_err),
            Err(de_err) => {
                let ser_err = <S::Error as ser::Error>::custom(de_err.to_string());
                *self.de_err.borrow_mut() = Some(de_err);
                Err(ser_err)
            }
        }
    }
}

struct Visitor<S>(S);

macro_rules! try_serializer {
    ($expr:expr) => {
        match $expr {
            Ok(v) => Ok(Ok(v)),
            Err(ser_err) => return Ok(Err(ser_err)),
        }
    };
}

macro_rules! with_transcoder {
    ($deserializer:expr, |$transcoder:ident| $body:expr) => {{
        let $transcoder = Transcoder::new($deserializer);
        let result = $body;
        match $transcoder.take_last_deserializer_error() {
            Some(de_err) => Err(de_err),
            None => match result {
                Ok(v) => Ok(Ok(v)),
                Err(ser_err) => Ok(Err(ser_err)),
            },
        }
    }};
}

impl<'de, S> de::Visitor<'de> for Visitor<S>
    where S: ser::Serializer
{
    type Value = Result<S::Ok, S::Error>;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "any value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_bool(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_i8(v))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_i16(v))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_i32(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_i64(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_u8(v))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_u16(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_u32(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_u64(v))
    }

    serde_if_integer128! {
        fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where E: de::Error
        {
            try_serializer!(self.0.serialize_i128(v))
        }

        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where E: de::Error
        {
            try_serializer!(self.0.serialize_u128(v))
        }
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_f32(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_f64(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_char(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_str(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_str(&v))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_unit())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_none())
    }

    fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer<'de>
    {
        with_transcoder!(d, |tr| self.0.serialize_some(&tr))
    }

    fn visit_newtype_struct<D>(self, d: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer<'de>
    {
        with_transcoder!(d, |tr| self.0.serialize_newtype_struct("<unknown>", &tr))
    }

    fn visit_seq<V>(self, mut v: V) -> Result<Self::Value, V::Error>
        where V: de::SeqAccess<'de>
    {
        let mut s = match self.0.serialize_seq(v.size_hint()) {
            Ok(v) => v,
            Err(ser_err) => return Ok(Err(ser_err)),
        };
        while let Some(result) = v.next_element_seed(SeqSeed(&mut s))? {
            let _: Result<Result<(), S::Error>, V::Error> = try_serializer!(result);
        }
        try_serializer!(s.end())
    }

    fn visit_map<V>(self, mut v: V) -> Result<Self::Value, V::Error>
        where V: de::MapAccess<'de>
    {
        let mut s = match self.0.serialize_map(v.size_hint()) {
            Ok(v) => v,
            Err(ser_err) => return Ok(Err(ser_err)),
        };
        while let Some(result) = v.next_key_seed(KeySeed(&mut s))? {
            let _: Result<Result<(), S::Error>, V::Error> = try_serializer!(result);
            let result = v.next_value_seed(ValueSeed(&mut s))?;
            let _: Result<Result<(), S::Error>, V::Error> = try_serializer!(result);
        }
        try_serializer!(s.end())
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_bytes(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where E: de::Error
    {
        try_serializer!(self.0.serialize_bytes(&v))
    }
}

struct SeqSeed<'a, S: 'a>(&'a mut S);

impl<'de, 'a, S> de::DeserializeSeed<'de> for SeqSeed<'a, S>
    where S: ser::SerializeSeq
{
    type Value = Result<(), S::Error>;

    fn deserialize<D>(self, d: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer<'de>
    {
        with_transcoder!(d, |tr| self.0.serialize_element(&tr))
    }
}

struct KeySeed<'a, S: 'a>(&'a mut S);

impl<'de, 'a, S> de::DeserializeSeed<'de> for KeySeed<'a, S>
    where S: ser::SerializeMap
{
    type Value = Result<(), S::Error>;

    fn deserialize<D>(self, d: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer<'de>
    {
        with_transcoder!(d, |tr| self.0.serialize_key(&tr))
    }
}

struct ValueSeed<'a, S: 'a>(&'a mut S);

impl<'de, 'a, S> de::DeserializeSeed<'de> for ValueSeed<'a, S>
    where S: ser::SerializeMap
{
    type Value = Result<(), S::Error>;

    fn deserialize<D>(self, d: D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer<'de>
    {
        with_transcoder!(d, |tr| self.0.serialize_value(&tr))
    }
}
