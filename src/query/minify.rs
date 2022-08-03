use crate::tokenizer::{Kind, Token, TokenStream};
use combine::StreamOnce;
use thiserror::Error;

/// Error minifying query
#[derive(Error, Debug)]
#[error("query minify error: {}", _0)]
pub struct MinifyError(String);

pub fn minify_query<'a>(source: String) ->  Result<String, MinifyError> {
  let mut bits: Vec<&str> = Vec::new();
  let mut stream = TokenStream::new(source.as_str());
  let mut prev_was_punctuator = false;

  loop {
      match stream.uncons() {
          Ok(x) => {
              let token: Token = x;
              let is_non_punctuator = token.kind != Kind::Punctuator;

              if prev_was_punctuator {
                  if is_non_punctuator {
                      bits.push(" ");
                  }
              }

              bits.push(token.value);
              prev_was_punctuator = is_non_punctuator;
          }
          Err(ref e) if e == &combine::easy::Error::end_of_input() => break,
          Err(e) => return Err(MinifyError(e.to_string())),
      }
  }

  Ok(bits.join(""))
}

#[cfg(test)]
mod tests {
    #[test]
    fn strip_ignored_characters() {
        let source = "
        query SomeQuery($foo: String!, $bar: String) {
            someField(foo: $foo, bar: $bar) {
                a
                b { 
                    ... on B {
                        c 
                        d 
                    } 
                } 
            } 
        }
        ";

        let minified = super::minify_query(source.to_string()).expect("minification failed");

        assert_eq!(
            &minified,
            "query SomeQuery($foo:String!$bar:String){someField(foo:$foo bar:$bar){a b{...on B{c d}}}}"
        );
    }
    
    #[test]
    fn unexpected_token() {
        let source = "
        query foo {
            bar;
        }
        ";

        let minified = super::minify_query(source.to_string());

        assert_eq!(
            minified.is_err(),
            true
        );

        assert_eq!(
            minified.unwrap_err().to_string(),
            "query minify error: Unexpected `unexpected character ';'`"
        );
    }
}
