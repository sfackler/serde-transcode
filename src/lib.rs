extern crate serde;

use serde::{de, ser};
use std::cell::RefCell;

#[cfg(test)]
mod test;

pub struct Transcoder<D>(RefCell<D>);

impl<D> Transcoder<D>
    where D: de::Deserializer
{
    pub fn new(d: D) -> Transcoder<D> {
        Transcoder(RefCell::new(d))
    }
}

impl<D> ser::Serialize for Transcoder<D>
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

    fn visit_none<E>(&mut self) -> Result<(), E>
        where E: de::Error
    {
        self.0.serialize_none().map_err(s2d)
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
