use std::path::PathBuf;

use crate::modules::git::GitView;
use crate::modules::spotify::SpotifyConfig;
use crate::modules::system::{SystemField, SystemRenderConfig};
use crate::render::blocks::ColorBlocksConfig;
use crate::render::image::ImageConfig;
use crate::render::layout::LayoutConfig;
use crate::render::logo::LogoConfig;
use crate::render::style::{TextColors, TextStyleConfig};

/// Runtime options that affect config discovery.
///
/// By default Meld reads only the production config path. Tests and one-off
/// invocations should use `explicit_path` rather than hidden debug lookup rules.
#[derive(Debug, Clone, Default)]
pub struct ConfigLoadOptions {
    pub explicit_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ConfigPathResolution {
    pub resolved_path: Option<PathBuf>,
    pub candidate_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct LoadedConfig {
    pub config: AppConfig,
    pub source_path: Option<PathBuf>,
    pub candidate_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKey {
    Directory,
    Git,
    Spotify,
    System(SystemField),
}

pub const DEFAULT_LINE_ORDER: [LineKey; 15] = [
    LineKey::Directory,
    LineKey::Git,
    LineKey::System(SystemField::Host),
    LineKey::System(SystemField::Os),
    LineKey::System(SystemField::Kernel),
    LineKey::System(SystemField::Uptime),
    LineKey::System(SystemField::Packages),
    LineKey::System(SystemField::Shell),
    LineKey::System(SystemField::Terminal),
    LineKey::System(SystemField::Cpu),
    LineKey::System(SystemField::Gpu),
    LineKey::System(SystemField::Memory),
    LineKey::System(SystemField::Battery),
    LineKey::System(SystemField::Disk),
    LineKey::System(SystemField::Display),
];

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub order: Vec<LineKey>,
    pub git_view: GitView,
    pub spotify: SpotifyConfig,
    pub system_views: SystemRenderConfig,
    pub layout: LayoutConfig,
    pub labels: LineLabels,
    pub colors: TextColors,
    pub text_style: TextStyleConfig,
    pub logo: LogoConfig,
    pub image: ImageConfig,
    pub blocks: ColorBlocksConfig,
}

#[derive(Debug, Clone)]
pub struct LineLabels {
    pub directory: String,
    pub git: String,
    pub spotify: String,
    pub host: String,
    pub os: String,
    pub kernel: String,
    pub uptime: String,
    pub packages: String,
    pub shell: String,
    pub terminal: String,
    pub cpu: String,
    pub gpu: String,
    pub memory: String,
    pub battery: String,
    pub disk: String,
    pub display: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            order: DEFAULT_LINE_ORDER.to_vec(),
            git_view: GitView::default(),
            spotify: SpotifyConfig::default(),
            system_views: SystemRenderConfig::default(),
            layout: LayoutConfig::default(),
            labels: LineLabels::default(),
            colors: TextColors::default(),
            text_style: TextStyleConfig::default(),
            logo: LogoConfig::default(),
            image: ImageConfig::default(),
            blocks: ColorBlocksConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn label_for(&self, line: LineKey) -> &str {
        self.labels.get(line)
    }

    pub fn requested_system_fields(&self) -> Vec<SystemField> {
        self.order
            .iter()
            .filter_map(|line| match line {
                LineKey::System(field) => Some(*field),
                _ => None,
            })
            .collect()
    }
}

impl Default for LineLabels {
    fn default() -> Self {
        Self {
            directory: "Directory".to_string(),
            git: "Git".to_string(),
            spotify: "Spotify".to_string(),
            host: "Host".to_string(),
            os: "System".to_string(),
            kernel: "Kernel".to_string(),
            uptime: "Uptime".to_string(),
            packages: "Packages".to_string(),
            shell: "Shell".to_string(),
            terminal: "Terminal".to_string(),
            cpu: "CPU".to_string(),
            gpu: "GPU".to_string(),
            memory: "Memory".to_string(),
            battery: "Battery".to_string(),
            disk: "Disk".to_string(),
            display: "Resolution".to_string(),
        }
    }
}

impl LineLabels {
    pub fn get(&self, line: LineKey) -> &str {
        match line {
            LineKey::Directory => &self.directory,
            LineKey::Git => &self.git,
            LineKey::Spotify => &self.spotify,
            LineKey::System(SystemField::Host) => &self.host,
            LineKey::System(SystemField::Os) => &self.os,
            LineKey::System(SystemField::Kernel) => &self.kernel,
            LineKey::System(SystemField::Uptime) => &self.uptime,
            LineKey::System(SystemField::Packages) => &self.packages,
            LineKey::System(SystemField::Shell) => &self.shell,
            LineKey::System(SystemField::Terminal) => &self.terminal,
            LineKey::System(SystemField::Cpu) => &self.cpu,
            LineKey::System(SystemField::Gpu) => &self.gpu,
            LineKey::System(SystemField::Memory) => &self.memory,
            LineKey::System(SystemField::Battery) => &self.battery,
            LineKey::System(SystemField::Disk) => &self.disk,
            LineKey::System(SystemField::Display) => &self.display,
        }
    }

    pub fn set(&mut self, line: LineKey, value: String) {
        match line {
            LineKey::Directory => self.directory = value,
            LineKey::Git => self.git = value,
            LineKey::Spotify => self.spotify = value,
            LineKey::System(SystemField::Host) => self.host = value,
            LineKey::System(SystemField::Os) => self.os = value,
            LineKey::System(SystemField::Kernel) => self.kernel = value,
            LineKey::System(SystemField::Uptime) => self.uptime = value,
            LineKey::System(SystemField::Packages) => self.packages = value,
            LineKey::System(SystemField::Shell) => self.shell = value,
            LineKey::System(SystemField::Terminal) => self.terminal = value,
            LineKey::System(SystemField::Cpu) => self.cpu = value,
            LineKey::System(SystemField::Gpu) => self.gpu = value,
            LineKey::System(SystemField::Memory) => self.memory = value,
            LineKey::System(SystemField::Battery) => self.battery = value,
            LineKey::System(SystemField::Disk) => self.disk = value,
            LineKey::System(SystemField::Display) => self.display = value,
        }
    }
}
