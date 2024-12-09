//! Formatting graphql
use std::default::Default;

use crate::common::Directive;

#[derive(Debug, PartialEq)]
pub(crate) struct Formatter<'a> {
    buf: String,
    style: &'a Style,
    indent: u32,
}

/// A configuration of formatting style
///
/// Currently we only have indentation configured, other things might be
/// added later (such as minification).
#[derive(Debug, PartialEq, Clone)]
pub struct Style {
    indent: u32,
    multiline_arguments: bool,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            indent: 2,
            multiline_arguments: false,
        }
    }
}

impl Style {
    /// Change the number of spaces used for indentation
    pub fn indent(&mut self, indent: u32) -> &mut Self {
        self.indent = indent;
        self
    }

    /// Set whether to add new lines between arguments
    pub fn multiline_arguments(&mut self, multiline_arguments: bool) -> &mut Self {
        self.multiline_arguments = multiline_arguments;
        self
    }
}

pub(crate) trait Displayable {
    fn display(&self, f: &mut Formatter);
}

impl<'a> Formatter<'a> {
    pub fn new(style: &Style) -> Formatter {
        Formatter {
            buf: String::with_capacity(1024),
            style,
            indent: 0,
        }
    }

    pub fn indent(&mut self) {
        for _ in 0..self.indent {
            self.buf.push(' ');
        }
    }

    pub fn endline(&mut self) {
        self.buf.push('\n');
    }

    pub fn start_argument_block(&mut self, open_char: char) {
        self.buf.push(open_char);
        if self.style.multiline_arguments {
            self.inc_indent();
        }
    }

    pub fn end_argument_block(&mut self, close_char: char) {
        if self.style.multiline_arguments {
            self.endline();
            self.dec_indent();
            self.indent();
        }
        self.buf.push(close_char);
    }

    pub fn start_argument(&mut self) {
        if self.style.multiline_arguments {
            self.endline();
            self.indent();
        }
    }

    pub fn deliniate_argument(&mut self) {
        self.buf.push(',');
        if !self.style.multiline_arguments {
            self.buf.push(' ');
        }
    }

    pub fn start_block(&mut self) {
        self.buf.push('{');
        self.endline();
        self.inc_indent();
    }

    pub fn end_block(&mut self) {
        self.dec_indent();
        self.indent();
        self.buf.push('}');
        self.endline();
    }

    pub fn margin(&mut self) {
        if !self.buf.is_empty() {
            self.buf.push('\n');
        }
    }

    pub fn write(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    pub fn into_string(self) -> String {
        self.buf
    }

    pub fn write_quoted(&mut self, s: &str) {
        let mut has_newline = false;
        let mut has_nonprintable = false;
        for c in s.chars() {
            match c {
                '\n' => has_newline = true,
                '\r' | '\t' | '\u{0020}'..='\u{FFFF}' => {}
                _ => has_nonprintable = true,
            }
        }
        if !has_newline || has_nonprintable {
            use std::fmt::Write;
            self.buf.push('"');
            for c in s.chars() {
                match c {
                    '\r' => self.write(r"\r"),
                    '\n' => self.write(r"\n"),
                    '\t' => self.write(r"\t"),
                    '"' => self.write("\\\""),
                    '\\' => self.write(r"\\"),
                    '\u{0020}'..='\u{FFFF}' => self.buf.push(c),
                    _ => write!(&mut self.buf, "\\u{:04}", c as u32).unwrap(),
                }
            }
            self.buf.push('"');
        } else {
            self.buf.push_str(r#"""""#);
            self.endline();
            self.indent += self.style.indent;
            for line in s.lines() {
                if !line.trim().is_empty() {
                    self.indent();
                    self.write(&line.replace(r#"""""#, r#"\""""#));
                }
                self.endline();
            }
            self.indent -= self.style.indent;
            self.indent();
            self.buf.push_str(r#"""""#);
        }
    }

    fn inc_indent(&mut self) {
        self.indent += self.style.indent;
    }

    fn dec_indent(&mut self) {
        self.indent = self
            .indent
            .checked_sub(self.style.indent)
            .expect("negative indent");
    }
}

pub(crate) fn format_directives<'a, T>(dirs: &[Directive<'a, T>], f: &mut Formatter)
where
    T: crate::common::Text<'a>,
{
    for dir in dirs {
        f.write(" ");
        dir.display(f);
    }
}

macro_rules! impl_display {
    ($( $typ: ident, )+) => {
        $(
            impl fmt::Display for $typ {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str(&to_string(self))
                }
            }
        )+
    };

    ('a $($typ: ident, )+) => {
        $(
            impl<'a, T> fmt::Display for $typ<'a, T>
                where T: Text<'a>,
            {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str(&to_string(self))
                }
            }
        )+
    };
}
