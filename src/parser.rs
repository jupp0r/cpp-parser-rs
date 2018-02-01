use nom::{is_alphanumeric, is_space};

named!(
    one_include,
    ws!(preceded!(
        tag!("#include"),
        ws!(take_until_either_and_consume!(" \t\r\n"))
    ))
);

named!(pub include_block(&[u8]) -> Vec<&[u8]>, preceded!(take_until!("#include"), many1!(one_include)));

named!(
    one_namespace,
    ws!(preceded!(
        tag!("namespace"),
        ws!(terminated!(
            take_while!(is_alphanumeric),
            take_until_and_consume!("{")
        ))
    ))
);

named!(namespaces(&[u8]) -> Vec<&[u8]>, preceded!(take_until!("namespace"),many1!(one_namespace)));

named!(
    class_name,
    preceded!(
        alt!(take_until_and_consume!("class") | take_until_and_consume!("struct")),
        ws!(terminated!(
            take_while!(is_alphanumeric),
            take_until_and_consume!("{")
        ))
    )
);

named!(pub header(&[u8]) -> (Vec<&[u8]>, Vec<&[u8]>, &[u8], Vec<Method>),
       tuple!(include_block, namespaces, class_name, many0!(method)));

#[derive(Debug, PartialEq)]
pub struct Method {
    pub return_value: Option<String>,
    pub name: String,
    pub is_pure_virtual: bool,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, PartialEq)]
pub struct Argument {
    pub argument_type: String,
    pub argument_name: Option<String>,
}

named!(method(&[u8]) -> Method,
       dbg_dmp!(delimited!(take_until_and_consume!("virtual"), method_inner, take_until_and_consume!(";"))));
named!(method_inner(&[u8]) -> Method,
       dbg_dmp!(ws!(map!(
           ws!(tuple!(
               take_while!(is_alphanumeric),
               take_until!("("),
           dbg_dmp!(delimited!(tag!("("),
                   separated_list!(
                       char!(','),
                       parse_argument), preceded!(tag!(")"), take_until!(";"))))))
           , make_method))));

named!(parse_argument(&[u8]) -> Argument,
       dbg_dmp!(map!(preceded!(many0!(tag!(" ")), separated_list!(
           take_while!(is_space),
           take_while!(is_alphanumeric)))
            , make_argument)));

fn make_argument(parsed_argument: Vec<&[u8]>) -> Argument {
    println!(
        "{:?}",
        parsed_argument
            .iter()
            .map(|v| String::from_utf8(v.to_vec()).unwrap())
            .collect::<Vec<String>>()
    );
    Argument {
        argument_type: String::from_utf8(parsed_argument[0].to_owned()).unwrap(),
        argument_name: parsed_argument
            .get(1)
            .map(|v| String::from_utf8(v.to_vec()).unwrap()),
    }
}

