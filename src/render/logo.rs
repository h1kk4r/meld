use crate::render::backend::VisualBackendKind;
use crate::render::style::ColorSpec;
use crate::render::visual::VisualLine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogoConfig {
    pub enabled: bool,
    pub preset: LogoPreset,
    pub size: LogoSize,
    pub padding: usize,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogoPreset {
    Macos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogoSize {
    Small,
    Large,
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preset: LogoPreset::default(),
            size: LogoSize::default(),
            padding: 2,
            text: None,
        }
    }
}

impl Default for LogoPreset {
    fn default() -> Self {
        Self::Macos
    }
}

impl Default for LogoSize {
    fn default() -> Self {
        Self::Large
    }
}

impl LogoPreset {
    pub fn from_name(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "macos" | "mac" => Some(Self::Macos),
            _ => None,
        }
    }
}

impl LogoSize {
    pub fn from_name(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "small" => Some(Self::Small),
            "large" | "big" => Some(Self::Large),
            _ => None,
        }
    }
}

#[allow(dead_code)]
pub fn render(config: &LogoConfig) -> Vec<VisualLine> {
    if !config.enabled {
        return Vec::new();
    }

    if config.text.is_some() {
        return render_for_kind(config, VisualBackendKind::LogoText);
    }

    render_for_kind(config, VisualBackendKind::LogoPreset)
}

pub fn render_for_kind(config: &LogoConfig, kind: VisualBackendKind) -> Vec<VisualLine> {
    if !config.enabled {
        return Vec::new();
    }

    match kind {
        VisualBackendKind::LogoText => config
            .text
            .as_deref()
            .map(|text| text.lines().map(VisualLine::plain).collect())
            .unwrap_or_default(),
        VisualBackendKind::LogoPreset => match (config.preset, config.size) {
            (LogoPreset::Macos, LogoSize::Small) => macos_small(),
            (LogoPreset::Macos, LogoSize::Large) => macos_large(),
        },
        _ => Vec::new(),
    }
}

fn macos_small() -> Vec<VisualLine> {
    vec![
        colored_line(ColorSpec::Ansi(2), "       .:'"),
        colored_line(ColorSpec::Ansi(2), "    _ :'_"),
        colored_line(ColorSpec::Ansi(3), " .'`_`-'_``."),
        colored_line(ColorSpec::Ansi(1), ":________.-'"),
        colored_line(ColorSpec::Ansi(1), ":_______:"),
        colored_line(ColorSpec::Ansi(5), " :_______`-;"),
        colored_line(ColorSpec::Ansi(4), "  `._.-._.'"),
    ]
}

fn macos_large() -> Vec<VisualLine> {
    vec![
        colored_line(ColorSpec::Ansi(2), "                    c.'"),
        colored_line(ColorSpec::Ansi(2), "                 ,xNMM."),
        colored_line(ColorSpec::Ansi(2), "               .OMMMMo"),
        colored_line(ColorSpec::Ansi(2), "               lMM\""),
        colored_line(ColorSpec::Ansi(2), "     .;loddo:.  .olloddol;."),
        colored_line(ColorSpec::Ansi(2), "   cKMMMMMMMMMMNWMMMMMMMMMM0:"),
        colored_line(ColorSpec::Ansi(3), " .KMMMMMMMMMMMMMMMMMMMMMMMWd."),
        colored_line(ColorSpec::Ansi(3), " XMMMMMMMMMMMMMMMMMMMMMMMX."),
        colored_line(ColorSpec::Ansi(1), ";MMMMMMMMMMMMMMMMMMMMMMMM:"),
        colored_line(ColorSpec::Ansi(1), ":MMMMMMMMMMMMMMMMMMMMMMMM:"),
        colored_line(ColorSpec::Ansi(1), ".MMMMMMMMMMMMMMMMMMMMMMMMX."),
        colored_line(ColorSpec::Ansi(1), " kMMMMMMMMMMMMMMMMMMMMMMMMWd."),
        colored_line(ColorSpec::Ansi(5), " 'XMMMMMMMMMMMMMMMMMMMMMMMMMMk"),
        colored_line(ColorSpec::Ansi(5), "  'XMMMMMMMMMMMMMMMMMMMMMMMMK."),
        colored_line(ColorSpec::Ansi(4), "    kMMMMMMMMMMMMMMMMMMMMMMd"),
        colored_line(ColorSpec::Ansi(4), "     ;KMMMMMMMWXXWMMMMMMMk."),
        colored_line(ColorSpec::Ansi(4), "       \"cooc*\"    \"*coo'\""),
    ]
}

fn colored_line(color: ColorSpec, text: &'static str) -> VisualLine {
    VisualLine::colored(color.paint_fg(text), text)
}

#[cfg(test)]
mod tests {
    use super::{LogoConfig, LogoSize, render};

    #[test]
    fn renders_large_logo_by_default() {
        let lines = render(&LogoConfig {
            enabled: true,
            ..LogoConfig::default()
        });

        assert_eq!(lines.len(), 17);
    }

    #[test]
    fn renders_custom_text_logo() {
        let lines = render(&LogoConfig {
            enabled: true,
            text: Some("A\nBC".to_string()),
            size: LogoSize::Small,
            ..LogoConfig::default()
        });

        assert_eq!(lines[0].width, 1);
        assert_eq!(lines[1].width, 2);
    }
}
