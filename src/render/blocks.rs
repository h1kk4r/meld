use crate::render::style::ColorSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorBlocksConfig {
    pub enabled: bool,
    pub width: usize,
    pub height: usize,
    pub symbol: String,
}

impl Default for ColorBlocksConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            width: 3,
            height: 1,
            symbol: "█".to_string(),
        }
    }
}

pub fn render(config: &ColorBlocksConfig) -> Vec<String> {
    if !config.enabled || config.width == 0 || config.height == 0 {
        return Vec::new();
    }

    let symbol = if config.symbol.is_empty() {
        "█".to_string()
    } else {
        config.symbol.repeat(config.width)
    };

    let mut rows = Vec::with_capacity(config.height * 2);

    for palette in [0_u8..8_u8, 8_u8..16_u8] {
        let row = render_row(palette, &symbol);

        for _ in 0..config.height {
            rows.push(row.clone());
        }
    }

    rows
}

fn render_row(range: std::ops::Range<u8>, symbol: &str) -> String {
    let mut row = String::new();

    for index in range {
        if let Some(color) = ColorSpec::from_index(index) {
            row.push_str(&color.paint_fg(symbol));
        }
    }

    row
}

#[cfg(test)]
mod tests {
    use super::{ColorBlocksConfig, render};

    #[test]
    fn renders_two_rows_of_blocks() {
        let output = render(&ColorBlocksConfig {
            enabled: true,
            width: 2,
            height: 1,
            symbol: "█".to_string(),
        });

        assert_eq!(output.len(), 2);
        assert!(output[0].contains("\u{1b}[30m██"));
        assert!(output[1].contains("\u{1b}[90m██"));
    }
}
