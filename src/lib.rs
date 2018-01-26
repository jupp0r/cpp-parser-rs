#[macro_use]
extern crate nom;

#[macro_use]
extern crate error_chain;

mod parser;
mod errors;

use std::convert::AsRef;
use errors::*;

use std::vec::Vec;
use std::iter::Iterator;

use nom::IResult;

pub struct Model {
    pub includes: Vec<String>,
}

pub fn parse<T: AsRef<str>>(input: &T) -> Result<Model> {
    let includes_result = parser::include_block(input.as_ref().as_bytes()).map(|v| {
        v.into_iter()
            .map(|v| v.to_vec())
            .map(|v| ::std::string::String::from_utf8(v).map_err(|e| e.into()))
            .collect::<Result<Vec<_>>>()
    });
    let includes = match includes_result {
        IResult::Done(_, o) => o?,
        IResult::Error(e) => bail!(e.description()),
        IResult::Incomplete(_) => bail!("incomplete"),
    };
    Ok(Model { includes: includes })
}

#[cfg(test)]
mod tests {
    //     use parse;

    //     static TEST_INTERFACE: &str = r###"
    // #pragma once

    // #include <foo>

    // namespace a { namespace da {

    // class Bar {
    // virtual ~Bar() = default;
    // virtual void foo(int baz) = 0;
    // };

    // }
    // }
    // "###;

    //     #[test]
    //     fn it_parses_includes() {
    //         let model = parse(&TEST_INTERFACE);
    //         assert_eq!(model.includes[0], "<foo>");
    //     }
}
