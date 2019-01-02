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
pub fn transcode<'de, D, S>(d: D, s: S) -> Result<S::Ok, S::Error>
    where D: de::Deserializer<'de>,
          S: ser::Serializer
{
    Transcoder::new(d).serialize(s)
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
pub struct Transcoder<D>(RefCell<Option<D>>);

impl<'de, D> Transcoder<D>
    where D: de::Deserializer<'de>
{
    /// Constructs a new `Transcoder`.
    pub fn new(d: D) -> Transcoder<D> {
        Transcoder(RefCell::new(Some(d)))
    }
}

impl<'de, D> ser::Serialize for Transcoder<D>
    where D: de::Deserializer<'de>
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        self.0.borrow_mut().take().unwrap().deserialize_any(Visitor(s)).map_err(d2s)
    }
}

struct Visitor<S>(S);

impl<'de, S> de::Visitor<'de> for Visitor<S>
    where S: ser::Serializer
{
    type Value = S::Ok;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "any value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_bool(v).map_err(s2d)
    }

    fn visit_i8<E>(self, v: i8) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_i8(v).map_err(s2d)
    }

    fn visit_i16<E>(self, v: i16) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_i16(v).map_err(s2d)
    }

    fn visit_i32<E>(self, v: i32) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_i32(v).map_err(s2d)
    }

    fn visit_i64<E>(self, v: i64) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_i64(v).map_err(s2d)
    }

    fn visit_u8<E>(self, v: u8) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_u8(v).map_err(s2d)
    }

    fn visit_u16<E>(self, v: u16) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_u16(v).map_err(s2d)
    }

    fn visit_u32<E>(self, v: u32) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_u32(v).map_err(s2d)
    }

    fn visit_u64<E>(self, v: u64) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_u64(v).map_err(s2d)
    }

    serde_if_integer128! {
        fn visit_i128<E>(self, v: i128) -> Result<S::Ok, E>
            where E: de::Error
        {
            self.0.serialize_i128(v).map_err(s2d)
        }

        fn visit_u128<E>(self, v: u128) -> Result<S::Ok, E>
            where E: de::Error
        {
            self.0.serialize_u128(v).map_err(s2d)
        }
    }

    fn visit_f32<E>(self, v: f32) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_f32(v).map_err(s2d)
    }

    fn visit_f64<E>(self, v: f64) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_f64(v).map_err(s2d)
    }

    fn visit_char<E>(self, v: char) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_char(v).map_err(s2d)
    }

    fn visit_str<E>(self, v: &str) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_str(v).map_err(s2d)
    }

    fn visit_string<E>(self, v: String) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_str(&v).map_err(s2d)
    }

    fn visit_unit<E>(self) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_unit().map_err(s2d)
    }

    fn visit_none<E>(self) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_none().map_err(s2d)
    }

    fn visit_some<D>(self, d: D) -> Result<S::Ok, D::Error>
        where D: de::Deserializer<'de>
    {
        self.0.serialize_some(&Transcoder::new(d)).map_err(s2d)
    }

    fn visit_newtype_struct<D>(self, d: D) -> Result<S::Ok, D::Error>
        where D: de::Deserializer<'de>
    {
        self.0.serialize_newtype_struct("<unknown>", &Transcoder::new(d)).map_err(s2d)
    }

    fn visit_seq<V>(self, mut v: V) -> Result<S::Ok, V::Error>
        where V: de::SeqAccess<'de>
    {
        let mut s = self.0.serialize_seq(v.size_hint()).map_err(s2d)?;
        while let Some(()) = v.next_element_seed(SeqSeed(&mut s))? {}
        s.end().map_err(s2d)
    }

    fn visit_map<V>(self, mut v: V) -> Result<S::Ok, V::Error>
        where V: de::MapAccess<'de>
    {
        let mut s = self.0.serialize_map(v.size_hint()).map_err(s2d)?;
        while let Some(()) = v.next_key_seed(KeySeed(&mut s))? {
            v.next_value_seed(ValueSeed(&mut s))?;
        }
        s.end().map_err(s2d)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_bytes(v).map_err(s2d)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<S::Ok, E>
        where E: de::Error
    {
        self.0.serialize_bytes(&v).map_err(s2d)
    }
}

struct SeqSeed<'a, S: 'a>(&'a mut S);

impl<'de, 'a, S> de::DeserializeSeed<'de> for SeqSeed<'a, S>
    where S: ser::SerializeSeq
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where D: de::Deserializer<'de>
    {
        self.0.serialize_element(&Transcoder::new(deserializer)).map_err(s2d)
    }
}

struct KeySeed<'a, S: 'a>(&'a mut S);

impl<'de, 'a, S> de::DeserializeSeed<'de> for KeySeed<'a, S>
    where S: ser::SerializeMap
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where D: de::Deserializer<'de>
    {
        self.0.serialize_key(&Transcoder::new(deserializer)).map_err(s2d)
    }
}

struct ValueSeed<'a, S: 'a>(&'a mut S);

impl<'de, 'a, S> de::DeserializeSeed<'de> for ValueSeed<'a, S>
    where S: ser::SerializeMap
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
        where D: de::Deserializer<'de>
    {
        self.0.serialize_value(&Transcoder::new(deserializer)).map_err(s2d)
    }
}

fn d2s<D, S>(d: D) -> S
    where D: de::Error,
          S: ser::Error
{
    S::custom(d.to_string())
}

fn s2d<S, D>(s: S) -> D
    where S: ser::Error,
          D: de::Error
{
    D::custom(s.to_string())
}
