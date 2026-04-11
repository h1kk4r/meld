use std::env;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use base64::Engine;
use crossterm::terminal;
use image::{DynamicImage, GenericImageView, ImageFormat, imageops::FilterType};

use crate::error::{AppError, AppResult};
use crate::render::visual::VisualLine;

const DEFAULT_CELL_WIDTH_PX: f64 = 8.0;
const DEFAULT_CELL_HEIGHT_PX: f64 = 16.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageConfig {
    pub enabled: bool,
    pub path: Option<PathBuf>,
    pub height: ImageHeight,
    pub crop: ImageCropMode,
    pub padding: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageHeight {
    Fixed(usize),
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageCropMode {
    Center,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedImage {
    pub placeholder: Vec<VisualLine>,
    pub overlay: String,
    pub padding: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CellGeometry {
    width_px: f64,
    height_px: f64,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: None,
            height: ImageHeight::Fixed(10),
            crop: ImageCropMode::Center,
            padding: 3,
        }
    }
}

impl Default for CellGeometry {
    fn default() -> Self {
        Self {
            width_px: DEFAULT_CELL_WIDTH_PX,
            height_px: DEFAULT_CELL_HEIGHT_PX,
        }
    }
}

impl ImageHeight {
    pub fn from_name(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" | "content" | "info" | "text" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn rows(self, auto_rows: usize) -> usize {
        match self {
            Self::Fixed(rows) => rows.max(1),
            Self::Auto => auto_rows.max(1),
        }
    }
}

impl ImageCropMode {
    pub fn from_name(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "center" | "crop_center" | "centre" => Some(Self::Center),
            "none" | "no_crop" | "no-crop" => Some(Self::None),
            _ => None,
        }
    }
}

impl ImageConfig {
    pub fn resolve_paths_from(&mut self, base_dir: &Path) {
        if let Some(path) = self.path.as_mut() {
            if path.is_relative() {
                *path = base_dir.join(&path);
            }
        }
    }
}

pub fn render(config: &ImageConfig, auto_height_rows: usize) -> AppResult<Option<RenderedImage>> {
    if !config.enabled {
        return Ok(None);
    }

    let Some(path) = config.path.as_deref() else {
        return Err(AppError::Config(
            "`image.path` is required when `image.enabled = true`".to_string(),
        ));
    };

    render_path(config, path, auto_height_rows)
}

pub fn render_path(
    config: &ImageConfig,
    path: &Path,
    auto_height_rows: usize,
) -> AppResult<Option<RenderedImage>> {
    if !is_wezterm_session() {
        return Ok(None);
    }

    if !path.is_file() {
        return Err(AppError::Config(format!(
            "image file not found: `{}`",
            path.display()
        )));
    }

    let source = image::open(path)?;
    let cell_geometry = current_cell_geometry();
    let rows = config.height.rows(auto_height_rows);
    let cols = estimate_width_cells(&source, rows, config.crop, cell_geometry);
    let prepared = preprocess(source, cols, rows, config.crop, cell_geometry);
    let overlay = wezterm_inline_image(&prepared, cols, rows)?;
    let placeholder = (0..rows)
        .map(|_| VisualLine::plain(" ".repeat(cols)))
        .collect();

    Ok(Some(RenderedImage {
        placeholder,
        overlay,
        padding: config.padding,
    }))
}

pub fn overlay_sequence(overlay: &str, line_count: usize) -> String {
    if overlay.is_empty() || line_count == 0 {
        return String::new();
    }

    let up = line_count.saturating_sub(1);

    if up == 0 {
        format!("\x1b7\r{overlay}\x1b8")
    } else {
        format!("\x1b7\x1b[{up}A\r{overlay}\x1b8")
    }
}

pub fn is_wezterm_session() -> bool {
    env::var_os("WEZTERM_PANE").is_some()
        || env::var("TERM_PROGRAM")
            .map(|value| value.eq_ignore_ascii_case("wezterm"))
            .unwrap_or(false)
}

fn current_cell_geometry() -> CellGeometry {
    let Ok(size) = terminal::window_size() else {
        return CellGeometry::default();
    };

    if size.columns == 0 || size.rows == 0 || size.width == 0 || size.height == 0 {
        return CellGeometry::default();
    }

    let width_px = size.width as f64 / size.columns as f64;
    let height_px = size.height as f64 / size.rows as f64;

    if width_px <= 0.0 || height_px <= 0.0 {
        CellGeometry::default()
    } else {
        CellGeometry {
            width_px,
            height_px,
        }
    }
}

fn estimate_width_cells(
    image: &DynamicImage,
    height_cells: usize,
    crop: ImageCropMode,
    cell_geometry: CellGeometry,
) -> usize {
    let square_width_factor = cell_geometry.height_px / cell_geometry.width_px;

    match crop {
        ImageCropMode::Center => {
            ((height_cells as f64 * square_width_factor).ceil() as usize).max(1)
        }
        ImageCropMode::None => {
            let (width, height) = image.dimensions();
            let aspect = width as f64 / height.max(1) as f64;
            ((height_cells as f64 * square_width_factor * aspect).ceil() as usize).max(1)
        }
    }
}

fn preprocess(
    image: DynamicImage,
    width_cells: usize,
    height_cells: usize,
    crop: ImageCropMode,
    cell_geometry: CellGeometry,
) -> DynamicImage {
    let target_width = ((width_cells as f64 * cell_geometry.width_px).round() as u32).max(1);
    let target_height = ((height_cells as f64 * cell_geometry.height_px).round() as u32).max(1);

    let source = match crop {
        ImageCropMode::Center => crop_to_aspect(image, target_width, target_height),
        ImageCropMode::None => image,
    };

    source.resize_exact(target_width, target_height, FilterType::CatmullRom)
}

fn crop_to_aspect(image: DynamicImage, target_width: u32, target_height: u32) -> DynamicImage {
    let (source_width, source_height) = image.dimensions();
    let source_aspect = source_width as f64 / source_height.max(1) as f64;
    let target_aspect = target_width as f64 / target_height.max(1) as f64;

    if (source_aspect - target_aspect).abs() < f64::EPSILON {
        return image;
    }

    if source_aspect > target_aspect {
        let cropped_width =
            ((source_height as f64 * target_aspect).round() as u32).min(source_width);
        let left = (source_width.saturating_sub(cropped_width)) / 2;
        image.crop_imm(left, 0, cropped_width.max(1), source_height)
    } else {
        let cropped_height =
            ((source_width as f64 / target_aspect).round() as u32).min(source_height);
        let top = (source_height.saturating_sub(cropped_height)) / 2;
        image.crop_imm(0, top, source_width, cropped_height.max(1))
    }
}

fn wezterm_inline_image(
    image: &DynamicImage,
    width_cells: usize,
    height_cells: usize,
) -> AppResult<String> {
    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, ImageFormat::Png)?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(buffer.into_inner());

    Ok(format!(
        "\x1b]1337;File=inline=1;width={width_cells};height={height_cells};preserveAspectRatio=0;doNotMoveCursor=1:{encoded}\x07"
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        CellGeometry, ImageConfig, ImageCropMode, ImageHeight, crop_to_aspect,
        estimate_width_cells, overlay_sequence, wezterm_inline_image,
    };
    use image::{DynamicImage, GenericImageView, RgbaImage};
    use std::path::Path;

    #[test]
    fn resolves_relative_paths_from_config_dir() {
        let mut config = ImageConfig {
            enabled: true,
            path: Some("assets/logo.png".into()),
            ..ImageConfig::default()
        };

        config.resolve_paths_from(Path::new("/tmp/meld"));

        assert_eq!(
            config.path.as_deref(),
            Some(Path::new("/tmp/meld/assets/logo.png"))
        );
    }

    #[test]
    fn center_crop_uses_square_like_slot_width() {
        let image = DynamicImage::ImageRgba8(RgbaImage::new(400, 200));
        assert_eq!(
            estimate_width_cells(&image, 8, ImageCropMode::Center, CellGeometry::default()),
            16
        );
    }

    #[test]
    fn auto_height_uses_info_block_rows() {
        assert_eq!(ImageHeight::Auto.rows(7), 7);
        assert_eq!(ImageHeight::Auto.rows(0), 1);
        assert_eq!(ImageHeight::Fixed(12).rows(7), 12);
    }

    #[test]
    fn no_crop_width_scales_with_source_aspect() {
        let image = DynamicImage::ImageRgba8(RgbaImage::new(400, 100));
        assert_eq!(
            estimate_width_cells(&image, 8, ImageCropMode::None, CellGeometry::default()),
            64
        );
    }

    #[test]
    fn center_crop_respects_non_square_cell_geometry() {
        let image = DynamicImage::ImageRgba8(RgbaImage::new(400, 200));
        let geometry = CellGeometry {
            width_px: 11.0,
            height_px: 27.0,
        };

        assert_eq!(
            estimate_width_cells(&image, 8, ImageCropMode::Center, geometry),
            20
        );
    }

    #[test]
    fn crops_from_center_to_target_aspect() {
        let image = DynamicImage::ImageRgba8(RgbaImage::new(400, 100));
        let cropped = crop_to_aspect(image, 160, 160);

        assert_eq!(cropped.dimensions(), (100, 100));
    }

    #[test]
    fn builds_overlay_cursor_restore_sequence() {
        let overlay = overlay_sequence("IMG", 4);
        assert_eq!(overlay, "\u{1b}7\u{1b}[3A\rIMG\u{1b}8");
    }

    #[test]
    fn builds_inline_wezterm_sequence() {
        let image = DynamicImage::ImageRgba8(RgbaImage::new(2, 2));
        let sequence = wezterm_inline_image(&image, 10, 5).unwrap();

        assert!(sequence.starts_with(
            "\u{1b}]1337;File=inline=1;width=10;height=5;preserveAspectRatio=0;doNotMoveCursor=1:"
        ));
        assert!(sequence.ends_with('\u{7}'));
    }
}
