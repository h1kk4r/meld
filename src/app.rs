use std::env;
use std::thread;

use crate::cli::Cli;
use crate::config::{self, ConfigLoadOptions, LineKey, LoadedConfig};
use crate::error::AppResult;
use crate::modules::InfoLine;
use crate::modules::git::GitInfo;
use crate::modules::spotify::SpotifyInfo;
use crate::modules::system::SystemProfile;
use crate::render;
use crate::render::backend as visual_backend;
use crate::render::image;

pub fn run(cli: Cli) -> AppResult<()> {
    let current_dir = env::current_dir()?;
    let load_options = ConfigLoadOptions {
        explicit_path: cli.config.clone(),
    };
    let resolution = config::inspect_paths(&current_dir, &load_options);

    if cli.show_config_path {
        if let Some(path) = resolution
            .resolved_path
            .clone()
            .or_else(|| resolution.candidate_paths.first().cloned())
        {
            println!("{}", path.display());
        } else {
            println!("no config path");
        }

        return Ok(());
    }

    if cli.print_config {
        let loaded = config::load_unresolved(&current_dir, &load_options)?;
        print!("{}", config::render_documented(&loaded.config));
        return Ok(());
    }

    let loaded = config::load(&current_dir, &load_options)?;

    if cli.diagnostics {
        print_diagnostics(&current_dir, &loaded);
        return Ok(());
    }

    let config = &loaded.config;
    let current_dir_for_git = current_dir.clone();
    let should_collect_git = config.order.contains(&LineKey::Git);
    let should_collect_spotify =
        config.order.contains(&LineKey::Spotify) || config.spotify.cover_as_image;
    let requested_system_fields = config.requested_system_fields();

    let git_handle =
        should_collect_git.then(|| thread::spawn(move || GitInfo::inspect(&current_dir_for_git)));
    let system_handle = (!requested_system_fields.is_empty()).then(|| {
        let requested_fields = requested_system_fields.clone();
        thread::spawn(move || SystemProfile::collect_requested(&requested_fields))
    });
    let spotify_handle = should_collect_spotify.then(|| thread::spawn(SpotifyInfo::inspect));

    let git = git_handle.and_then(join_handle);
    let system = system_handle.and_then(join_handle);
    let spotify = spotify_handle.and_then(join_handle).flatten();
    let lines = build_lines(
        &current_dir,
        config,
        git.as_ref(),
        system.as_ref(),
        spotify.as_ref(),
    );

    let output = render::plain::render(
        &lines,
        &config.layout,
        &config.colors,
        &config.text_style,
        &config.logo,
        &config.image,
        &config.spotify,
        spotify.as_ref(),
        &config.blocks,
    )?;
    if !output.is_empty() {
        println!("{output}");
    }

    Ok(())
}

fn configured_line(line_key: LineKey, mut line: InfoLine, config: &config::AppConfig) -> InfoLine {
    line.label = config.label_for(line_key).to_string();
    line
}

fn build_lines(
    current_dir: &std::path::Path,
    config: &config::AppConfig,
    git: Option<&GitInfo>,
    system: Option<&SystemProfile>,
    spotify: Option<&SpotifyInfo>,
) -> Vec<InfoLine> {
    let mut lines = Vec::new();

    for line_key in config.order.iter().copied() {
        match line_key {
            LineKey::Directory => lines.push(configured_line(
                line_key,
                InfoLine::new("Directory", current_dir.display().to_string()),
                config,
            )),
            LineKey::Git => {
                if let Some(git) = git {
                    if let Some(line) = git.render(config.git_view) {
                        lines.push(configured_line(line_key, line, config));
                    }
                }
            }
            LineKey::Spotify => {
                if let Some(spotify) = spotify {
                    if let Some(line) = spotify.render(&config.spotify) {
                        lines.push(configured_line(line_key, line, config));
                    }
                }
            }
            LineKey::System(field) => {
                if let Some(line) =
                    system.and_then(|system| system.render_field(field, &config.system_views))
                {
                    lines.push(configured_line(line_key, line, config));
                }
            }
        }
    }

    lines
}

fn join_handle<T>(handle: thread::JoinHandle<T>) -> Option<T> {
    handle.join().ok()
}

fn print_diagnostics(current_dir: &std::path::Path, loaded: &LoadedConfig) {
    let config = &loaded.config;
    let backend_plan = visual_backend::plan(&config.spotify, &config.image, &config.logo);
    let spotify = (config.order.contains(&LineKey::Spotify) || config.spotify.cover_as_image)
        .then(SpotifyInfo::inspect)
        .flatten();
    let requested_lines = config
        .order
        .iter()
        .map(|line| line_key_name(*line))
        .collect::<Vec<_>>()
        .join(", ");
    let requested_system = config
        .requested_system_fields()
        .iter()
        .map(|field| field.key())
        .collect::<Vec<_>>()
        .join(", ");
    let image_path = config
        .image
        .path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "<none>".to_string());
    let image_status = if config.image.enabled {
        if image::is_wezterm_session() {
            if config
                .image
                .path
                .as_ref()
                .is_some_and(|path| path.is_file())
            {
                "ready"
            } else {
                "missing file"
            }
        } else {
            "waiting for wezterm session"
        }
    } else {
        "disabled"
    };

    println!("Current Directory : {}", current_dir.display());
    println!(
        "Config Source     : {}",
        loaded
            .source_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "<defaults>".to_string())
    );
    println!("Config Candidates :");
    for path in &loaded.candidate_paths {
        println!("  {}", path.display());
    }
    println!("Requested Lines   : {requested_lines}");
    println!(
        "Requested System  : {}",
        if requested_system.is_empty() {
            "<none>"
        } else {
            &requested_system
        }
    );
    println!("Text Case         : {:?}", config.text_style.case);
    println!("Visual Backends   : {}", backend_plan.summary());
    println!(
        "WezTerm Session   : {}",
        if image::is_wezterm_session() {
            "yes"
        } else {
            "no"
        }
    );
    println!("Image Path        : {image_path}");
    println!("Image Status      : {image_status}");
    println!(
        "Spotify Requested : {}",
        if config.order.contains(&LineKey::Spotify) || config.spotify.cover_as_image {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "Spotify Status    : {}",
        spotify
            .as_ref()
            .map(|info| info.state.as_str())
            .unwrap_or("not available")
    );
    println!(
        "Spotify Track     : {}",
        spotify
            .as_ref()
            .map(|info| info.format_value(&config.spotify.format))
            .unwrap_or_else(|| "<none>".to_string())
    );
}

fn line_key_name(line: LineKey) -> &'static str {
    match line {
        LineKey::Directory => "directory",
        LineKey::Git => "git",
        LineKey::Spotify => "spotify",
        LineKey::System(field) => field.key(),
    }
}

#[cfg(test)]
mod tests {
    use super::build_lines;
    use crate::config::{AppConfig, LineKey};
    use std::path::Path;

    #[test]
    fn skips_spotify_line_when_spotify_is_unavailable() {
        let mut config = AppConfig::default();
        config.order = vec![LineKey::Spotify, LineKey::Directory];

        let lines = build_lines(Path::new("/tmp/meld"), &config, None, None, None);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].label, "Directory");
        assert_eq!(lines[0].value, "/tmp/meld");
    }
}
