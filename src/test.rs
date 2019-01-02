extern crate serde_json;

use serde::{ser, de};
use std::collections::HashMap;
use std::fmt;

use super::*;

fn test<T>(input: T)
    where T: fmt::Debug + PartialEq + ser::Serialize + de::DeserializeOwned
{
    let json = serde_json::to_string(&input).unwrap();
    println!("json: {}", json);
    let mut de = serde_json::Deserializer::from_str(&json);
    let pretty = serde_json::to_string_pretty(&Transcoder::new(&mut de)).unwrap();
    println!("pretty: {}", pretty);
    let output: T = serde_json::from_str(&pretty).unwrap();
    println!("output: {:?}", output);
    assert_eq!(input, output);
}

#[test]
fn bool() {
    test(true);
    test(false);
}

#[test]
fn isize() {
    test(isize::min_value());
    test(0isize);
    test(isize::max_value());
}

#[test]
fn i8() {
    test(i8::min_value());
    test(0i8);
    test(i8::max_value());
}

#[test]
fn i16() {
    test(i16::min_value());
    test(0i16);
    test(i16::max_value());
}

#[test]
fn i32() {
    test(i32::min_value());
    test(0i32);
    test(i32::max_value());
}

#[test]
fn i64() {
    test(i64::min_value());
    test(0i64);
    test(i64::max_value());
}

#[test]
fn usize() {
    test(0usize);
    test(u32::max_value() as usize + 1);
}

#[test]
fn u8() {
    test(0u8);
    test(u8::max_value());
}

#[test]
fn u16() {
    test(0u16);
    test(u16::max_value());
}

#[test]
fn u32() {
    test(0u32);
    test(u32::max_value());
}

#[test]
fn u64() {
    test(0u64);
    test(u32::max_value() as u64 + 1);
}

serde_if_integer128! {
    #[test]
    fn i128() {
        // JSON not support too large numbers
        test(i64::min_value() as i128);
        test(0i128);
        // JSON not support too large numbers
        test(i64::max_value() as i128 + 1);
    }

    #[test]
    fn u128() {
        test(0u128);
        // JSON not support too large numbers
        test(u32::max_value() as u128 + 1);
    }
}

#[test]
fn f32() {
    test(1.3f32);
    test(-1e10f32);
}

#[test]
fn f64() {
    test(1.3f64);
    test(-1e10f64);
}

#[test]
fn char() {
    test('a');
    test('\0');
}

#[test]
fn str() {
    test("hello world".to_string());
    test("".to_string());
}

#[test]
fn unit() {
    test(());
}

#[test]
fn none() {
    test(None::<i32>);
}

#[test]
fn some() {
    test(Some(0i32));
    test(Some("hi".to_string()));
}

#[test]
fn newtype_struct() {
    #[derive(PartialEq, Debug)]
    struct Foo(i32);

    impl ser::Serialize for Foo {
        fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where S: ser::Serializer
        {
            s.serialize_newtype_struct("Foo", &self.0)
        }
    }

    impl<'de> de::Deserialize<'de> for Foo {
        fn deserialize<D>(d: D) -> Result<Foo, D::Error>
            where D: de::Deserializer<'de>
        {
            struct V;

            impl<'de> de::Visitor<'de> for V {
                type Value = Foo;

                fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                    write!(fmt, "a Foo struct")
                }

                fn visit_newtype_struct<D>(self, d: D) -> Result<Foo, D::Error>
                    where D: de::Deserializer<'de>
                {
                    Ok(Foo(try!(de::Deserialize::deserialize(d))))
                }
            }

            d.deserialize_newtype_struct("Foo", V)
        }
    }

    test(Foo(100));
}

#[test]
fn seq() {
    test(vec![0, 1, 2, 3]);
}

#[test]
fn map() {
    let mut map = HashMap::new();
    map.insert("hello".to_owned(), vec![1, 2]);
    map.insert("goodbye".to_owned(), vec![]);
    test(map);
}
