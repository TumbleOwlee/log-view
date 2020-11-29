
use nom::character::complete::hex_digit1;
use nom::bytes::complete::tag;
use nom::sequence::tuple;
use nom::combinator::opt;
use nom::error::ParseError;
use nom::IResult;

pub struct ColorParser {}

impl ColorParser {
    fn color_code(s: &str) -> IResult<&str, &str> {
        hex_digit1(s)
    }

    fn prefix(s: &str) -> IResult<&str, &str> {
        if !s.is_empty() && s.as_bytes()[0] == 0x1b {
            Ok((&s[1..], &s[0..1]))
        } else {
           Err(nom::Err::Error(nom::error::Error::from_char(s, 0x1b as char)))
        }
    }

    fn suffix(s: &str) -> IResult<&str, &str> {
        tag("m")(s)
    }

    fn background(s: &str) -> IResult<&str, (&str, &str)> {
        tuple((tag(";"), Self::color_code, tag(";"), Self::color_code))(s).map(
            |(r, (_, b, _, s))| (r, (b, s))
        )
    }

    pub fn parse(s: &str) -> IResult<&str, (&str, Option<(&str, &str)>)> {
        tuple((tuple((Self::prefix, tag("["))), Self::color_code, opt(Self::background), Self::suffix))(s).map(
            |(r, (_, fg, opt, _))| (r, (fg, opt))
        )
    }
}

