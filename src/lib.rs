extern crate serde;

use serde::{de, ser};
use std::cell::RefCell;

#[cfg(test)]
mod test;

pub struct Transcoder<'a, D: 'a>(RefCell<&'a mut D>);

impl<'a, D> Transcoder<'a, D>
    where D: de::Deserializer
{
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
