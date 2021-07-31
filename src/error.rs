use core::fmt;

pub use combine::easy::{Error, Errors, Info};

use crate::position::Pos;
use crate::tokenizer::{OwnedToken, Token};

pub type BorrowedParseError<'a> = Errors<Token<'a>, Token<'a>, Pos>;

#[derive(Debug, PartialEq)]
pub struct ParseError {
    /// The position where the error occurred
    pub position: Pos,
    /// A vector containing specific information on what errors occurred at `position`. Usually
    /// a fully formed message contains one `Unexpected` error and one or more `Expected` errors.
    /// `Message` and `Other` may also appear (`combine` never generates these errors on its own)
    /// and may warrant custom handling.
    pub errors: Vec<Error<OwnedToken, OwnedToken>>,
}

impl<'a> From<Errors<Token<'a>, Token<'a>, Pos>> for ParseError {
    fn from(errors: Errors<Token<'a>, Token<'a>, Pos>) -> Self {
        ParseError {
            position: errors.position,
            errors: errors
                .errors
                .into_iter()
                .map(|e| match e {
                    Error::Unexpected(e) => Error::Unexpected(convert_token(e)),
                    Error::Expected(e) => Error::Expected(convert_token(e)),
                    Error::Message(e) => Error::Message(convert_token(e)),
                    Error::Other(e) => Error::Other(e),
                })
                .collect(),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        "parse error"
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Parse error at {}", self.position)?;
        Error::fmt_errors(&self.errors, f)
    }
}

fn convert_token(info: Info<Token, Token>) -> Info<OwnedToken, OwnedToken> {
    match info {
        Info::Token(token) => Info::Token(token.into()),
        Info::Range(token) => Info::Range(token.into()),
        Info::Owned(token) => Info::Owned(token.into()),
        Info::Borrowed(token) => Info::Borrowed(token.into()),
    }
}
