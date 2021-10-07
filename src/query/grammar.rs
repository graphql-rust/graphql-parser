use combine::{parser, StdParseResult, Parser, many1, eof, optional, position};

use crate::common::{Directive};
use crate::common::{directives, arguments, default_value, parse_type};
use crate::tokenizer::{TokenStream};
use crate::helpers::{punct, ident, name};
use crate::query::error::{ParseError};
use crate::query::ast::*;

pub fn field<'a, S>(input: &mut TokenStream<'a>) -> StdParseResult<Field<'a, S>, TokenStream<'a>>
    where S: Text<'a>
{
    (
        position(),
        name::<'a, S>(),
        optional(punct(":").with(name::<'a, S>())),
        parser(arguments),
        parser(directives),
        optional(parser(selection_set)),
    ).map(|(position, name_or_alias, opt_name, arguments, directives, sel)| {
        let (name, alias) = match opt_name {
            Some(name) => (name, Some(name_or_alias)),
            None => (name_or_alias, None),
        };
        Field {
            position, name, alias, arguments, directives,
            selection_set: sel.unwrap_or_else(|| {
                SelectionSet {
                    span: (position, position),
                    items: Vec::new(),
                }
            }),
        }
    })
    .parse_stream(input).into()
}

pub fn selection<'a, S>(input: &mut TokenStream<'a>) -> StdParseResult<Selection<'a, S>, TokenStream<'a>>
    where S: Text<'a>
{
    parser(field).map(Selection::Field)
    .or(punct("...").with((
                position(),
                optional(ident("on").with(name::<'a, S>()).map(TypeCondition::On)),
                parser(directives),
                parser(selection_set),
            ).map(|(position, type_condition, directives, selection_set)| {
                InlineFragment { position, type_condition,
                                 selection_set, directives }
            })
            .map(Selection::InlineFragment)
        .or((position(),
             name::<'a, S>(),
             parser(directives),
            ).map(|(position, fragment_name, directives)| {
                FragmentSpread { position, fragment_name, directives }
            })
            .map(Selection::FragmentSpread))
    ))
    .parse_stream(input).into()
}

pub fn selection_set<'a, S>(input: &mut TokenStream<'a>) -> StdParseResult<SelectionSet<'a, S>, TokenStream<'a>>
    where S: Text<'a>,
{
    (
        position().skip(punct("{")),
        many1(parser(selection)),
        position().skip(punct("}")),
    ).map(|(start, items, end)| SelectionSet { span: (start, end), items })
    .parse_stream(input).into()
}

pub fn query<'a, T: Text<'a>>(input: &mut TokenStream<'a>) -> StdParseResult<Query<'a, T>, TokenStream<'a>>
    where T: Text<'a>,
{
    position()
    .skip(ident("query"))
    .and(parser(operation_common))
    .map(|(position, (name, variable_definitions, directives, selection_set))|
        Query {
            position, name, selection_set, variable_definitions, directives,
        })
    .parse_stream(input).into()
}

/// A set of attributes common to a Query and a Mutation
#[allow(type_alias_bounds)]
type OperationCommon<'a, T: Text<'a>> = (
    Option<T::Value>,
    Vec<VariableDefinition<'a, T>>,
    Vec<Directive<'a, T>>,
    SelectionSet<'a, T>,
);

pub fn operation_common<'a, T: Text<'a>>(input: &mut TokenStream<'a>) -> StdParseResult<OperationCommon<'a, T>, TokenStream<'a>>
    where T: Text<'a>,
{
    optional(name::<'a, T>())
    .and(optional(
        punct("(")
        .with(many1(
            (
                position(),
                punct("$").with(name::<'a, T>()).skip(punct(":")),
                parser(parse_type),
                optional(
                    punct("=")
                    .with(parser(default_value))),
            ).map(|(position, name, var_type, default_value)| {
                VariableDefinition {
                    position, name, var_type, default_value,
                }
            })))
        .skip(punct(")")))
        .map(|vars| vars.unwrap_or_else(Vec::new)))
    .and(parser(directives))
    .and(parser(selection_set))
    .map(|(((a, b), c), d)| (a, b, c, d))
    .parse_stream(input).into()
}

pub fn mutation<'a, T: Text<'a>>(input: &mut TokenStream<'a>) -> StdParseResult<Mutation<'a, T>, TokenStream<'a>>
    where T: Text<'a>,
{
    position()
    .skip(ident("mutation"))
    .and(parser(operation_common))
    .map(|(position, (name, variable_definitions, directives, selection_set))|
        Mutation {
            position, name, selection_set, variable_definitions, directives,
        })
    .parse_stream(input).into()
}

pub fn subscription<'a, T: Text<'a>>(input: &mut TokenStream<'a>) -> StdParseResult<Subscription<'a, T>, TokenStream<'a>>
    where T: Text<'a>,
{
    position()
    .skip(ident("subscription"))
    .and(parser(operation_common))
    .map(|(position, (name, variable_definitions, directives, selection_set))|
        Subscription {
            position, name, selection_set, variable_definitions, directives,
        })
    .parse_stream(input).into()
}

pub fn operation_definition<'a, S>(input: &mut TokenStream<'a>) -> StdParseResult<OperationDefinition<'a, S>, TokenStream<'a>>
    where S: Text<'a>,
{
    parser(selection_set).map(OperationDefinition::SelectionSet)
    .or(parser(query).map(OperationDefinition::Query))
    .or(parser(mutation).map(OperationDefinition::Mutation))
    .or(parser(subscription).map(OperationDefinition::Subscription))
    .parse_stream(input).into()
}

pub fn fragment_definition<'a, T: Text<'a>>(input: &mut TokenStream<'a>) -> StdParseResult<FragmentDefinition<'a, T>, TokenStream<'a>>
    where T: Text<'a>,
{
    (
        position().skip(ident("fragment")),
        name::<'a, T>(),
        ident("on").with(name::<'a, T>()).map(TypeCondition::On),
        parser(directives),
        parser(selection_set)
    ).map(|(position, name, type_condition, directives, selection_set)| {
        FragmentDefinition {
            position, name, type_condition, directives, selection_set,
        }
    })
    .parse_stream(input).into()
}

pub fn definition<'a, S>(input: &mut TokenStream<'a>) -> StdParseResult<Definition<'a, S>, TokenStream<'a>>
    where S: Text<'a>,
{
    parser(operation_definition).map(Definition::Operation)
    .or(parser(fragment_definition).map(Definition::Fragment))
    .parse_stream(input).into()
}

/// Parses a piece of query language and returns an AST
pub fn parse_query<'a, S>(s: &'a str) -> Result<Document<'a, S>, ParseError>
    where S: Text<'a>,
{
    let mut tokens = TokenStream::new(s);
    match many1(parser(definition)).map(|d| Document { definitions: d }) .skip(eof()) .parse_stream(&mut tokens) {
        combine::ParseResult::CommitOk(doc) => Ok(doc),
        combine::ParseResult::PeekOk(doc) => Ok(doc),
        combine::ParseResult::CommitErr(err) => Err(err.into()),
        combine::ParseResult::PeekErr(err) => Err(err.error.into()),
    }
}

/// Parses a single ExecutableDefinition and returns an AST as well as the
/// remainder of the input which is unparsed
pub fn consume_definition<'a, S>(s: &'a str) -> Result<(Definition<'a, S>, &'a str), ParseError> where S: Text<'a> {
    let tokens = TokenStream::new(s);
    let (doc, tokens) = parser(definition).parse(tokens)?;

    Ok((doc, &s[tokens.offset()..]))
}

#[cfg(test)]
mod test {
    use crate::position::Pos;
    use crate::query::grammar::*;
    use super::{parse_query, consume_definition};

    fn ast(s: &str) -> Document<String> {
        parse_query::<String>(&s).unwrap().to_owned()
    }

    #[test]
    fn one_field() {
        assert_eq!(ast("{ a }"), Document {
            definitions: vec![
                Definition::Operation(OperationDefinition::SelectionSet(
                    SelectionSet {
                        span: (Pos { line: 1, column: 1 },
                               Pos { line: 1, column: 5 }),
                        items: vec![
                            Selection::Field(Field {
                                position: Pos { line: 1, column: 3 },
                                alias: None,
                                name: "a".into(),
                                arguments: Vec::new(),
                                directives: Vec::new(),
                                selection_set: SelectionSet {
                                    span: (Pos { line: 1, column: 3 },
                                           Pos { line: 1, column: 3 }),
                                    items: Vec::new()
                                },
                            }),
                        ],
                    }
                ))
            ],
        });
    }

    #[test]
    fn builtin_values() {
        assert_eq!(ast("{ a(t: true, f: false, n: null) }"),
            Document {
                definitions: vec![
                    Definition::Operation(OperationDefinition::SelectionSet(
                        SelectionSet {
                            span: (Pos { line: 1, column: 1 },
                                   Pos { line: 1, column: 33 }),
                            items: vec![
                                Selection::Field(Field {
                                    position: Pos { line: 1, column: 3 },
                                    alias: None,
                                    name: "a".into(),
                                    arguments: vec![
                                        ("t".into(),
                                            Value::Boolean(true)),
                                        ("f".into(),
                                            Value::Boolean(false)),
                                        ("n".into(),
                                            Value::Null),
                                    ],
                                    directives: Vec::new(),
                                    selection_set: SelectionSet {
                                        span: (Pos { line: 1, column: 3 },
                                               Pos { line: 1, column: 3 }),
                                        items: Vec::new()
                                    },
                                }),
                            ],
                        }
                    ))
                ],
            });
    }

    #[test]
    fn one_field_roundtrip() {
        assert_eq!(ast("{ a }").to_string(), "{\n  a\n}\n");
    }

    #[test]
    #[should_panic(expected="number too large")]
    fn large_integer() {
        ast("{ a(x: 10000000000000000000000000000 }");
    }

    #[test]
    fn consume_single_query() {
        let (query, remainder) = consume_definition::<String>("query { a } query { b }").unwrap();
        assert!(matches!(query, Definition::Operation(_)));
        assert_eq!(remainder, "query { b }");
    }

    #[test]
    fn consume_full_text() {
        let (query, remainder) = consume_definition::<String>("query { a }").unwrap();
        assert!(matches!(query, Definition::Operation(_)));
        assert_eq!(remainder, "");
    }

    #[test]
    fn consume_single_query_preceding_non_graphql() {
        let (query, remainder) =
            consume_definition::<String>("query { a } where a > 1 => 10.0").unwrap();
        assert!(matches!(query, Definition::Operation(_)));
        assert_eq!(remainder, "where a > 1 => 10.0");
    }

    #[test]
    fn consume_fails_without_operation() {
        let err = consume_definition::<String>("where a > 1 => 10.0")
            .expect_err("Expected parse to fail with an error");
        let err = format!("{}", err);
        assert_eq!(err, "query parse error: Parse error at 1:1\nUnexpected `where[Name]`\nExpected `{`, `query`, `mutation`, `subscription` or `fragment`\n");
    }

    #[test]
    fn recursion_too_deep() {
        let query = format!("{}(b: {}{}){}", "{ a".repeat(30), "[".repeat(25), "]".repeat(25),  "}".repeat(30));
        let result = parse_query::<&str>(&query);
        let err = format!("{}", result.unwrap_err());
        assert_eq!(&err, "query parse error: Parse error at 1:114\nExpected `]`\nRecursion limit exceeded\n")
    }
}
