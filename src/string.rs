use cursive::theme::{Color, ColorStyle, ColorType, Effect, Style};
use cursive::utils::span::SpannedString;

use crate::parser::{ColorMode, ColorParser};

enum Value<'a> {
    Text(&'a str),
    Color((Option<ColorMode>, Option<ColorMode>, Option<u8>)),
}

pub struct ColoredString {}

impl ColoredString {
    pub fn plain(s: &str) -> SpannedString<Style> {
        let mut values: Vec<Value> = Vec::new();
        let (mut start, mut end) = (0, 0);
        let mut input = s;

        while !input.is_empty() {
            match ColorParser::parse(input) {
                Ok((r, _)) => {
                    if start < end {
                        values.push(Value::Text(&s[start..end]));
                    }
                    start = s.len() - r.len();
                    end = start;
                    input = r;
                }
                Err(_) => {
                    input = &input[1..];
                    end += 1;
                }
            }
        }
        if start != end {
            values.push(Value::Text(&s[start..end]));
        }
        Self::create_styled_string(values)
    }

    pub fn unstyled(s: &str) -> SpannedString<Style> {
        SpannedString::<Style>::plain(s)
    }

    pub fn styled(s: &str) -> SpannedString<Style> {
        let mut values: Vec<Value> = Vec::new();
        let (mut start, mut end) = (0, 0);
        let mut input = s;

        while !input.is_empty() {
            match ColorParser::parse(input) {
                Ok((r, (fg, bg, sp))) => {
                    if start < end {
                        values.push(Value::Text(&s[start..end]));
                    }
                    values.push(Value::Color((fg, bg, sp)));
                    start = s.len() - r.len();
                    end = start;
                    input = r;
                }
                Err(_) => {
                    input = &input[1..];
                    end += 1;
                }
            }
        }
        if start != end {
            values.push(Value::Text(&s[start..end]));
        }
        Self::create_styled_string(values)
    }

    fn create_style(fg: Option<ColorMode>, bg: Option<ColorMode>, sp: Option<u8>) -> Style {
        let mut fg = match fg {
            Some(v) => ColorType::Color(v.into()),
            None => ColorType::Color(Color::None),
        };
        let mut bg = match bg {
            Some(v) => ColorType::Color(v.into()),
            None => ColorType::Color(Color::None),
        };
        let mut effects = Default::default();
        match sp {
            Some(3) => std::mem::swap(&mut fg, &mut bg),
            Some(0) => effects &= Effect::Simple,
            Some(1) => effects &= Effect::Bold,
            Some(9) => effects &= Effect::Strikethrough,
            Some(4) => effects &= Effect::Underline,
            _ => {}
        }

        Style {
            color: Some(ColorStyle::new(fg, bg)),
            effects,
        }
    }

    fn create_styled_string(values: Vec<Value>) -> SpannedString<Style> {
        let mut output = SpannedString::new();
        let mut color: Option<(Option<ColorMode>, Option<ColorMode>, Option<u8>)> = None;
        let mut text = None;

        for val in values.into_iter() {
            match val {
                Value::Color((fg, bg, sp)) => color = Some((fg, bg, sp)),
                Value::Text(t) => text = Some(t),
            }
            if text.is_some() {
                let s = text.take().unwrap();
                if color.is_some() {
                    let (fg, bg, sp) = color.take().unwrap();
                    output.append(SpannedString::styled(s, Self::create_style(fg, bg, sp)));
                } else {
                    output.append(s);
                }
            }
        }

        output
    }
}
