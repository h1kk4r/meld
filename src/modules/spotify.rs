use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(not(target_os = "windows"))]
use directories::BaseDirs;

use crate::util::process;

use super::InfoLine;

const SPOTIFY_SEPARATOR: char = '\u{1f}';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpotifyConfig {
    pub format: String,
    pub cover_as_image: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpotifyInfo {
    pub state: SpotifyState,
    pub artist: String,
    pub track: String,
    pub album: String,
    pub artwork_url: Option<String>,
    pub track_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotifyState {
    Playing,
    Paused,
}

impl Default for SpotifyConfig {
    fn default() -> Self {
        Self {
            format: "$artist - $track".to_string(),
            cover_as_image: false,
        }
    }
}

impl SpotifyState {
    fn from_name(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "playing" => Some(Self::Playing),
            "paused" => Some(Self::Paused),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Playing => "playing",
            Self::Paused => "paused",
        }
    }
}

impl SpotifyInfo {
    pub fn inspect() -> Option<Self> {
        #[cfg(target_os = "macos")]
        {
            inspect_macos()
        }

        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }

    pub fn render(&self, config: &SpotifyConfig) -> Option<InfoLine> {
        let value = self.format_value(&config.format);

        (!value.trim().is_empty()).then(|| InfoLine::new("Spotify", value))
    }

    pub fn format_value(&self, format: &str) -> String {
        format
            .replace("$artist", &self.artist)
            .replace("$track", &self.track)
            .replace("$album", &self.album)
            .replace("$state", self.state.as_str())
            .replace("$id", self.track_id.as_deref().unwrap_or(""))
    }

    pub fn cover_path(&self) -> Option<PathBuf> {
        let artwork_url = self.artwork_url.as_deref()?;
        let cache_dir = spotify_cache_dir();
        let file_name = format!("{}.{}", self.cache_key(), artwork_extension(artwork_url));
        let path = cache_dir.join(file_name);

        if path.is_file() {
            return Some(path);
        }

        fs::create_dir_all(&cache_dir).ok()?;

        let temp_path = path.with_extension("download");
        let status = Command::new("curl")
            .args([
                "-fsSL",
                "--max-time",
                "2",
                "--output",
                temp_path.to_string_lossy().as_ref(),
                artwork_url,
            ])
            .status()
            .ok()?;

        if !status.success() {
            let _ = fs::remove_file(&temp_path);
            return None;
        }

        fs::rename(&temp_path, &path).ok()?;
        Some(path)
    }

    fn cache_key(&self) -> String {
        let raw = self
            .track_id
            .as_deref()
            .filter(|value| !value.is_empty())
            .or_else(|| self.artwork_url.as_deref())
            .unwrap_or("current-track");

        sanitize_cache_key(raw)
    }
}

#[cfg(target_os = "macos")]
fn inspect_macos() -> Option<SpotifyInfo> {
    let output = process::run("osascript", &spotify_script_args()).ok()?;
    if !output.success {
        return None;
    }

    parse_output(&output.stdout)
}

#[cfg(target_os = "macos")]
fn spotify_script_args() -> Vec<&'static str> {
    vec![
        "-l",
        "JavaScript",
        "-e",
        "(() => { try { const app = Application('Spotify'); if (!app.running()) return ''; const state = String(app.playerState()); if (state === 'stopped') return ''; const track = app.currentTrack(); const sep = '\\u001f'; return [state, track.artist() || '', track.name() || '', track.album() || '', track.artworkUrl() || '', track.id() || ''].join(sep); } catch (error) { return ''; } })()",
    ]
}

fn parse_output(output: &str) -> Option<SpotifyInfo> {
    let trimmed = output.trim();

    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split(SPOTIFY_SEPARATOR);
    let state = SpotifyState::from_name(parts.next()?)?;
    let artist = parts.next()?.to_string();
    let track = parts.next()?.to_string();
    let album = parts.next()?.to_string();
    let artwork_url = empty_to_none(parts.next()?);
    let track_id = empty_to_none(parts.next()?);

    if artist.is_empty() && track.is_empty() && album.is_empty() {
        return None;
    }

    Some(SpotifyInfo {
        state,
        artist,
        track,
        album,
        artwork_url,
        track_id,
    })
}

fn empty_to_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn spotify_cache_dir() -> PathBuf {
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(base_dirs) = BaseDirs::new() {
            return base_dirs
                .home_dir()
                .join(".cache")
                .join(env!("CARGO_PKG_NAME"))
                .join("spotify");
        }
    }

    #[allow(unreachable_code)]
    env::temp_dir().join(env!("CARGO_PKG_NAME")).join("spotify")
}

fn artwork_extension(url: &str) -> &'static str {
    let lower = url.to_ascii_lowercase();

    if lower.contains(".png") {
        "png"
    } else if lower.contains(".webp") {
        "webp"
    } else {
        "jpg"
    }
}

fn sanitize_cache_key(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());

    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
            sanitized.push(character);
        } else {
            sanitized.push('_');
        }
    }

    sanitized.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::{SpotifyConfig, SpotifyInfo, SpotifyState, parse_output, sanitize_cache_key};

    #[test]
    fn formats_track_placeholders() {
        let info = SpotifyInfo {
            state: SpotifyState::Playing,
            artist: "GONE.Fludd".to_string(),
            track: "СВЯТОЙ ГРААЛЬ".to_string(),
            album: "VOODOO CHILD".to_string(),
            artwork_url: None,
            track_id: Some("spotify:track:1".to_string()),
        };
        let config = SpotifyConfig {
            format: "$artist — $track [$album]".to_string(),
            cover_as_image: true,
        };

        let rendered = info.render(&config).unwrap();

        assert_eq!(rendered.value, "GONE.Fludd — СВЯТОЙ ГРААЛЬ [VOODOO CHILD]");
    }

    #[test]
    fn parses_spotify_output() {
        let output =
            "paused\u{1f}Artist\u{1f}Track\u{1f}Album\u{1f}https://img\u{1f}spotify:track:1";
        let info = parse_output(output).unwrap();

        assert_eq!(info.state, SpotifyState::Paused);
        assert_eq!(info.artist, "Artist");
        assert_eq!(info.track, "Track");
        assert_eq!(info.album, "Album");
        assert_eq!(info.artwork_url.as_deref(), Some("https://img"));
        assert_eq!(info.track_id.as_deref(), Some("spotify:track:1"));
    }

    #[test]
    fn sanitizes_cache_keys() {
        assert_eq!(
            sanitize_cache_key("spotify:track:1bIsbimwBDdZpEhKsALdKV"),
            "spotify_track_1bIsbimwBDdZpEhKsALdKV"
        );
    }
}
