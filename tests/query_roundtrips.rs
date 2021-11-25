extern crate graphql_parser;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use std::fs::File;
use std::io::Read;

use graphql_parser::{parse_query, Style};

fn roundtrip_multiline_args(filename: &str) {
    roundtrip(filename, Style::default().multiline_arguments(true))
}

fn roundtrip_default(filename: &str) {
    roundtrip(filename, &Style::default())
}

fn roundtrip(filename: &str, style: &Style) {
    let mut buf = String::with_capacity(1024);
    let path = format!("tests/queries/{}.graphql", filename);
    let mut f = File::open(&path).unwrap();
    f.read_to_string(&mut buf).unwrap();
    let ast = parse_query::<String>(&buf).unwrap().to_owned();
    assert_eq!(ast.format(style), buf);
}

fn roundtrip2(filename: &str) {
    let mut buf = String::with_capacity(1024);
    let source = format!("tests/queries/{}.graphql", filename);
    let target = format!("tests/queries/{}_canonical.graphql", filename);
    let mut f = File::open(&source).unwrap();
    f.read_to_string(&mut buf).unwrap();
    let ast = parse_query::<String>(&buf).unwrap().to_owned();

    let mut buf = String::with_capacity(1024);
    let mut f = File::open(&target).unwrap();
    f.read_to_string(&mut buf).unwrap();
    assert_eq!(ast.to_string(), buf);
}

#[test]
fn minimal() {
    roundtrip_default("minimal");
}
#[test]
fn minimal_query() {
    roundtrip_default("minimal_query");
}
#[test]
fn named_query() {
    roundtrip_default("named_query");
}
#[test]
fn query_vars() {
    roundtrip_default("query_vars");
}
#[test]
fn query_nameless_vars() {
    roundtrip_default("query_nameless_vars");
}
#[test]
fn query_nameless_vars_multiple_fields() {
    roundtrip2("query_nameless_vars_multiple_fields");
}
#[test]
fn query_var_defaults() {
    roundtrip_default("query_var_defaults");
}
#[test]
fn query_var_defaults1() {
    roundtrip_default("query_var_default_string");
}
#[test]
fn query_var_defaults2() {
    roundtrip_default("query_var_default_float");
}
#[test]
fn query_var_defaults3() {
    roundtrip_default("query_var_default_list");
}
#[test]
fn query_var_defaults4() {
    roundtrip_default("query_var_default_object");
}
#[test]
fn query_aliases() {
    roundtrip_default("query_aliases");
}
#[test]
fn query_arguments() {
    roundtrip_default("query_arguments");
}
#[test]
fn query_arguments_multiline() {
    roundtrip_multiline_args("query_arguments_multiline");
}
#[test]
fn query_directive() {
    roundtrip_default("query_directive");
}
#[test]
fn mutation_directive() {
    roundtrip_default("mutation_directive");
}
#[test]
fn mutation_nameless_vars() {
    roundtrip_default("mutation_nameless_vars");
}
#[test]
fn subscription_directive() {
    roundtrip_default("subscription_directive");
}
#[test]
fn string_literal() {
    roundtrip_default("string_literal");
}
#[test]
fn triple_quoted_literal() {
    roundtrip_default("triple_quoted_literal");
}
#[test]
fn query_list_arg() {
    roundtrip_default("query_list_argument");
}
#[test]
fn query_object_arg() {
    roundtrip_default("query_object_argument");
}
#[test]
fn query_object_arg_multiline() {
    roundtrip_multiline_args("query_object_argument_multiline");
}
#[test]
fn query_array_arg_multiline() {
    roundtrip_multiline_args("query_array_argument_multiline");
}
#[test]
fn nested_selection() {
    roundtrip_default("nested_selection");
}
#[test]
fn inline_fragment() {
    roundtrip_default("inline_fragment");
}
#[test]
fn inline_fragment_dir() {
    roundtrip_default("inline_fragment_dir");
}
#[test]
fn fragment_spread() {
    roundtrip_default("fragment_spread");
}
#[test]
fn minimal_mutation() {
    roundtrip_default("minimal_mutation");
}
#[test]
fn fragment() {
    roundtrip_default("fragment");
}
#[test]
fn directive_args() {
    roundtrip_default("directive_args");
}
#[test]
fn directive_args_multiline() {
    roundtrip_multiline_args("directive_args_multiline");
}
#[test]
fn kitchen_sink() {
    roundtrip2("kitchen-sink");
}
