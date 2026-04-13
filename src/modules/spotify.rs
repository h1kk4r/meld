use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use chrono::Utc;
#[cfg(not(target_os = "windows"))]
use directories::BaseDirs;
use rspotify::{
    AuthCodePkceSpotify, Config as SpotifyApiConfig, Credentials, OAuth, Token, prelude::*, scopes,
};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::util::process;

use super::InfoLine;

const SPOTIFY_SEPARATOR: char = '\u{1f}';
const SPOTIFY_TOKEN_FILE: &str = "spotify-token.json";
const SPOTIFY_PLAYBACK_TIMEOUT_MS: u64 = 1_200;
const SPOTIFY_TOKEN_TIMEOUT_MS: u64 = 2_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpotifyConfig {
    pub format: String,
    pub cover_as_image: bool,
    pub client_id: Option<String>,
    pub redirect_uri: String,
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
            client_id: None,
            redirect_uri: "http://127.0.0.1:8888/callback".to_string(),
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
    pub fn inspect(config: &SpotifyConfig, config_dir: Option<&Path>) -> Option<Self> {
        inspect_api(config, config_dir).or_else(inspect_local)
    }

    pub fn login(config: &SpotifyConfig, config_dir: &Path) -> AppResult<PathBuf> {
        let token_path = spotify_token_path(config_dir);
        let mut spotify = spotify_client(config, &token_path)?;
        fs::create_dir_all(config_dir)?;

        let url = spotify
            .get_authorize_url(None)
            .map_err(|err| spotify_login_error(err.to_string()))?;

        spotify
            .prompt_for_token(&url)
            .map_err(|err| spotify_login_error(err.to_string()))?;

        Ok(token_path)
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

fn inspect_api(config: &SpotifyConfig, config_dir: Option<&Path>) -> Option<SpotifyInfo> {
    let token_path = spotify_token_path(config_dir?);
    let spotify = spotify_client(config, &token_path).ok()?;
    let mut token = spotify.read_token_cache(true).ok().flatten()?;

    if token.is_expired() {
        token = refresh_spotify_token(config, &token, &token_path)?;
    }

    let context = fetch_current_playing(&token)?;
    spotify_info_from_api_context(&context)
}

fn spotify_client(config: &SpotifyConfig, token_path: &Path) -> AppResult<AuthCodePkceSpotify> {
    let client_id = config
        .client_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Config(
                "set `config.spotify.client_id` before running `meld --spotify-login`".to_string(),
            )
        })?;

    if config.redirect_uri.trim().is_empty() {
        return Err(AppError::Config(
            "`config.spotify.redirect_uri` cannot be empty".to_string(),
        ));
    }

    let oauth = OAuth {
        redirect_uri: config.redirect_uri.trim().to_string(),
        scopes: scopes!("user-read-currently-playing", "user-read-playback-state"),
        ..OAuth::default()
    };
    let api_config = SpotifyApiConfig {
        cache_path: token_path.to_path_buf(),
        token_cached: true,
        ..SpotifyApiConfig::default()
    };

    Ok(AuthCodePkceSpotify::with_config(
        Credentials::new_pkce(client_id),
        oauth,
        api_config,
    ))
}

fn refresh_spotify_token(
    config: &SpotifyConfig,
    token: &Token,
    token_path: &Path,
) -> Option<Token> {
    let client_id = config.client_id.as_deref()?.trim();
    if client_id.is_empty() {
        return None;
    }

    let refresh_token = token.refresh_token.as_deref()?;
    let response = spotify_agent(SPOTIFY_TOKEN_TIMEOUT_MS)
        .post("https://accounts.spotify.com/api/token")
        .send_form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id),
        ])
        .ok()?;
    let mut refreshed = response.into_json::<Token>().ok()?;

    refreshed.expires_at = Utc::now().checked_add_signed(refreshed.expires_in);
    if refreshed.refresh_token.is_none() {
        refreshed.refresh_token = token.refresh_token.clone();
    }
    if refreshed.scopes.is_empty() {
        refreshed.scopes = token.scopes.clone();
    }
    refreshed.write_cache(token_path).ok()?;

    Some(refreshed)
}