fn make_method((return_raw, name_raw, arguments): (&[u8], &[u8], Vec<Argument>)) -> Method {
    Method {
        return_value: Some(String::from_utf8(return_raw.to_owned()).unwrap()),
        name: String::from_utf8(name_raw.to_owned()).unwrap(),
        is_pure_virtual: true,
        arguments,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use nom::IResult;

    // tests for one_include
    #[test]
    fn one_include_test() {
        assert_eq!(
            one_include(&b"#include <foo.h>\n"[..]),
            IResult::Done(&b""[..], &b"<foo.h>"[..])
        );
    }

    #[test]
    fn one_include_with_leading_whitespace_test() {
        assert_eq!(
            one_include(&b"  #include <foo.h>\n"[..]),
            IResult::Done(&b""[..], &b"<foo.h>"[..])
        );
    }

    #[test]
    fn one_include_with_trailing_whitespace_test() {
        assert_eq!(
            one_include(&b"#include <foo.h>  \n"[..]),
            IResult::Done(&b""[..], &b"<foo.h>"[..])
        );
    }

    #[test]
    fn one_include_with_trailing_and_leading_whitespace_test() {
        assert_eq!(
            one_include(&b"   #include <foo.h>  \n"[..]),
            IResult::Done(&b""[..], &b"<foo.h>"[..])
        );
    }

    // tests for include_block
    #[test]
    fn include_block_one_include_test() {
        assert_eq!(
            include_block(&b"#include <foo.h>\n"[..]),
            IResult::Done(&b""[..], vec![&b"<foo.h>"[..]])
        );
    }

    #[test]
    fn include_block_multiple_include_test() {
        assert_eq!(
            include_block(&b"#include <foo.h>\n#include \"bar.h\"\n"[..]),
            IResult::Done(&b""[..], vec![&b"<foo.h>"[..], &b"\"bar.h\""[..]])
        );
    }

    #[test]
    fn include_block_multiple_newline_separated_blocks_test() {
        let include_blocks = &br###"
#include <foo.h>
#include "bar.h"

#include <boost/shared_ptr.hpp>
#include <boost/thread.hpp>
"###[..];

        assert_eq!(
            include_block(include_blocks),
            IResult::Done(
                &b""[..],
                vec![
                    &b"<foo.h>"[..],
                    &b"\"bar.h\""[..],
                    &b"<boost/shared_ptr.hpp>"[..],
                    &b"<boost/thread.hpp>"[..],
                ]
            )
        );
    }

    // tests for one_namespace
    #[test]
    fn one_namespace_simple_test() {
        assert_eq!(
            one_namespace(&b"namespace foo {"[..]),
            IResult::Done(&b""[..], &b"foo"[..])
        );
    }

    // tests for namespaces
    #[test]
    fn namespaces_simple_test() {
        let namespace_block = &br###"
namespace foo {
namespace bar {
namespace baz{
"###[..];
        assert_eq!(
            namespaces(namespace_block),
            IResult::Done(&b""[..], vec![&b"foo"[..], &b"bar"[..], &b"baz"[..]])
        );
    }

    // tests for class_name
    #[test]
    fn test_class_name_class() {
        let class_block = &br###"class Foo {"###[..];
        assert_eq!(
            class_name(class_block),
            IResult::Done(&b""[..], &b"Foo"[..])
        );
    }

    #[test]
    fn test_class_name_struct() {
        let class_block = &br###"struct Bar {"###[..];
        assert_eq!(
            class_name(class_block),
            IResult::Done(&b""[..], &b"Bar"[..])
        );
    }

    // test for method
    #[test]
    fn test_simple_method_name() {
        let simple_method = &b"virtual void foobar() = 0;"[..];
        let parsed_method = method(simple_method).to_result().unwrap();
        assert_eq!(parsed_method.name, "foobar".to_owned());
    }

    #[test]
    fn test_simple_method_arguments() {
        let simple_method = &b"virtual void foobar() = 0;"[..];
        let parsed_method = method(simple_method).to_result().unwrap();
        assert_eq!(parsed_method.arguments.len(), 0);
    }

    #[test]
    fn test_method_with_one_argument() {
        let method_with_arguments = &b"virtual void baz(int abc) = 0;"[..];
        let parsed_method = method(method_with_arguments).to_result().unwrap();
        assert_eq!(parsed_method.arguments[0].argument_type, "int".to_owned());
        assert_eq!(
            parsed_method.arguments[0].argument_name,
            Some("abc".to_owned())
        );
    }

    #[test]
    fn test_method_with_two_arguments() {
        let method_with_arguments = &b"virtual void baz(int abc, bool cde) = 0;"[..];
        let parsed_method = method(method_with_arguments).to_result().unwrap();
        assert_eq!(parsed_method.arguments[0].argument_type, "int".to_owned());
        assert_eq!(
            parsed_method.arguments[0].argument_name,
            Some("abc".to_owned())
        );
        assert_eq!(parsed_method.arguments[1].argument_type, "bool".to_owned());
        assert_eq!(
            parsed_method.arguments[1].argument_name,
            Some("cde".to_owned())
        );
    }
}
