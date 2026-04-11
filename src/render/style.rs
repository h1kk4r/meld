#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpec {
    Ansi(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextCase {
    #[default]
    Default,
    Lower,
    Upper,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextColors {
    pub label: Option<ColorSpec>,
    pub separator: Option<ColorSpec>,
    pub value: Option<ColorSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextStyleConfig {
    pub case: TextCase,
}

impl ColorSpec {
    pub fn from_name(name: &str) -> Option<Self> {
        let normalized = name.trim().to_ascii_lowercase();

        let index = match normalized.as_str() {
            "black" => 0,
            "red" => 1,
            "green" => 2,
            "yellow" => 3,
            "blue" => 4,
            "magenta" => 5,
            "cyan" => 6,
            "white" => 7,
            "bright_black" | "bright-black" | "gray" | "grey" => 8,
            "bright_red" | "bright-red" => 9,
            "bright_green" | "bright-green" => 10,
            "bright_yellow" | "bright-yellow" => 11,
            "bright_blue" | "bright-blue" => 12,
            "bright_magenta" | "bright-magenta" => 13,
            "bright_cyan" | "bright-cyan" => 14,
            "bright_white" | "bright-white" => 15,
            _ => normalized.parse::<u8>().ok()?,
        };

        Self::from_index(index)
    }

    pub fn from_index(index: u8) -> Option<Self> {
        (index < 16).then_some(Self::Ansi(index))
    }

    pub fn paint_fg(self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }

        format!("\x1b[{}m{}\x1b[0m", self.fg_code(), text)
    }

    fn fg_code(self) -> u8 {
        match self {
            Self::Ansi(index @ 0..=7) => 30 + index,
            Self::Ansi(index) => 90 + (index - 8),
        }
    }
}

impl TextCase {
    pub fn from_name(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "default" | "normal" => Some(Self::Default),
            "lower" | "lowercase" => Some(Self::Lower),
            "upper" | "uppercase" => Some(Self::Upper),
            _ => None,
        }
    }
}

pub fn colorize(text: &str, color: Option<ColorSpec>) -> String {
    match color {
        Some(color) => color.paint_fg(text),
        None => text.to_string(),
    }
}

pub fn apply_case(text: &str, text_case: TextCase) -> String {
    match text_case {
        TextCase::Default => text.to_string(),
        TextCase::Lower => text.to_lowercase(),
        TextCase::Upper => text.to_uppercase(),
    }
}

pub fn visible_width(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut width = 0;
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'[') {
            index += 2;

            while index < bytes.len() {
                let byte = bytes[index];
                index += 1;

                if byte.is_ascii_alphabetic() {
                    break;
                }
            }

            continue;
        }

        let character = text[index..].chars().next().unwrap();
        width += 1;
        index += character.len_utf8();
    }

    width
}

#[cfg(test)]
mod tests {
    use super::{ColorSpec, TextCase, apply_case, colorize, visible_width};

    #[test]
    fn parses_named_and_numeric_colors() {
        assert_eq!(ColorSpec::from_name("blue"), Some(ColorSpec::Ansi(4)));
        assert_eq!(
            ColorSpec::from_name("bright-black"),
            Some(ColorSpec::Ansi(8))
        );
        assert_eq!(ColorSpec::from_name("15"), Some(ColorSpec::Ansi(15)));
        assert_eq!(ColorSpec::from_name("16"), None);
    }

    #[test]
    fn colorize_wraps_text_in_ansi_codes() {
        let rendered = colorize("Shell", Some(ColorSpec::Ansi(4)));
        assert_eq!(rendered, "\u{1b}[34mShell\u{1b}[0m");
    }

    #[test]
    fn visible_width_counts_characters() {
        assert_eq!(visible_width("CPU"), 3);
        assert_eq!(visible_width("Диск"), 4);
    }

    #[test]
    fn visible_width_ignores_ansi_sequences() {
        assert_eq!(visible_width("\u{1b}[34mCPU\u{1b}[0m"), 3);
    }

    #[test]
    fn applies_requested_case() {
        assert_eq!(apply_case("Shell", TextCase::Lower), "shell");
        assert_eq!(apply_case("Shell", TextCase::Upper), "SHELL");
        assert_eq!(apply_case("Shell", TextCase::Default), "Shell");
    }
}
