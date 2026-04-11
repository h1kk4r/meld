mod documented;
mod parser;
mod schema;

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(not(target_os = "windows"))]
use directories::BaseDirs;
#[cfg(target_os = "windows")]
use directories::ProjectDirs;

use crate::error::{AppError, AppResult};
use parser::parse_source;

pub use schema::{AppConfig, ConfigLoadOptions, ConfigPathResolution, LineKey, LoadedConfig};

pub fn load(current_dir: &Path, options: &ConfigLoadOptions) -> AppResult<LoadedConfig> {
    load_internal(current_dir, options, true)
}

pub fn load_unresolved(current_dir: &Path, options: &ConfigLoadOptions) -> AppResult<LoadedConfig> {
    load_internal(current_dir, options, false)
}

pub fn render_documented(config: &AppConfig) -> String {
    documented::render(config)
}

fn load_internal(
    current_dir: &Path,
    options: &ConfigLoadOptions,
    resolve_relative_paths: bool,
) -> AppResult<LoadedConfig> {
    let resolution = inspect_paths(current_dir, options);

    if options.explicit_path.is_some() && resolution.resolved_path.is_none() {
        let path = resolution
            .candidate_paths
            .first()
            .cloned()
            .unwrap_or_else(|| current_dir.join("init.lua"));

        return Err(AppError::Config(format!(
            "config file not found: `{}`",
            path.display()
        )));
    }

    let Some(path) = resolution.resolved_path.clone() else {
        return Ok(LoadedConfig {
            config: AppConfig::default(),
            source_path: None,
            candidate_paths: resolution.candidate_paths,
        });
    };

    let source = fs::read_to_string(&path)?;
    let mut config = parse_source(&source, path.to_string_lossy().as_ref())?;

    if resolve_relative_paths {
        if let Some(parent) = path.parent() {
            config.image.resolve_paths_from(parent);
        }
    }

    Ok(LoadedConfig {
        config,
        source_path: Some(path),
        candidate_paths: resolution.candidate_paths,
    })
}

pub fn inspect_paths(current_dir: &Path, options: &ConfigLoadOptions) -> ConfigPathResolution {
    let candidate_paths = resolve_config_candidates(current_dir, options);
    let resolved_path = if options.explicit_path.is_some() {
        candidate_paths
            .first()
            .filter(|path| path.is_file())
            .cloned()
    } else {
        candidate_paths.iter().find(|path| path.is_file()).cloned()
    };

    ConfigPathResolution {
        resolved_path,
        candidate_paths,
    }
}

fn resolve_config_candidates(current_dir: &Path, options: &ConfigLoadOptions) -> Vec<PathBuf> {
    if let Some(path) = options.explicit_path.as_ref() {
        let explicit = if path.is_absolute() {
            path.clone()
        } else {
            current_dir.join(path)
        };

        return vec![explicit];
    }

    // Production lookup intentionally stays boring and predictable: one
    // XDG-style config path unless the caller explicitly passes --config.
    standard_config_path().into_iter().collect()
}

fn standard_config_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
            .map(|dirs| dirs.config_dir().join("init.lua"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        BaseDirs::new().map(|dirs| {
            dirs.home_dir()
                .join(".config")
                .join(env!("CARGO_PKG_NAME"))
                .join("init.lua")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigLoadOptions, resolve_config_candidates};
    use std::path::Path;

    #[test]
    fn standard_mode_uses_only_standard_config_path() {
        let current_dir = Path::new("/tmp/meld");
        let candidates = resolve_config_candidates(
            current_dir,
            &ConfigLoadOptions {
                explicit_path: None,
            },
        );

        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].ends_with(".config/meld/init.lua"));
    }
}
