extern crate serde_json;
extern crate serde_yaml;

use serde::{Serialize, Deserialize};
use std::fmt;

use super::*;

fn test<T>(input: T)
    where T: fmt::Debug + PartialEq + Serialize + Deserialize
{
    let json = serde_json::to_string(&input).unwrap();
    let de = serde_json::Deserializer::new(json.bytes().map(Ok));
    let yaml = serde_yaml::to_string(&Transcoder::new(de)).unwrap();
    let output: T = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(input, output);
}

#[test]
fn unit() {
    test(());
}

#[test]
fn none() {
    test(None::<i32>);
}
