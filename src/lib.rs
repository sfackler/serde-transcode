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
//!     let mut deserializer = Deserializer::new(reader.bytes());
//!     let mut serializer = Serializer::pretty(writer);
//!     serde_transcode::transcode(&mut deserializer, &mut serializer).unwrap();
//!     serializer.into_inner().flush().unwrap();
//! }
//! ```
#![warn(missing_docs)]
#![doc(html_root_url="https://sfackler.github.io/serde-transcode/doc/v0.1.1")]

extern crate serde;
extern crate serde_stateful_deserialize;

use serde::{de, ser, Serialize};
use serde_stateful_deserialize::{DeserializeState, MapVisitorExt, SeqVisitorExt};
use std::cell::RefCell;

#[cfg(test)]
mod test;

/// Transcodes from a Serde `Deserializer` to a Serde `Serializer`.
pub fn transcode<D, S>(d: &mut D, s: &mut S) -> Result<(), S::Error>
    where D: de::Deserializer,
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
pub struct Transcoder<'a, D: 'a>(RefCell<&'a mut D>);

impl<'a, D> Transcoder<'a, D>
    where D: de::Deserializer
{
    /// Constructs a new `Transcoder`.
    pub fn new(d: &'a mut D) -> Transcoder<'a, D> {
        Transcoder(RefCell::new(d))
    }
}

impl<'a, D> ser::Serialize for Transcoder<'a, D>
    where D: de::Deserializer
{
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: ser::Serializer
    {
        self.0.borrow_mut().deserialize(Visitor(s)).map_err(d2s)
    }
}

struct Visitor<'a, S: 'a>(&'a mut S);

impl<'a, S> de::Visitor for Visitor<'a, S>
    where S: ser::Serializer
{
    type Value = ();

    fn visit_bool<E>(&mut self, v: bool) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_bool(v).map_err(s2d)
    }

    fn visit_isize<E>(&mut self, v: isize) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_isize(v).map_err(s2d)
    }

    fn visit_i8<E>(&mut self, v: i8) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_i8(v).map_err(s2d)
    }

    fn visit_i16<E>(&mut self, v: i16) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_i16(v).map_err(s2d)
    }

    fn visit_i32<E>(&mut self, v: i32) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_i32(v).map_err(s2d)
    }

    fn visit_i64<E>(&mut self, v: i64) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_i64(v).map_err(s2d)
    }

    fn visit_usize<E>(&mut self, v: usize) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_usize(v).map_err(s2d)
    }

    fn visit_u8<E>(&mut self, v: u8) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_u8(v).map_err(s2d)
    }

    fn visit_u16<E>(&mut self, v: u16) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_u16(v).map_err(s2d)
    }

    fn visit_u32<E>(&mut self, v: u32) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_u32(v).map_err(s2d)
    }

    fn visit_u64<E>(&mut self, v: u64) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_u64(v).map_err(s2d)
    }

    fn visit_f32<E>(&mut self, v: f32) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_f32(v).map_err(s2d)
    }

    fn visit_f64<E>(&mut self, v: f64) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_f64(v).map_err(s2d)
    }

    fn visit_char<E>(&mut self, v: char) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_char(v).map_err(s2d)
    }

    fn visit_str<E>(&mut self, v: &str) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_str(v).map_err(s2d)
    }

    fn visit_string<E>(&mut self, v: String) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_str(&v).map_err(s2d)
    }

    fn visit_unit<E>(&mut self) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_unit().map_err(s2d)
    }

    fn visit_unit_struct<E>(&mut self, name: &'static str) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_unit_struct(name).map_err(s2d)
    }

    fn visit_none<E>(&mut self) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_none().map_err(s2d)
    }

    fn visit_some<D>(&mut self, d: &mut D) -> Result<(), D::Error>
        where D: de::Deserializer
    {
        self.0.serialize_some(&Transcoder::new(d)).map_err(s2d)
    }

    fn visit_newtype_struct<D>(&mut self, d: &mut D) -> Result<(), D::Error>
        where D: de::Deserializer
    {
        self.0.serialize_newtype_struct("<unknown>", &Transcoder::new(d)).map_err(s2d)
    }

    fn visit_seq<V>(&mut self, mut v: V) -> Result<(), V::Error>
        where V: de::SeqVisitor
    {
        let mut state = try!(self.0.serialize_seq(None).map_err(s2d));
        while let Some(()) = try!(v.visit_state(SeqEltState(self.0, &mut state))) {
        }
        try!(v.end());
        self.0.serialize_seq_end(state).map_err(s2d)
    }

    fn visit_map<V>(&mut self, mut v: V) -> Result<(), V::Error>
        where V: de::MapVisitor
    {
        let mut state = try!(self.0.serialize_map(None).map_err(s2d));
        while let Some(()) = try!(v.visit_key_state(MapKeyState(self.0, &mut state))) {
            try!(v.visit_value_state(MapValueState(self.0, &mut state)));
        }
        try!(v.end());
        self.0.serialize_map_end(state).map_err(s2d)
    }

    fn visit_bytes<E>(&mut self, v: &[u8]) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_bytes(v).map_err(s2d)
    }

    fn visit_byte_buf<E>(&mut self, v: Vec<u8>) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_bytes(&v).map_err(s2d)
    }
}

struct SeqEltState<'a, S: 'a>(&'a mut S, &'a mut S::SeqState)
    where S: ser::Serializer;

impl<'a, S> DeserializeState for SeqEltState<'a, S>
    where S: ser::Serializer
{
    type Value = ();

    fn deserialize<D>(self, deserializer: &mut D) -> Result<(), D::Error>
        where D: de::Deserializer
    {
        self.0.serialize_seq_elt(self.1, &Transcoder::new(deserializer)).map_err(s2d)
    }
}

struct MapKeyState<'a, S: 'a>(&'a mut S, &'a mut S::MapState)
    where S: ser::Serializer;

impl<'a, S> DeserializeState for MapKeyState<'a, S>
    where S: ser::Serializer
{
    type Value = ();

    fn deserialize<D>(self, deserializer: &mut D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer
    {
        self.0.serialize_map_key(self.1, &Transcoder::new(deserializer)).map_err(s2d)
    }
}

struct MapValueState<'a, S: 'a>(&'a mut S, &'a mut S::MapState)
    where S: ser::Serializer;

impl<'a, S> DeserializeState for MapValueState<'a, S>
    where S: ser::Serializer
{
    type Value = ();

    fn deserialize<D>(self, deserializer: &mut D) -> Result<Self::Value, D::Error>
        where D: de::Deserializer
    {
        self.0.serialize_map_value(self.1, &Transcoder::new(deserializer)).map_err(s2d)
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
