use std::fmt;

use crate::modules::spotify::SpotifyConfig;
use crate::render::image::ImageConfig;
use crate::render::logo::LogoConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualBackendKind {
    SpotifyCover,
    Image,
    LogoText,
    LogoPreset,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualBackendPlan {
    pub priority: Vec<VisualBackendKind>,
}

pub fn plan(spotify: &SpotifyConfig, image: &ImageConfig, logo: &LogoConfig) -> VisualBackendPlan {
    let mut priority = Vec::new();

    if spotify.cover_as_image {
        priority.push(VisualBackendKind::SpotifyCover);
    }

    if image.enabled {
        priority.push(VisualBackendKind::Image);
    }

    if logo.enabled {
        if logo.text.is_some() {
            priority.push(VisualBackendKind::LogoText);
        } else {
            priority.push(VisualBackendKind::LogoPreset);
        }
    }

    if priority.is_empty() {
        priority.push(VisualBackendKind::None);
    }

    VisualBackendPlan { priority }
}

impl fmt::Display for VisualBackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpotifyCover => write!(f, "spotify-cover"),
            Self::Image => write!(f, "image"),
            Self::LogoText => write!(f, "logo-text"),
            Self::LogoPreset => write!(f, "logo-preset"),
            Self::None => write!(f, "none"),
        }
    }
}

impl VisualBackendPlan {
    pub fn summary(&self) -> String {
        self.priority
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

#[cfg(test)]
mod tests {
    use super::{VisualBackendKind, plan};
    use crate::modules::spotify::SpotifyConfig;
    use crate::render::image::ImageConfig;
    use crate::render::logo::LogoConfig;

    #[test]
    fn prioritizes_spotify_cover_then_image_then_logo() {
        let spotify = SpotifyConfig {
            cover_as_image: true,
            ..SpotifyConfig::default()
        };
        let image = ImageConfig {
            enabled: true,
            ..ImageConfig::default()
        };
        let logo = LogoConfig {
            enabled: true,
            ..LogoConfig::default()
        };

        let plan = plan(&spotify, &image, &logo);

        assert_eq!(
            plan.priority,
            vec![
                VisualBackendKind::SpotifyCover,
                VisualBackendKind::Image,
                VisualBackendKind::LogoPreset,
            ]
        );
    }
}
