use std::fmt;

use crate::util::process::ProcessError;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Image(image::ImageError),
    Lua(mlua::Error),
    Process(ProcessError),
    Config(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "io error: {}", err),
            AppError::Image(err) => write!(f, "image error: {}", err),
            AppError::Lua(err) => write!(f, "lua error: {}", err),
            AppError::Process(err) => write!(f, "process error: {}", err),
            AppError::Config(message) => write!(f, "config error: {}", message),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<ProcessError> for AppError {
    fn from(err: ProcessError) -> Self {
        AppError::Process(err)
    }
}

impl From<image::ImageError> for AppError {
    fn from(err: image::ImageError) -> Self {
        AppError::Image(err)
    }
}

impl From<mlua::Error> for AppError {
    fn from(err: mlua::Error) -> Self {
        AppError::Lua(err)
    }
}
