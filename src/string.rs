
use std::str::FromStr;

use cursive::utils::span::SpannedString;
use cursive::theme::{Style, BaseColor, Color, ColorStyle, ColorType};

use crate::ansi::Code;
use crate::parser::ColorParser;

macro_rules! match_color {
    ($a:ident, $x:ident, $b:ident, $($y:ident => $z:ident ($h:expr)),+) => (
        match $a {
            $(Code::$y => ColorType::$z($h)),+,
            _ => ColorType::Color(Color::None)
        }
    )
}

enum Value<'a> {
    Text(&'a str),
    Color((&'a str, Option<(&'a str, &'a str)>))
}

pub struct ColoredString {}

impl ColoredString {
    pub fn plain(s: &str) -> SpannedString<Style> {
        let mut values: Vec<Value> = Vec::new();
        let (mut start, mut end) = (0, 0);
        let mut input = s;

        while !input.is_empty() {
            match ColorParser::parse(input) {
                Ok((r, (_, _))) => {
                    if start < end {
                        values.push(Value::Text(&s[start..end]));
                    }
                    start = s.len() - r.len();
                    end = start;
                    input = r;
                },
                Err(_) => {
                    input = &input[1..];
                    end += 1;
                }
            }
        };
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
                Ok((r, (fg, opt))) => {
                    if start < end {
                        values.push(Value::Text(&s[start..end]));
                    }
                    values.push(Value::Color((fg, opt)));
                    start = s.len() - r.len();
                    end = start;
                    input = r;
                },
                Err(_) => {
                    input = &input[1..];
                    end += 1;
                }
            }
        };
        if start != end {
            values.push(Value::Text(&s[start..end]));
        }
        Self::create_styled_string(values)
    }

    fn create_style(fg: Code, bg: Code, c: Code) -> Style {
        let fg = match_color!(fg, style, front,
            FgRed => Color(Color::Dark(BaseColor::Red)),
            FgBlue => Color(Color::Dark(BaseColor::Blue)),
            FgCyan => Color(Color::Dark(BaseColor::Cyan)),
            FgBlack => Color(Color::Dark(BaseColor::Black)),
            FgGreen => Color(Color::Dark(BaseColor::Green)),
            FgYellow => Color(Color::Dark(BaseColor::Yellow)),
            FgMagenta => Color(Color::Dark(BaseColor::Magenta)),
            FgWhite => Color(Color::Dark(BaseColor::White)),
            FgLightRed => Color(Color::Light(BaseColor::Red)),
            FgLightBlue => Color(Color::Light(BaseColor::Blue)),
            FgLightCyan => Color(Color::Light(BaseColor::Cyan)),
            FgLightBlack => Color(Color::Light(BaseColor::Black)),
            FgLightGreen => Color(Color::Light(BaseColor::Green)),
            FgLightYellow => Color(Color::Light(BaseColor::Yellow)),
            FgLightMagenta => Color(Color::Light(BaseColor::Magenta)),
            FgLightWhite => Color(Color::Light(BaseColor::White))
        );
        let bg = match_color!(bg, style, back,
            BgRed => Color(Color::Dark(BaseColor::Red)),
            BgBlue => Color(Color::Dark(BaseColor::Blue)),
            BgCyan => Color(Color::Dark(BaseColor::Cyan)),
            BgBlack => Color(Color::Dark(BaseColor::Black)),
            BgGreen => Color(Color::Dark(BaseColor::Green)),
            BgYellow => Color(Color::Dark(BaseColor::Yellow)),
            BgMagenta => Color(Color::Dark(BaseColor::Magenta)),
            BgWhite => Color(Color::Dark(BaseColor::White)),
            BgLightRed => Color(Color::Light(BaseColor::Red)),
            BgLightBlue => Color(Color::Light(BaseColor::Blue)),
            BgLightCyan => Color(Color::Light(BaseColor::Cyan)),
            BgLightBlack => Color(Color::Light(BaseColor::Black)),
            BgLightGreen => Color(Color::Light(BaseColor::Green)),
            BgLightYellow => Color(Color::Light(BaseColor::Yellow)),
            BgLightMagenta => Color(Color::Light(BaseColor::Magenta)),
            BgLightWhite => Color(Color::Light(BaseColor::White))
        );

        Style {
            color: Some(ColorStyle::new(fg, bg)),
            ..Default::default()
        }
    }

    fn style_detailed(fg: u8, bg: u8, e: u8, s: &str) -> SpannedString<Style> {
        let fg = num_traits::FromPrimitive::from_u8(fg).unwrap_or(Code::FgDefault);
        let bg = num_traits::FromPrimitive::from_u8(bg).unwrap_or(Code::BgDefault);
        let e = num_traits::FromPrimitive::from_u8(e).unwrap_or(Code::ResetNormal);

        let style = Self::create_style(fg, bg, e);
        SpannedString::styled(s, style)
    }

    fn style_simple(fg: u8, s: &str) -> SpannedString<Style> {
        let fg = num_traits::FromPrimitive::from_u8(fg).unwrap_or(Code::FgDefault);
        let style = Self::create_style(fg, Code::BgDefault, Code::ResetNormal);
        SpannedString::styled(s, style)
    }


    fn create_styled_string(values: Vec<Value>) -> SpannedString<Style> {
        let mut output = SpannedString::new();
        let mut color: Option<(&str, Option<(&str, &str)>)> = None;
        let mut text = None;

        for val in values.into_iter() {
            match val {
                Value::Color((fg, opt)) => color = Some((fg, opt)),
                Value::Text(t) => text = Some(t)
            }
            if text.is_some() {
                let s = text.take().unwrap();
                if color.is_some() {
                    let (fg, opt) = color.take().unwrap();
                    if let Some((bg, c)) = opt {
                        output.append(
                            Self::style_detailed(
                                u8::from_str(fg).unwrap(), 
                                u8::from_str(bg).unwrap(), 
                                u8::from_str(c).unwrap(), 
                                s
                            )
                        );
                    } else {
                        output.append(
                            Self::style_simple(
                                u8::from_str(fg).unwrap(),
                                s
                            )
                        );
                    }
                } else {
                    output.append(s);
                }
            }
        }
        
        output
    }
}
