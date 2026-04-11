use crate::modules::InfoLine;
use crate::render::style::{TextColors, TextStyleConfig, apply_case, colorize, visible_width};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutConfig {
    pub align: bool,
    pub separator: String,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            align: true,
            separator: ":".to_string(),
        }
    }
}

pub fn render_lines(
    lines: &[InfoLine],
    config: &LayoutConfig,
    colors: &TextColors,
    text_style: &TextStyleConfig,
) -> Vec<String> {
    let width = if config.align {
        lines
            .iter()
            .map(|line| visible_width(&apply_case(&line.label, text_style.case)))
            .max()
            .unwrap_or(0)
    } else {
        0
    };

    lines
        .iter()
        .map(|line| render_line(line, width, config, colors, text_style))
        .collect()
}

pub fn separator_string(separator: &str) -> String {
    if separator.is_empty() {
        " ".to_string()
    } else {
        format!(" {} ", separator)
    }
}

fn render_line(
    line: &InfoLine,
    width: usize,
    config: &LayoutConfig,
    colors: &TextColors,
    text_style: &TextStyleConfig,
) -> String {
    let label = apply_case(&line.label, text_style.case);
    let value = apply_case(&line.value, text_style.case);

    let label = if config.align {
        pad_visible(&label, width)
    } else {
        label
    };
    let separator = apply_case(&separator_string(&config.separator), text_style.case);

    format!(
        "{}{}{}",
        colorize(&label, colors.label),
        colorize(&separator, colors.separator),
        colorize(&value, colors.value)
    )
}

fn pad_visible(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(visible_width(text));

    if padding == 0 {
        text.to_string()
    } else {
        format!("{text}{}", " ".repeat(padding))
    }
}

#[cfg(test)]
mod tests {
    use super::{LayoutConfig, pad_visible, render_lines, separator_string};
    use crate::modules::InfoLine;
    use crate::render::style::{ColorSpec, TextCase, TextColors, TextStyleConfig};

    #[test]
    fn pads_labels_when_alignment_is_enabled() {
        let lines = vec![
            InfoLine::new("CPU", "Apple M2"),
            InfoLine::new("Resolution", "2560 x 1664"),
        ];

        let output = render_lines(
            &lines,
            &LayoutConfig::default(),
            &TextColors::default(),
            &TextStyleConfig::default(),
        );

        assert_eq!(output[0], "CPU        : Apple M2");
        assert_eq!(output[1], "Resolution : 2560 x 1664");
    }

    #[test]
    fn keeps_natural_width_when_alignment_is_disabled() {
        let lines = vec![InfoLine::new("Shell", "fish")];
        let config = LayoutConfig {
            align: false,
            separator: ":".to_string(),
        };

        let output = render_lines(
            &lines,
            &config,
            &TextColors::default(),
            &TextStyleConfig::default(),
        );

        assert_eq!(output[0], "Shell : fish");
    }

    #[test]
    fn colorizes_label_separator_and_value() {
        let lines = vec![InfoLine::new("Shell", "fish")];
        let colors = TextColors {
            label: Some(ColorSpec::Ansi(4)),
            separator: Some(ColorSpec::Ansi(8)),
            value: Some(ColorSpec::Ansi(2)),
        };

        let output = render_lines(
            &lines,
            &LayoutConfig::default(),
            &colors,
            &TextStyleConfig::default(),
        );

        assert_eq!(
            output[0],
            "\u{1b}[34mShell\u{1b}[0m\u{1b}[90m : \u{1b}[0m\u{1b}[32mfish\u{1b}[0m"
        );
    }

    #[test]
    fn separator_string_wraps_non_empty_symbols() {
        assert_eq!(separator_string(":"), " : ");
        assert_eq!(separator_string("|"), " | ");
        assert_eq!(separator_string(""), " ");
    }

    #[test]
    fn applies_case_before_rendering() {
        let lines = vec![InfoLine::new("Shell", "fish")];
        let text_style = TextStyleConfig {
            case: TextCase::Upper,
        };

        let output = render_lines(
            &lines,
            &LayoutConfig::default(),
            &TextColors::default(),
            &text_style,
        );

        assert_eq!(output[0], "SHELL : FISH");
    }

    #[test]
    fn preserves_alignment_for_ansi_colored_labels() {
        let lines = vec![
            InfoLine::new("\u{1b}[31mSh\u{1b}[0m", "fish"),
            InfoLine::new("Terminal", "Codex"),
        ];

        let output = render_lines(
            &lines,
            &LayoutConfig::default(),
            &TextColors::default(),
            &TextStyleConfig::default(),
        );

        assert_eq!(output[0], "\u{1b}[31mSh\u{1b}[0m       : fish");
        assert_eq!(output[1], "Terminal : Codex");
    }

    #[test]
    fn pad_visible_uses_display_width_instead_of_ansi_byte_length() {
        let padded = pad_visible("\u{1b}[31mkr\u{1b}[0m", 3);

        assert_eq!(padded, "\u{1b}[31mkr\u{1b}[0m ");
    }
}
