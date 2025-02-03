use std::{collections::BTreeMap, fmt};

use combine::easy::{Error, Info};
use combine::{choice, many, many1, optional, position, StdParseResult};
use combine::{parser, Parser};

use crate::helpers::{ident, kind, name, punct};
use crate::position::Pos;
use crate::tokenizer::{Kind as T, Token, TokenStream};

/// Text abstracts over types that hold a string value.
/// It is used to make the AST generic over the string type.
pub trait Text<'a>: 'a {
    type Value: 'a
        + From<&'a str>
        + AsRef<str>
        + std::borrow::Borrow<str>
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + fmt::Debug
        + Clone;
}

impl<'a> Text<'a> for &'a str {
    type Value = Self;
}

impl Text<'_> for String {
    type Value = String;
}

impl<'a> Text<'a> for std::borrow::Cow<'a, str> {
    type Value = Self;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Directive<'a, T: Text<'a>> {
    pub position: Pos,
    pub name: T::Value,
    pub arguments: Vec<(T::Value, Value<'a, T>)>,
}

/// This represents integer number
///
/// But since there is no definition on limit of number in spec
/// (only in implemetation), we do a trick similar to the one
/// in `serde_json`: encapsulate value in new-type, allowing type
/// to be extended later.
#[derive(Debug, Clone, PartialEq)]
// we use i64 as a reference implementation: graphql-js thinks even 32bit
// integers is enough. We might consider lift this limit later though
pub struct Number(pub(crate) i64);

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a, T: Text<'a>> {
    Variable(T::Value),
    Int(Number),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Enum(T::Value),
    List(Vec<Value<'a, T>>),
    Object(BTreeMap<T::Value, Value<'a, T>>),
}

impl<'a, T: Text<'a>> Value<'a, T> {
    pub fn into_static(&self) -> Value<'static, String> {
        match self {
            Self::Variable(v) => Value::Variable(v.as_ref().into()),
            Self::Int(i) => Value::Int(i.clone()),
            Self::Float(f) => Value::Float(*f),
            Self::String(s) => Value::String(s.clone()),
            Self::Boolean(b) => Value::Boolean(*b),
            Self::Null => Value::Null,
            Self::Enum(v) => Value::Enum(v.as_ref().into()),
            Self::List(l) => Value::List(l.iter().map(|e| e.into_static()).collect()),
            Self::Object(o) => Value::Object(
                o.iter()
                    .map(|(k, v)| (k.as_ref().into(), v.into_static()))
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type<'a, T: Text<'a>> {
    NamedType(T::Value),
    ListType(Box<Type<'a, T>>),
    NonNullType(Box<Type<'a, T>>),
}

impl Number {
    /// Returns a number as i64 if it fits the type
    pub fn as_i64(&self) -> Option<i64> {
        Some(self.0)
    }
}

impl From<i32> for Number {
    fn from(i: i32) -> Self {
        Number(i as i64)
    }
}

pub fn directives<'a, T>(
    input: &mut TokenStream<'a>,
) -> StdParseResult<Vec<Directive<'a, T>>, TokenStream<'a>>
where
    T: Text<'a>,
{
    many(
        position()
            .skip(punct("@"))
            .and(name::<'a, T>())
            .and(parser(arguments))
            .map(|((position, name), arguments)| Directive {
                position,
                name,
                arguments,
            }),
    )
    .parse_stream(input)
    .into_result()
}

#[allow(clippy::type_complexity)]
pub fn arguments<'a, T>(
    input: &mut TokenStream<'a>,
) -> StdParseResult<Vec<(T::Value, Value<'a, T>)>, TokenStream<'a>>
where
    T: Text<'a>,
{
    optional(
        punct("(")
            .with(many1(name::<'a, T>().skip(punct(":")).and(parser(value))))
            .skip(punct(")")),
    )
    .map(|opt| opt.unwrap_or_default())
    .parse_stream(input)
    .into_result()
}

pub fn int_value<'a, S>(
    input: &mut TokenStream<'a>,
) -> StdParseResult<Value<'a, S>, TokenStream<'a>>
where
    S: Text<'a>,
{
    kind(T::IntValue)
        .and_then(|tok| tok.value.parse())
        .map(Number)
        .map(Value::Int)
        .parse_stream(input)
        .into_result()
}

pub fn float_value<'a, S>(
    input: &mut TokenStream<'a>,
) -> StdParseResult<Value<'a, S>, TokenStream<'a>>
where
    S: Text<'a>,
{
    kind(T::FloatValue)
        .and_then(|tok| tok.value.parse())
        .map(Value::Float)
        .parse_stream(input)
        .into_result()
}

fn unquote_block_string(src: &str) -> Result<String, Error<Token<'_>, Token<'_>>> {
    debug_assert!(src.starts_with("\"\"\"") && src.ends_with("\"\"\""));
    let lines = src[3..src.len() - 3].lines();

    let mut common_indent = usize::MAX;
    let mut first_non_empty_line: Option<usize> = None;
    let mut last_non_empty_line = 0;
    for (idx, line) in lines.clone().enumerate() {
        let indent = line.len() - line.trim_start().len();
        if indent == line.len() {
            continue;
        }

        first_non_empty_line.get_or_insert(idx);
        last_non_empty_line = idx;

        if idx != 0 {
            common_indent = std::cmp::min(common_indent, indent);
        }
    }

    if first_non_empty_line.is_none() {
        // The block string contains only whitespace.
        return Ok("".to_string());
    }
    let first_non_empty_line = first_non_empty_line.unwrap();

    let mut result = String::with_capacity(src.len() - 6);
    let mut lines = lines
        .enumerate()
        // Skip leading and trailing empty lines.
        .skip(first_non_empty_line)
        .take(last_non_empty_line - first_non_empty_line + 1)
        // Remove indent, except the first line.
        .map(|(idx, line)| {
            if idx != 0 && line.len() >= common_indent {
                &line[common_indent..]
            } else {
                line
            }
        })
        // Handle escaped triple-quote (\""").
        .map(|x| x.replace(r#"\""""#, r#"""""#));

    if let Some(line) = lines.next() {
        result.push_str(&line);

        for line in lines {
            result.push('\n');
            result.push_str(&line);
        }
    }
    Ok(result)
}

fn unquote_string(s: &str) -> Result<String, Error<Token, Token>> {
    let mut res = String::with_capacity(s.len());
    debug_assert!(s.starts_with('"') && s.ends_with('"'));
    let mut chars = s[1..s.len() - 1].chars();
    let mut temp_code_point = String::with_capacity(4);
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                match chars.next().expect("slash cant be at the end") {
                    c @ '"' | c @ '\\' | c @ '/' => res.push(c),
                    'b' => res.push('\u{0010}'),
                    'f' => res.push('\u{000C}'),
                    'n' => res.push('\n'),
                    'r' => res.push('\r'),
                    't' => res.push('\t'),
                    'u' => {
                        temp_code_point.clear();
                        for _ in 0..4 {
                            match chars.next() {
                                Some(inner_c) => temp_code_point.push(inner_c),
                                None => {
                                    return Err(Error::Unexpected(Info::Owned(
                                        format_args!(
                                            "\\u must have 4 characters after it, only found '{}'",
                                            temp_code_point
                                        )
                                        .to_string(),
                                    )))
                                }
                            }
                        }

                        // convert our hex string into a u32, then convert that into a char
                        match u32::from_str_radix(&temp_code_point, 16).map(std::char::from_u32) {
                            Ok(Some(unicode_char)) => res.push(unicode_char),
                            _ => {
                                return Err(Error::Unexpected(Info::Owned(
                                    format_args!(
                                        "{} is not a valid unicode code point",
                                        temp_code_point
                                    )
                                    .to_string(),
                                )))
                            }
                        }
                    }
                    c => {
                        return Err(Error::Unexpected(Info::Owned(
                            format_args!("bad escaped char {:?}", c).to_string(),
                        )));
                    }
                }
            }
            c => res.push(c),
        }
    }

    Ok(res)
}

pub fn string<'a>(input: &mut TokenStream<'a>) -> StdParseResult<String, TokenStream<'a>> {
    choice((
        kind(T::StringValue).and_then(|tok| unquote_string(tok.value)),
        kind(T::BlockString).and_then(|tok| unquote_block_string(tok.value)),
    ))
    .parse_stream(input)
    .into_result()
}

pub fn string_value<'a, S>(
    input: &mut TokenStream<'a>,
) -> StdParseResult<Value<'a, S>, TokenStream<'a>>
where
    S: Text<'a>,
{
    kind(T::StringValue)
        .and_then(|tok| unquote_string(tok.value))
        .map(Value::String)
        .parse_stream(input)
        .into_result()
}

pub fn block_string_value<'a, S>(
    input: &mut TokenStream<'a>,
) -> StdParseResult<Value<'a, S>, TokenStream<'a>>
where
    S: Text<'a>,
{
    kind(T::BlockString)
        .and_then(|tok| unquote_block_string(tok.value))
        .map(Value::String)
        .parse_stream(input)
        .into_result()
}

pub fn plain_value<'a, T>(
    input: &mut TokenStream<'a>,
) -> StdParseResult<Value<'a, T>, TokenStream<'a>>
where
    T: Text<'a>,
{
    ident("true")
        .map(|_| Value::Boolean(true))
        .or(ident("false").map(|_| Value::Boolean(false)))
        .or(ident("null").map(|_| Value::Null))
        .or(name::<'a, T>().map(Value::Enum))
        .or(parser(int_value))
        .or(parser(float_value))
        .or(parser(string_value))
        .or(parser(block_string_value))
        .parse_stream(input)
        .into_result()
}

pub fn value<'a, T>(input: &mut TokenStream<'a>) -> StdParseResult<Value<'a, T>, TokenStream<'a>>
where
    T: Text<'a>,
{
    parser(plain_value)
        .or(punct("$").with(name::<'a, T>()).map(Value::Variable))
        .or(punct("[")
            .with(many(parser(value)))
            .skip(punct("]"))
            .map(Value::List))
        .or(punct("{")
            .with(many(name::<'a, T>().skip(punct(":")).and(parser(value))))
            .skip(punct("}"))
            .map(Value::Object))
        .parse_stream(input)
        .into_result()
}

pub fn default_value<'a, T>(
    input: &mut TokenStream<'a>,
) -> StdParseResult<Value<'a, T>, TokenStream<'a>>
where
    T: Text<'a>,
{
    parser(plain_value)
        .or(punct("[")
            .with(many(parser(default_value)))
            .skip(punct("]"))
            .map(Value::List))
        .or(punct("{")
            .with(many(
                name::<'a, T>().skip(punct(":")).and(parser(default_value)),
            ))
            .skip(punct("}"))
            .map(Value::Object))
        .parse_stream(input)
        .into_result()
}

pub fn parse_type<'a, T>(
    input: &mut TokenStream<'a>,
) -> StdParseResult<Type<'a, T>, TokenStream<'a>>
where
    T: Text<'a>,
{
    name::<'a, T>()
        .map(Type::NamedType)
        .or(punct("[")
            .with(parser(parse_type))
            .skip(punct("]"))
            .map(Box::new)
            .map(Type::ListType))
        .and(optional(punct("!")).map(|v| v.is_some()))
        .map(|(typ, strict)| {
            if strict {
                Type::NonNullType(Box::new(typ))
            } else {
                typ
            }
        })
        .parse_stream(input)
        .into_result()
}

#[cfg(test)]
mod tests {
    use super::unquote_block_string;
    use super::unquote_string;
    use super::Number;

    #[test]
    fn number_from_i32_and_to_i64_conversion() {
        assert_eq!(Number::from(1).as_i64(), Some(1));
        assert_eq!(Number::from(584).as_i64(), Some(584));
        assert_eq!(Number::from(i32::MIN).as_i64(), Some(i32::MIN as i64));
        assert_eq!(Number::from(i32::MAX).as_i64(), Some(i32::MAX as i64));
    }

    #[test]
    fn unquote_unicode_string() {
        // basic tests
        assert_eq!(unquote_string(r#""\u0009""#).expect(""), "\u{0009}");
        assert_eq!(unquote_string(r#""\u000A""#).expect(""), "\u{000A}");
        assert_eq!(unquote_string(r#""\u000D""#).expect(""), "\u{000D}");
        assert_eq!(unquote_string(r#""\u0020""#).expect(""), "\u{0020}");
        assert_eq!(unquote_string(r#""\uFFFF""#).expect(""), "\u{FFFF}");

        // a more complex string
        assert_eq!(
            unquote_string(r#""\u0009 hello \u000A there""#).expect(""),
            "\u{0009} hello \u{000A} there"
        );
    }

    #[test]
    fn block_string_leading_and_trailing_empty_lines() {
        let block = &triple_quote("   \n\n  Hello,\n    World!\n\n  Yours,\n    GraphQL.\n\n\n");
        assert_eq!(
            unquote_block_string(block),
            Result::Ok("Hello,\n  World!\n\nYours,\n  GraphQL.".to_string())
        );
    }

    #[test]
    fn block_string_indent() {
        let block = &triple_quote("Hello   \n\n  Hello,\n    World!\n");
        assert_eq!(
            unquote_block_string(block),
            Result::Ok("Hello   \n\nHello,\n  World!".to_string())
        );
    }

    #[test]
    fn block_string_escaping() {
        let block = triple_quote(r#"\""""#);
        assert_eq!(
            unquote_block_string(&block),
            Result::Ok("\"\"\"".to_string())
        );
    }

    #[test]
    fn block_string_empty() {
        let block = triple_quote("");
        assert_eq!(unquote_block_string(&block), Result::Ok("".to_string()));
        let block = triple_quote("   \n\t\n");
        assert_eq!(unquote_block_string(&block), Result::Ok("".to_string()));
    }

    fn triple_quote(input: &str) -> String {
        format!("\"\"\"{}\"\"\"", input)
    }
}
