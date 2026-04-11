use std::env;
use std::process::Command;

use crate::config::{OutputConfig, OutputHookConfig, OutputItemConfig};
use crate::error::{AppError, AppResult};

pub fn compose(config: &OutputConfig, main_output: &str) -> AppResult<String> {
    let mut output = String::new();

    append_hook(&mut output, &config.before)?;
    append_chunk(&mut output, main_output);
    append_hook(&mut output, &config.after)?;

    Ok(output)
}

fn append_hook(output: &mut String, hook: &OutputHookConfig) -> AppResult<()> {
    for item in &hook.items {
        match item {
            OutputItemConfig::Text(text) => append_chunk(output, text),
            OutputItemConfig::Command(command) => {
                if let Some(command_output) = run_shell_command(command)? {
                    append_chunk(output, &command_output);
                }
            }
        }
    }

    Ok(())
}

fn append_chunk(output: &mut String, chunk: &str) {
    if chunk.is_empty() {
        return;
    }

    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }

    output.push_str(chunk);
}

fn run_shell_command(command: &str) -> AppResult<Option<String>> {
    let shell = env::var("SHELL")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "/bin/sh".to_string());
    let output = Command::new(&shell).arg("-lc").arg(command).output()?;

    if !output.status.success() {
        let stderr = trim_trailing_newlines(&String::from_utf8_lossy(&output.stderr));
        return Err(AppError::Config(format!(
            "output hook command `{}` failed (code: {:?}): {}",
            command,
            output.status.code(),
            stderr
        )));
    }

    let stdout = trim_trailing_newlines(&String::from_utf8_lossy(&output.stdout));
    Ok((!stdout.is_empty()).then_some(stdout))
}

fn trim_trailing_newlines(value: &str) -> String {
    value.trim_end_matches(['\r', '\n']).to_string()
}

#[cfg(test)]
mod tests {
    use super::compose;
    use crate::config::{OutputConfig, OutputHookConfig, OutputItemConfig};

    #[test]
    fn wraps_main_output_with_text_hooks() {
        let output = compose(
            &OutputConfig {
                before: OutputHookConfig {
                    items: vec![OutputItemConfig::Text("hello".to_string())],
                },
                after: OutputHookConfig {
                    items: vec![OutputItemConfig::Text("bye".to_string())],
                },
            },
            "main",
        )
        .unwrap();

        assert_eq!(output, "hello\nmain\nbye");
    }

    #[test]
    fn inserts_command_stdout() {
        let output = compose(
            &OutputConfig {
                before: OutputHookConfig {
                    items: vec![OutputItemConfig::Command("printf command".to_string())],
                },
                after: OutputHookConfig::default(),
            },
            "main",
        )
        .unwrap();

        assert_eq!(output, "command\nmain");
    }

    #[test]
    fn preserves_ordered_output_items() {
        let output = compose(
            &OutputConfig {
                before: OutputHookConfig {
                    items: vec![
                        OutputItemConfig::Text("hello".to_string()),
                        OutputItemConfig::Command("printf command".to_string()),
                        OutputItemConfig::Text("after command".to_string()),
                    ],
                },
                after: OutputHookConfig::default(),
            },
            "main",
        )
        .unwrap();

        assert_eq!(output, "hello\ncommand\nafter command\nmain");
    }

    #[test]
    fn preserves_explicit_blank_line_after_output() {
        let output = compose(
            &OutputConfig {
                before: OutputHookConfig::default(),
                after: OutputHookConfig {
                    items: vec![OutputItemConfig::Text("\n".to_string())],
                },
            },
            "main",
        )
        .unwrap();

        assert_eq!(output, "main\n\n");
    }
}
