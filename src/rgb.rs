use ratatui::style::Color;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Default)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseRgbError;

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for Color {
    fn from(val: Rgb) -> Self {
        Self::Rgb(val.r, val.g, val.b)
    }
}

impl std::fmt::Display for Rgb {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b,)
    }
}

impl FromStr for Rgb {
    type Err = ParseRgbError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let (Ok(r), Ok(g), Ok(b)) = {
            if !s.starts_with('#') || s.len() != 7 {
                return Err(ParseRgbError);
            }
            (
                u8::from_str_radix(&s[1..3], 16),
                u8::from_str_radix(&s[3..5], 16),
                u8::from_str_radix(&s[5..7], 16),
            )
        } {
            Ok(Self::new(r, g, b))
        } else {
            Err(ParseRgbError)
        }
    }
}
