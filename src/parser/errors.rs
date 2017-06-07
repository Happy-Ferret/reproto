use core::*;
use num_bigint;
use super::parser;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        ParseInt(::std::num::ParseIntError);
        ParseFloat(::std::num::ParseFloatError);
        ParseBigInt(::num_bigint::ParseBigIntError);
        FromUtf8Error(::std::string::FromUtf8Error);
    }

    errors {
        InvalidEscape {
        }

        Syntax(pos: RpPos, expected: Vec<parser::Rule>) {
            description("syntax error")
        }
    }
}