fn fetch_current_playing(token: &Token) -> Option<Value> {
    let auth_header = format!("Bearer {}", token.access_token);
    let response = spotify_agent(SPOTIFY_PLAYBACK_TIMEOUT_MS)
        .get("https://api.spotify.com/v1/me/player/currently-playing")
        .query("additional_types", "track")
        .set("Authorization", &auth_header)
        .call()
        .ok()?;

    if response.status() == 204 {
        return None;
    }

    response.into_json::<Value>().ok()
}

fn spotify_agent(timeout_ms: u64) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout(Duration::from_millis(timeout_ms))
        .build()
}

fn spotify_info_from_api_context(context: &Value) -> Option<SpotifyInfo> {
    if context
        .get("currently_playing_type")
        .and_then(Value::as_str)
        .is_some_and(|item_type| item_type != "track")
    {
        return None;
    }

    let item = context.get("item")?;
    let state = if context
        .get("is_playing")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        SpotifyState::Playing
    } else {
        SpotifyState::Paused
    };

    spotify_info_from_raw_item(state, item)
}

fn spotify_info_from_raw_item(state: SpotifyState, item: &Value) -> Option<SpotifyInfo> {
    if item
        .get("type")
        .and_then(Value::as_str)
        .is_some_and(|item_type| item_type != "track")
    {
        return None;
    }

    let artist = item
        .get("artists")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|artist| artist.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join(", ");
    let track = item
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let album_value = item.get("album");
    let album = album_value
        .and_then(|album| album.get("name"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let artwork_url = album_value
        .and_then(|album| album.get("images"))
        .and_then(Value::as_array)
        .and_then(|images| {
            images
                .iter()
                .find_map(|image| image.get("url").and_then(Value::as_str))
        })
        .map(str::to_string);
    let track_id = item
        .get("uri")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            item.get("id")
                .and_then(Value::as_str)
                .map(|id| format!("spotify:track:{id}"))
        });

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

fn spotify_token_path(config_dir: &Path) -> PathBuf {
    config_dir.join(SPOTIFY_TOKEN_FILE)
}

fn spotify_login_error(message: String) -> AppError {
    AppError::Config(format!("spotify login failed: {message}"))
}

fn inspect_local() -> Option<SpotifyInfo> {
    #[cfg(target_os = "macos")]
    {
        inspect_macos()
    }

    #[cfg(not(target_os = "macos"))]
    {
        None
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
    use super::{
        SpotifyConfig, SpotifyInfo, SpotifyState, parse_output, sanitize_cache_key,
        spotify_info_from_api_context, spotify_info_from_raw_item,
    };

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
            ..SpotifyConfig::default()
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
    fn parses_raw_spotify_api_track() {
        let item = serde_json::json!({
            "type": "track",
            "id": "1bIsbimwBDdZpEhKsALdKV",
            "uri": "spotify:track:1bIsbimwBDdZpEhKsALdKV",
            "name": "Track",
            "artists": [
                { "name": "Artist One" },
                { "name": "Artist Two" }
            ],
            "album": {
                "name": "Album",
                "images": [
                    { "height": 640, "url": "https://img", "width": 640 }
                ]
            }
        });

        let info = spotify_info_from_raw_item(SpotifyState::Playing, &item).unwrap();

        assert_eq!(info.state, SpotifyState::Playing);
        assert_eq!(info.artist, "Artist One, Artist Two");
        assert_eq!(info.track, "Track");
        assert_eq!(info.album, "Album");
        assert_eq!(info.artwork_url.as_deref(), Some("https://img"));
        assert_eq!(
            info.track_id.as_deref(),
            Some("spotify:track:1bIsbimwBDdZpEhKsALdKV")
        );
    }

    #[test]
    fn parses_raw_spotify_api_context() {
        let context = serde_json::json!({
            "currently_playing_type": "track",
            "is_playing": false,
            "item": {
                "type": "track",
                "name": "Track",
                "artists": [{ "name": "Artist" }],
                "album": { "name": "Album", "images": [] }
            }
        });

        let info = spotify_info_from_api_context(&context).unwrap();

        assert_eq!(info.state, SpotifyState::Paused);
        assert_eq!(info.artist, "Artist");
        assert_eq!(info.track, "Track");
        assert_eq!(info.album, "Album");
    }

    #[test]
    fn sanitizes_cache_keys() {
        assert_eq!(
            sanitize_cache_key("spotify:track:1bIsbimwBDdZpEhKsALdKV"),
            "spotify_track_1bIsbimwBDdZpEhKsALdKV"
        );
    }
}
