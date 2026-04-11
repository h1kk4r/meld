use crate::render::style::visible_width;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualLine {
    pub rendered: String,
    pub width: usize,
}

impl VisualLine {
    pub fn plain(text: impl Into<String>) -> Self {
        let rendered = text.into();
        let width = visible_width(&rendered);

        Self { rendered, width }
    }

    pub fn colored(rendered: String, plain: &str) -> Self {
        Self {
            rendered,
            width: visible_width(plain),
        }
    }
}

pub fn compose(left: &[VisualLine], right: &[String], gap: usize) -> String {
    if left.is_empty() {
        return right.join("\n");
    }

    let left_width = left.iter().map(|line| line.width).max().unwrap_or(0);
    let right_offset = " ".repeat(left_width + gap);
    let row_count = left.len().max(right.len());
    let mut rows = Vec::with_capacity(row_count);

    for index in 0..row_count {
        match (left.get(index), right.get(index)) {
            (Some(left_line), Some(right_line)) if !right_line.is_empty() => {
                let padding = " ".repeat(left_width.saturating_sub(left_line.width) + gap);
                rows.push(format!("{}{}{}", left_line.rendered, padding, right_line));
            }
            (Some(left_line), _) => rows.push(left_line.rendered.clone()),
            (None, Some(right_line)) if !right_line.is_empty() => {
                rows.push(format!("{}{}", right_offset, right_line));
            }
            (None, Some(_)) => rows.push(String::new()),
            (None, None) => {}
        }
    }

    rows.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{VisualLine, compose};

    #[test]
    fn composes_visual_and_info_columns() {
        let visual = vec![VisualLine::plain("XX"), VisualLine::plain("YY")];
        let info = vec!["CPU : Apple M2".to_string(), "GPU : Apple M2".to_string()];

        let output = compose(&visual, &info, 2);

        assert_eq!(output, "XX  CPU : Apple M2\nYY  GPU : Apple M2");
    }
}
