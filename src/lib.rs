/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "cssparser"]
#![crate_type = "rlib"]

#![deny(missing_docs)]

/*!

Implementation of [CSS Syntax Module Level 3](https://drafts.csswg.org/css-syntax/) for Rust.

# Input

Everything is based on `Parser` objects, which borrow a `&str` input.
If you have bytes (from a file, the network, or something),
see the `decode_stylesheet_bytes` function.

# Conventions for parsing functions

* Take (at least) a `input: &mut cssparser::Parser` parameter
* Return `Result<_, ()>`
* When returning `Ok(_)`,
  the function must have consume exactly the amount of input that represents the parsed value.
* When returning `Err(())`, any amount of input may have been consumed.

As a consequence, when calling another parsing function, either:

* Any `Err(())` return value must be propagated.
  This happens by definition for tail calls,
  and can otherwise be done with the `try!` macro.
* Or the call must be wrapped in a `Parser::try` call.
  `try` takes a closure that takes a `Parser` and returns a `Result`,
  calls it once,
  and returns itself that same result.
  If the result is `Err`,
  it restores the position inside the input to the one saved before calling the closure.

Examples:

```{rust,ignore}
// 'none' | <image>
fn parse_background_image(context: &ParserContext, input: &mut Parser)
                                    -> Result<Option<Image>, ()> {
    if input.try(|input| input.expect_ident_matching("none")).is_ok() {
        Ok(None)
    } else {
        Image::parse(context, input).map(Some)  // tail call
    }
}
```

```{rust,ignore}
// [ <length> | <percentage> ] [ <length> | <percentage> ]?
fn parse_border_spacing(_context: &ParserContext, input: &mut Parser)
                          -> Result<(LengthOrPercentage, LengthOrPercentage), ()> {
    let first = try!(LengthOrPercentage::parse);
    let second = input.try(LengthOrPercentage::parse).unwrap_or(first);
    (first, second)
}
```

*/

#![recursion_limit="200"]  // For color::parse_color_keyword

extern crate encoding;
#[macro_use] extern crate matches;
#[cfg(test)] extern crate tempdir;
#[cfg(test)] extern crate rustc_serialize;
#[cfg(feature = "serde")] extern crate serde;
#[cfg(feature = "heapsize")] #[macro_use] extern crate heapsize;

pub use tokenizer::{Token, NumericValue, PercentageValue, SourceLocation};
pub use rules_and_declarations::{parse_important};
pub use rules_and_declarations::{DeclarationParser, DeclarationListParser, parse_one_declaration};
pub use rules_and_declarations::{RuleListParser, parse_one_rule};
pub use rules_and_declarations::{AtRuleType, QualifiedRuleParser, AtRuleParser};
pub use from_bytes::decode_stylesheet_bytes;
pub use color::{RGBA, Color, parse_color_keyword};
pub use nth::parse_nth;
pub use serializer::{ToCss, CssStringWriter, serialize_identifier, serialize_string, TokenSerializationType};
pub use parser::{Parser, Delimiter, Delimiters, SourcePosition};


/**

This macro is equivalent to a `match` expression on an `&str` value,
but matching is case-insensitive in the ASCII range.

Usage example:

```{rust,ignore}
match_ignore_ascii_case! { string,
    "foo" => Some(Foo),
    "bar" => Some(Bar),
    "baz" => Some(Baz),
    _ => None
}
```

The macro also takes a slice of the value,
so that a `String` or `CowString` could be passed directly instead of a `&str`.

*/
#[macro_export]
macro_rules! match_ignore_ascii_case {
    // parse the last case plus the fallback
    (@inner $value:expr, ($string:expr => $result:expr, _ => $fallback:expr) -> ($($parsed:tt)*) ) => {
        match_ignore_ascii_case!(@inner $value, () -> ($($parsed)* ($string => $result)) $fallback)
    };

    // parse a case (not the last one)
    (@inner $value:expr, ($string:expr => $result:expr, $($rest:tt)*) -> ($($parsed:tt)*) ) => {
        match_ignore_ascii_case!(@inner $value, ($($rest)*) -> ($($parsed)* ($string => $result)))
    };

    // finished parsing
    (@inner $value:expr, () -> ($(($string:expr => $result:expr))*) $fallback:expr ) => {
        {
            use std::ascii::AsciiExt;
            match &$value[..] {
                $(
                    s if s.eq_ignore_ascii_case($string) => $result,
                )+
                _ => $fallback
            }
        }
    };

    // entry point, start parsing
    ( $value:expr, $($rest:tt)* ) => {
        match_ignore_ascii_case!(@inner $value, ($($rest)*) -> ())
    };
}

mod rules_and_declarations;
mod tokenizer;
mod parser;
mod from_bytes;
mod color;
mod nth;
mod serializer;

#[cfg(test)]
mod tests;
