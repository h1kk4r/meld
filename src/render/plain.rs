use crate::error::AppResult;
use crate::modules::InfoLine;
use crate::modules::spotify::{SpotifyConfig, SpotifyInfo};
use crate::render::backend::{self, VisualBackendKind};
use crate::render::blocks::{self, ColorBlocksConfig};
use crate::render::image::{self, ImageConfig};
use crate::render::layout::{self, LayoutConfig};
use crate::render::logo::{self, LogoConfig};
use crate::render::style::{TextColors, TextStyleConfig};
use crate::render::visual;

pub fn render(
    lines: &[InfoLine],
    layout_config: &LayoutConfig,
    text_colors: &TextColors,
    text_style: &TextStyleConfig,
    logo_config: &LogoConfig,
    image_config: &ImageConfig,
    spotify_config: &SpotifyConfig,
    spotify: Option<&SpotifyInfo>,
    blocks_config: &ColorBlocksConfig,
) -> AppResult<String> {
    let mut info_lines = layout::render_lines(lines, layout_config, text_colors, text_style);
    info_lines.extend(blocks::render(blocks_config));

    for kind in backend::plan(spotify_config, image_config, logo_config).priority {
        match kind {
            VisualBackendKind::SpotifyCover => {
                if let Some(spotify) = spotify {
                    if let Some(path) = spotify.cover_path() {
                        if let Ok(Some(image)) = image::render_path(image_config, &path) {
                            let output =
                                visual::compose(&image.placeholder, &info_lines, image.padding);
                            let line_count = output.lines().count();
                            let overlay = image::overlay_sequence(&image.overlay, line_count);

                            return Ok(format!("{output}{overlay}"));
                        }
                    }
                }
            }
            VisualBackendKind::Image => {
                if let Ok(Some(image)) = image::render(image_config) {
                    let output = visual::compose(&image.placeholder, &info_lines, image.padding);
                    let line_count = output.lines().count();
                    let overlay = image::overlay_sequence(&image.overlay, line_count);

                    return Ok(format!("{output}{overlay}"));
                }
            }
            VisualBackendKind::LogoText | VisualBackendKind::LogoPreset => {
                let logo_lines = logo::render_for_kind(logo_config, kind);
                if !logo_lines.is_empty() {
                    return Ok(visual::compose(
                        &logo_lines,
                        &info_lines,
                        logo_config.padding,
                    ));
                }
            }
            VisualBackendKind::None => return Ok(info_lines.join("\n")),
        }
    }

    Ok(info_lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::modules::InfoLine;
    use crate::modules::spotify::SpotifyConfig;
    use crate::render::blocks::ColorBlocksConfig;
    use crate::render::image::ImageConfig;
    use crate::render::layout::LayoutConfig;
    use crate::render::logo::{LogoConfig, LogoSize};
    use crate::render::style::{TextColors, TextStyleConfig};

    #[test]
    fn renders_with_layout_config() {
        let lines = vec![
            InfoLine::new("CPU", "Apple M2"),
            InfoLine::new("Resolution", "2560 x 1664"),
        ];

        let output = render(
            &lines,
            &LayoutConfig::default(),
            &TextColors::default(),
            &TextStyleConfig::default(),
            &LogoConfig::default(),
            &ImageConfig::default(),
            &SpotifyConfig::default(),
            None,
            &ColorBlocksConfig::default(),
        )
        .unwrap();

        assert_eq!(output, "CPU        : Apple M2\nResolution : 2560 x 1664");
    }

    #[test]
    fn renders_logo_next_to_info() {
        let lines = vec![InfoLine::new("CPU", "Apple M2")];
        let output = render(
            &lines,
            &LayoutConfig::default(),
            &TextColors::default(),
            &TextStyleConfig::default(),
            &LogoConfig {
                enabled: true,
                size: LogoSize::Small,
                ..LogoConfig::default()
            },
            &ImageConfig::default(),
            &SpotifyConfig::default(),
            None,
            &ColorBlocksConfig::default(),
        )
        .unwrap();

        assert!(output.contains("CPU : Apple M2"));
        assert!(output.contains("\u{1b}[32m"));
    }
}
