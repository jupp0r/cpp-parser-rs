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

use parser::Method;

#[derive(Debug, PartialEq)]
pub struct Class {
    pub namespaces: Vec<String>,
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug, PartialEq)]
pub struct Model {
    pub includes: Vec<String>,
    pub classes: Vec<Class>,
}

pub fn parse<T: AsRef<str>>(input: &T) -> Result<Model> {
    parser::header(input.as_ref().as_bytes())
        .map(|(includes_raw, namespaces_raw, class_name_raw, methods)| {
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
                        methods,
                    },
                ],
            }
        })
        .to_result()
        .map_err(|e| format!("{:?}", e).into())
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
    virtual void foo(int, bool boo) = 0;
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

    #[test]
    fn it_parses_namespaces() {
        let model = parse(&TEST_INTERFACE).unwrap();
        assert_eq!(
            model.classes[0].namespaces,
            vec!["a".to_owned(), "da".to_owned()]
        )
    }

    #[test]
    fn it_parses_method_names() {
        let model = parse(&TEST_INTERFACE).unwrap();
        assert_eq!(model.classes[0].methods[0].name, "~Bar".to_owned());
        assert_eq!(model.classes[0].methods[1].name, "foo".to_owned());
    }

    #[test]
    fn it_parses_method_return_values() {
        let model = parse(&TEST_INTERFACE).unwrap();
        assert_eq!(
            model.classes[0].methods[1].return_value,
            Some("void".to_owned())
        );
    }

    #[test]
    fn it_parses_method_argument_types() {
        let model = parse(&TEST_INTERFACE).unwrap();
        assert_eq!(
            model.classes[0].methods[1].arguments[0].argument_type,
            "int".to_owned()
        );
        assert_eq!(
            model.classes[0].methods[1].arguments[1].argument_type,
            "bool".to_owned()
        );
    }

    #[test]
    fn it_parses_method_argument_names() {
        let model = parse(&TEST_INTERFACE).unwrap();
        assert_eq!(model.classes[0].methods[1].arguments[0].argument_name, None);
        assert_eq!(
            model.classes[0].methods[1].arguments[1].argument_name,
            Some("boo".to_owned())
        );
    }

}
