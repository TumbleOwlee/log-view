use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::opt;
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::IResult;

use std::str::FromStr;

use cursive::theme::BaseColor;
use cursive::theme::Color as CursiveColor;
use cursive::theme::ColorType;

type ColorTuple = (Option<ColorMode>, Option<ColorMode>, Option<u8>);

pub enum ColorMode {
    Default,
    Base16(u8),
    Base256(Color),
}

impl ColorMode {
    fn u8_into_color(val: u8) -> CursiveColor {
        match val {
            0 => CursiveColor::Dark(BaseColor::Black),
            1 => CursiveColor::Dark(BaseColor::Red),
            2 => CursiveColor::Dark(BaseColor::Green),
            3 => CursiveColor::Dark(BaseColor::Yellow),
            4 => CursiveColor::Dark(BaseColor::Blue),
            5 => CursiveColor::Dark(BaseColor::Magenta),
            6 => CursiveColor::Dark(BaseColor::Cyan),
            7 => CursiveColor::Dark(BaseColor::White),
            9 => CursiveColor::Light(BaseColor::Black),
            10 => CursiveColor::Light(BaseColor::Red),
            11 => CursiveColor::Light(BaseColor::Green),
            12 => CursiveColor::Light(BaseColor::Yellow),
            13 => CursiveColor::Light(BaseColor::Blue),
            14 => CursiveColor::Light(BaseColor::Magenta),
            15 => CursiveColor::Light(BaseColor::Cyan),
            16 => CursiveColor::Light(BaseColor::White),
            _ => CursiveColor::Dark(BaseColor::Black),
        }
    }
}

impl Into<ColorType> for ColorMode {
    fn into(self) -> ColorType {
        match self {
            ColorMode::Default => ColorType::InheritParent,
            ColorMode::Base16(v) => ColorType::Color(Self::u8_into_color(v)),
            ColorMode::Base256(c) => ColorType::Color(c.into()),
        }
    }
}

pub enum Color {
    Base(u8),
    Rgb((u8, u8, u8)),
}

impl Into<CursiveColor> for Color {
    fn into(self) -> CursiveColor {
        match self {
            Color::Base(v) => CursiveColor::from_256colors(v),
            Color::Rgb((r, g, b)) => CursiveColor::Rgb(r, g, b),
        }
    }
}

pub struct ColorParser {}

impl ColorParser {
    fn number_0_to_7(s: &str) -> IResult<&str, &str> {
        alt((
            tag("0"),
            tag("1"),
            tag("2"),
            tag("3"),
            tag("4"),
            tag("5"),
            tag("6"),
            tag("7"),
        ))(s)
    }

    fn foreground(s: &str) -> IResult<&str, ColorMode> {
        let mut res = tuple((tag("38;2;"), digit1, tag(";"), digit1, tag(";"), digit1))(s).map(
            |(rem, (_, r, _, g, _, b))| {
                (
                    rem,
                    ColorMode::Base256(Color::Rgb((
                        u8::from_str(r).unwrap(),
                        u8::from_str(g).unwrap(),
                        u8::from_str(b).unwrap(),
                    ))),
                )
            },
        );
        if res.is_err() {
            res = tuple((tag("38;5;"), digit1))(s).map(|(rem, (_, n))| {
                (
                    rem,
                    ColorMode::Base256(Color::Base(u8::from_str(n).unwrap())),
                )
            });
        }
        if res.is_err() {
            res = tuple((tag("3"), Self::number_0_to_7))(s)
                .map(|(r, (_, b))| (r, ColorMode::Base16(u8::from_str(b).unwrap())));
        }
        if res.is_err() {
            res = tuple((tag("9"), Self::number_0_to_7))(s)
                .map(|(r, (_, b))| (r, ColorMode::Base16(8 + u8::from_str(b).unwrap())));
        }
        if res.is_err() {
            res = tag("39")(s).map(|(r, _)| (r, ColorMode::Default));
        }
        res
    }

    fn background(s: &str) -> IResult<&str, ColorMode> {
        let mut res = tuple((tag("48;2;"), digit1, tag(";"), digit1, tag(";"), digit1))(s).map(
            |(rem, (_, r, _, g, _, b))| {
                (
                    rem,
                    ColorMode::Base256(Color::Rgb((
                        u8::from_str(r).unwrap(),
                        u8::from_str(g).unwrap(),
                        u8::from_str(b).unwrap(),
                    ))),
                )
            },
        );
        if res.is_err() {
            res = tuple((tag("48;5;"), digit1))(s).map(|(rem, (_, n))| {
                (
                    rem,
                    ColorMode::Base256(Color::Base(u8::from_str(n).unwrap())),
                )
            });
        }
        if res.is_err() {
            res = tuple((tag("4"), Self::number_0_to_7))(s)
                .map(|(r, (_, b))| (r, ColorMode::Base16(u8::from_str(b).unwrap())));
        }
        if res.is_err() {
            res = tuple((tag("10"), Self::number_0_to_7))(s)
                .map(|(r, (_, b))| (r, ColorMode::Base16(8 + u8::from_str(b).unwrap())));
        }
        if res.is_err() {
            res = tag("49")(s).map(|(r, _)| (r, ColorMode::Default));
        }
        res
    }

    fn special(s: &str) -> IResult<&str, u8> {
        digit1(s).map(|(r, n)| (r, u8::from_str(n).unwrap()))
    }

    fn prefix(s: &str) -> IResult<&str, &str> {
        if !s.is_empty() && s.as_bytes()[0] == 0x1b {
            Ok((&s[1..], &s[0..1]))
        } else {
            Err(nom::Err::Error(nom::error::Error::from_char(
                s,
                0x1b as char,
            )))
        }
    }

    fn suffix(s: &str) -> IResult<&str, &str> {
        tag("m")(s)
    }

    pub fn parse(s: &str) -> IResult<&str, ColorTuple> {
        tuple((
            tuple((Self::prefix, tag("["))),
            opt(Self::foreground),
            opt(tag(";")),
            opt(Self::background),
            opt(tag(";")),
            opt(Self::special),
            Self::suffix,
        ))(s)
        .map(|(r, (_, fg, _, bg, _, sp, _))| (r, (fg, bg, sp)))
    }
}
