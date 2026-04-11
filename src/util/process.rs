use std::fmt;
use std::path::Path;
use std::process::{Command, Output};

pub type ProcessResult<T> = Result<T, ProcessError>;

#[derive(Debug)]
pub enum ProcessError {
    Io(std::io::Error),
    Failed {
        program: String,
        code: Option<i32>,
        stderr: String,
    },
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::Io(err) => write!(f, "process io error: {}", err),
            ProcessError::Failed {
                program,
                code,
                stderr,
            } => {
                write!(
                    f,
                    "command `{}` failed (code: {:?}): {}",
                    program, code, stderr
                )
            }
        }
    }
}

impl std::error::Error for ProcessError {}

impl From<std::io::Error> for ProcessError {
    fn from(err: std::io::Error) -> Self {
        ProcessError::Io(err)
    }
}

#[derive(Debug, Clone)]
pub struct CmdOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub code: Option<i32>,
}

impl From<Output> for CmdOutput {
    fn from(output: Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            success: output.status.success(),
            code: output.status.code(),
        }
    }
}

pub fn run(program: &str, args: &[&str]) -> ProcessResult<CmdOutput> {
    run_inner(program, args, None)
}

pub fn run_checked(program: &str, args: &[&str]) -> ProcessResult<CmdOutput> {
    ensure_success(program, run(program, args)?)
}

pub fn run_in_dir(program: &str, args: &[&str], dir: &Path) -> ProcessResult<CmdOutput> {
    run_inner(program, args, Some(dir))
}

#[allow(dead_code)]
pub fn run_checked_in_dir(program: &str, args: &[&str], dir: &Path) -> ProcessResult<CmdOutput> {
    ensure_success(program, run_in_dir(program, args, dir)?)
}

fn run_inner(program: &str, args: &[&str], dir: Option<&Path>) -> ProcessResult<CmdOutput> {
    let mut command = Command::new(program);
    command.args(args);

    if let Some(dir) = dir {
        command.current_dir(dir);
    }

    let output = command.output()?;

    Ok(CmdOutput::from(output))
}

fn ensure_success(program: &str, result: CmdOutput) -> ProcessResult<CmdOutput> {
    if result.success {
        return Ok(result);
    }

    Err(ProcessError::Failed {
        program: program.to_string(),
        code: result.code,
        stderr: result.stderr.clone(),
    })
}
