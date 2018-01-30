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

#[derive(Debug, PartialEq)]
pub struct Class {
    pub namespaces: Vec<String>,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct Model {
    pub includes: Vec<String>,
    pub classes: Vec<Class>,
}

pub fn parse<T: AsRef<str>>(input: &T) -> Result<Model> {
    parser::header(input.as_ref().as_bytes())
        .map(|(includes_raw, namespaces_raw, class_name_raw)| {
            let includes = includes_raw
                .into_iter()
                .map(|v| v.to_vec())
                .map(|v| ::std::string::String::from_utf8(v).map_err(|e| e.into()))
                .collect::<Result<Vec<_>>>()
                .unwrap();
            let namespaces = namespaces_raw
                .into_iter()
                .map(|v| v.to_vec())
                .map(|v| ::std::string::String::from_utf8(v).map_err(|e| e.into()))
                .collect::<Result<Vec<_>>>()
                .unwrap();
            let class_name = String::from_utf8_lossy(class_name_raw).into_owned();
            Model {
                includes,
                classes: vec![
                    Class {
                        namespaces,
                        name: class_name,
                    },
                ],
            }
        })
        .to_result()
        .map_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INTERFACE: &str = r###"
     #pragma once

    #include <foo>

     namespace a { namespace da {

    class Bar {
    virtual ~Bar() = default;
    virtual void foo(int baz) = 0;
    };

    }
    }
    "###;

    #[test]
    fn it_parses_includes() {
        let model = parse(&TEST_INTERFACE).unwrap();
        assert_eq!(model.includes[0], "<foo>");
    }

    #[test]
    fn it_parses_class_names() {
        let model = parse(&TEST_INTERFACE).unwrap();
        assert_eq!(model.classes[0].name, "Bar");
    }
}
