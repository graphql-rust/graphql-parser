use std::marker::PhantomData;

use combine::{Parser, ParseResult, satisfy, StreamOnce};
use combine::error::{Tracked};
use combine::stream::easy::{Error, Errors, Info};

use crate::tokenizer::{TokenStream, Kind, Token};
use crate::position::Pos;

use super::common::{Text};


#[derive(Debug, Clone)]
pub struct TokenMatch<'a> {
    kind: Kind,
    phantom: PhantomData<&'a u8>,
}

#[derive(Debug, Clone)]
pub struct NameMatch<'a, T>
    where T: Text<'a>
{
    phantom: PhantomData<&'a T>,
}

#[derive(Debug, Clone)]
pub struct Value<'a> {
    kind: Kind,
    value: &'static str,
    phantom: PhantomData<&'a u8>,
}


pub fn kind<'x>(kind: Kind) -> TokenMatch<'x> {
    TokenMatch {
        kind: kind,
        phantom: PhantomData,
    }
}

pub fn name<'a, T>() -> NameMatch<'a, T>
    where T: Text<'a>
{
    NameMatch {
        phantom: PhantomData,
    }
}

impl<'a> Parser<TokenStream<'a>> for TokenMatch<'a> {
    type Output = Token<'a>;
    type PartialState = ();

    #[inline]
    fn parse_lazy(&mut self, input: &mut TokenStream<'a>) -> ParseResult<Self::Output, <TokenStream<'a> as StreamOnce>::Error> {
        satisfy(|c: Token<'a>| c.kind == self.kind).parse_lazy(input)
    }

    fn add_error(&mut self, error: &mut Tracked<Errors<Token<'a>, Token<'a>, Pos>>) {
        error.error.add_error(Error::Expected(Info::Owned(
            format!("{:?}", self.kind))));
    }
}

pub fn punct<'s>(value: &'static str) -> Value<'s> {
    Value {
        kind: Kind::Punctuator,
        value: value,
        phantom: PhantomData,
    }
}

pub fn ident<'s>(value: &'static str) -> Value<'s> {
    Value {
        kind: Kind::Name,
        value: value,
        phantom: PhantomData,
    }
}

impl<'a> Parser<TokenStream<'a>> for Value<'a> {
    type Output = Token<'a>;
    type PartialState = ();

    #[inline]
    fn parse_lazy(&mut self, input: &mut TokenStream<'a>) -> ParseResult<Self::Output, <TokenStream<'a> as StreamOnce>::Error> {
        satisfy(|c: Token<'a>| {
            c.kind == self.kind && c.value == self.value
        }).parse_lazy(input)
    }

    fn add_error(&mut self,
        error: &mut Tracked<<TokenStream<'a> as StreamOnce>::Error>)
    {
        error.error.add_error(Error::Expected(Info::Static(self.value)));
    }
}

impl<'a, S> Parser<TokenStream<'a>> for NameMatch<'a, S>
    where S: Text<'a>,
{
    type Output = S::Value;
    type PartialState = ();

    #[inline]
    fn parse_lazy(&mut self, input: &mut TokenStream<'a>) -> ParseResult<Self::Output, <TokenStream<'a> as StreamOnce>::Error> {
        satisfy(|c: Token<'a>| c.kind == Kind::Name)
        .map(|t: Token<'a>| -> S::Value { S::Value::from(t.value) } )
        .parse_lazy(input)
    }

    fn add_error(&mut self,
        error: &mut Tracked<Errors<Token<'a>, Token<'a>, Pos>>)
    {
        error.error.add_error(Error::Expected(Info::Static("Name")));
    }
}
