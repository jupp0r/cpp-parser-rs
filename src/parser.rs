use nom::is_alphanumeric;

named!(
    one_include,
    ws!(preceded!(
        tag!("#include"),
        ws!(take_until_either_and_consume!(" \t\r\n"))
    ))
);

named!(pub include_block(&[u8]) -> Vec<&[u8]>, many1!(one_include));

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

named!(namespaces(&[u8]) -> Vec<&[u8]>, ws!(many1!(one_namespace)));

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
}