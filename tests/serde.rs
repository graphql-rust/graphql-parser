extern crate graphql_parser;
use std::fs::File;
use std::io::Read;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use graphql_parser::parse_schema;

#[cfg(feature = "serde")]
fn serialize<'a, T: Serialize + Deserialize<'a>>(_: T) {}

#[cfg(feature = "serde")]
#[test]
fn serializable() {
    let mut buf = String::with_capacity(1024);
    let path = format!("tests/schemas/minimal.graphql");
    let mut f = File::open(path).unwrap();
    f.read_to_string(&mut buf).unwrap();
    let ast = parse_schema::<String>(&buf).unwrap().to_owned();
    serialize(ast);
}
