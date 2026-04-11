pub mod git;
pub mod spotify;
pub mod system;

#[derive(Debug, Clone)]
pub struct InfoLine {
    pub label: String,
    pub value: String,
}

impl InfoLine {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}
