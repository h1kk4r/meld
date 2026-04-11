#[cfg(target_os = "linux")]
use std::fs;
use std::path::Path;
use std::thread;
#[cfg(target_os = "macos")]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::util::process;

use super::InfoLine;

#[derive(Debug, Clone)]
pub struct SystemProfile {
    pub host: Option<HostInfo>,
    pub kernel: Option<KernelInfo>,
    pub os: Option<OsInfo>,
    pub uptime: Option<UptimeInfo>,
    pub packages: Option<PackagesInfo>,
    pub shell: Option<ShellInfo>,
    pub terminal: Option<TerminalInfo>,
    pub cpu: Option<CpuInfo>,
    pub gpus: Vec<GpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub battery: Option<BatteryInfo>,
    pub disks: Vec<DiskInfo>,
    pub displays: Vec<DisplayInfo>,
}

#[derive(Debug, Clone, Copy)]
pub struct SystemRenderConfig {
    pub host: HostView,
    pub kernel: KernelView,
    pub os: OsView,
    pub uptime: UptimeView,
    pub packages: PackagesView,
    pub shell: ShellView,
    pub terminal: TerminalView,
    pub cpu: CpuView,
    pub gpu: GpuView,
    pub memory: MemoryView,
    pub battery: BatteryView,
    pub disk: DiskView,
    pub display: DisplayView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemField {
    Host,
    Os,
    Kernel,
    Uptime,
    Packages,
    Shell,
    Terminal,
    Cpu,
    Gpu,
    Memory,
    Battery,
    Disk,
    Display,
}

pub const DEFAULT_SYSTEM_FIELD_ORDER: [SystemField; 13] = [
    SystemField::Host,
    SystemField::Os,
    SystemField::Kernel,
    SystemField::Uptime,
    SystemField::Packages,
    SystemField::Shell,
    SystemField::Terminal,
    SystemField::Cpu,
    SystemField::Gpu,
    SystemField::Memory,
    SystemField::Battery,
    SystemField::Disk,
    SystemField::Display,
];

impl SystemField {
    pub fn key(self) -> &'static str {
        match self {
            SystemField::Host => "host",
            SystemField::Os => "os",
            SystemField::Kernel => "kernel",
            SystemField::Uptime => "uptime",
            SystemField::Packages => "packages",
            SystemField::Shell => "shell",
            SystemField::Terminal => "terminal",
            SystemField::Cpu => "cpu",
            SystemField::Gpu => "gpu",
            SystemField::Memory => "memory",
            SystemField::Battery => "battery",
            SystemField::Disk => "disk",
            SystemField::Display => "display",
        }
    }
}

impl Default for SystemRenderConfig {
    fn default() -> Self {
        Self {
            host: HostView::Full,
            kernel: KernelView::Full,
            os: OsView::NameVersion,
            uptime: UptimeView::Short,
            packages: PackagesView::Compact,
            shell: ShellView::Name,
            terminal: TerminalView::Name,
            cpu: CpuView::ModelCores,
            gpu: GpuView::Full,
            memory: MemoryView::Full,
            battery: BatteryView::Status,
            disk: DiskView::MountUsedTotal,
            display: DisplayView::Resolution,
        }
    }
}

impl SystemProfile {
    pub fn collect_requested(fields: &[SystemField]) -> Self {
        let selection = SystemFieldSelection::from_fields(fields);
        let host = selection.host.then(|| thread::spawn(collect_host));
        let kernel = selection
            .kernel
            .then(|| thread::spawn(|| collect_kernel().ok()));
        let os = selection.os.then(|| thread::spawn(|| collect_os().ok()));
        let uptime = selection
            .uptime
            .then(|| thread::spawn(|| collect_uptime().ok()));
        let packages = selection.packages.then(|| thread::spawn(collect_packages));
        let shell = selection.shell.then(|| thread::spawn(collect_shell));
        let terminal = selection.terminal.then(|| thread::spawn(collect_terminal));
        let cpu = selection.cpu.then(|| thread::spawn(|| collect_cpu().ok()));
        let memory = selection
            .memory
            .then(|| thread::spawn(|| collect_memory().ok()));
        let battery = selection.battery.then(|| thread::spawn(collect_battery));
        let disks = selection
            .disk
            .then(|| thread::spawn(|| collect_disks().unwrap_or_default()));
        let graphics = (selection.gpu || selection.display)
            .then(|| thread::spawn(|| collect_graphics_bundle().unwrap_or_default()));
        let graphics = join_handle(graphics).unwrap_or_default();

        Self {
            host: join_handle(host).flatten(),
            kernel: join_handle(kernel).flatten(),
            os: join_handle(os).flatten(),
            uptime: join_handle(uptime).flatten(),
            packages: join_handle(packages).flatten(),
            shell: join_handle(shell).flatten(),
            terminal: join_handle(terminal).flatten(),
            cpu: join_handle(cpu).flatten(),
            gpus: graphics.gpus,
            memory: join_handle(memory).flatten(),
            battery: join_handle(battery).flatten(),
            disks: join_handle(disks).unwrap_or_default(),
            displays: graphics.displays,
        }
    }

    #[allow(dead_code)]
    pub fn render(&self, config: &SystemRenderConfig) -> Vec<InfoLine> {
        DEFAULT_SYSTEM_FIELD_ORDER
            .into_iter()
            .filter_map(|field| self.render_field(field, config))
            .collect()
    }

    pub fn render_field(
        &self,
        field: SystemField,
        config: &SystemRenderConfig,
    ) -> Option<InfoLine> {
        match field {
            SystemField::Host => self
                .host
                .as_ref()
                .map(|host| InfoLine::new("Host", host.render(config.host))),
            SystemField::Os => self
                .os
                .as_ref()
                .map(|os| InfoLine::new("System", os.render(config.os))),
            SystemField::Kernel => self
                .kernel
                .as_ref()
                .map(|kernel| InfoLine::new("Kernel", kernel.render(config.kernel))),
            SystemField::Uptime => self
                .uptime
                .as_ref()
                .map(|uptime| InfoLine::new("Uptime", uptime.render(config.uptime))),
            SystemField::Packages => self
                .packages
                .as_ref()
                .map(|packages| InfoLine::new("Packages", packages.render(config.packages))),
            SystemField::Shell => self
                .shell
                .as_ref()
                .map(|shell| InfoLine::new("Shell", shell.render(config.shell))),
            SystemField::Terminal => self
                .terminal
                .as_ref()
                .map(|terminal| InfoLine::new("Terminal", terminal.render(config.terminal))),
            SystemField::Cpu => self
                .cpu
                .as_ref()
                .map(|cpu| InfoLine::new("CPU", cpu.render(config.cpu))),
            SystemField::Gpu => (!self.gpus.is_empty()).then(|| {
                let value = self
                    .gpus
                    .iter()
                    .map(|gpu| gpu.render(config.gpu))
                    .collect::<Vec<_>>()
                    .join(" | ");

                InfoLine::new("GPU", value)
            }),
            SystemField::Memory => self
                .memory
                .as_ref()
                .map(|memory| InfoLine::new("Memory", memory.render(config.memory))),
            SystemField::Battery => self
                .battery
                .as_ref()
                .map(|battery| InfoLine::new("Battery", battery.render(config.battery))),
            SystemField::Disk => (!self.disks.is_empty()).then(|| {
                let value = self
                    .disks
                    .iter()
                    .map(|disk| disk.render(config.disk))
                    .collect::<Vec<_>>()
                    .join(" | ");

                InfoLine::new("Disk", value)
            }),
            SystemField::Display => (!self.displays.is_empty()).then(|| {
                let value = self
                    .displays
                    .iter()
                    .map(|display| display.render(config.display))
                    .collect::<Vec<_>>()
                    .join(" | ");

                InfoLine::new("Resolution", value)
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostInfo {
    pub name: String,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum HostView {
    Name,
    Hostname,
    #[default]
    Full,
}

impl HostInfo {
    pub fn render(&self, view: HostView) -> String {
        match view {
            HostView::Name => self.name.clone(),
            HostView::Hostname => self.hostname.clone().unwrap_or_else(|| self.name.clone()),
            HostView::Full => match &self.hostname {
                Some(hostname) if hostname != &self.name => format!("{} | {}", self.name, hostname),
                _ => self.name.clone(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct KernelInfo {
    pub name: String,
    pub version: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum KernelView {
    Name,
    Version,
    NameVersion,
    #[default]
    Full,
}

impl KernelInfo {
    pub fn render(&self, view: KernelView) -> String {
        match view {
            KernelView::Name => self.name.clone(),
            KernelView::Version => self.version.clone(),
            KernelView::NameVersion => format!("{} {}", self.name, self.version),
            KernelView::Full => format!("{} {} ({})", self.name, self.version, self.architecture),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OsInfo {
    pub name: String,
    pub version: Option<String>,
    pub build: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum OsView {
    Name,
    Version,
    #[default]
    NameVersion,
    Full,
}

impl OsInfo {
    pub fn render(&self, view: OsView) -> String {
        match view {
            OsView::Name => self.name.clone(),
            OsView::Version => self.version.clone().unwrap_or_else(|| self.name.clone()),
            OsView::NameVersion => match &self.version {
                Some(version) => format!("{} {}", self.name, version),
                None => self.name.clone(),
            },
            OsView::Full => {
                let mut value = self.render(OsView::NameVersion);

                if let Some(build) = &self.build {
                    value.push_str(&format!(" ({})", build));
                }

                value
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct UptimeInfo {
    pub seconds: u64,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum UptimeView {
    #[default]
    Short,
    Full,
}

impl UptimeInfo {
    pub fn render(&self, view: UptimeView) -> String {
        match view {
            UptimeView::Short => format_duration_short(self.seconds),
            UptimeView::Full => format_duration_long(self.seconds),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackagesInfo {
    pub total_count: usize,
    pub managers: Vec<PackageManagerCount>,
}

#[derive(Debug, Clone)]
pub struct PackageManagerCount {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum PackagesView {
    Total,
    #[default]
    Compact,
    Full,
}

impl PackagesInfo {
    pub fn render(&self, view: PackagesView) -> String {
        match view {
            PackagesView::Total => self.total_count.to_string(),
            PackagesView::Compact => {
                if self.managers.is_empty() {
                    self.total_count.to_string()
                } else {
                    let breakdown = self
                        .managers
                        .iter()
                        .map(|manager| format!("{} {}", manager.name, manager.count))
                        .collect::<Vec<_>>()
                        .join(", ");

                    format!("{} ({})", self.total_count, breakdown)
                }
            }
            PackagesView::Full => {
                let breakdown = self
                    .managers
                    .iter()
                    .map(|manager| format!("{}: {}", manager.name, manager.count))
                    .collect::<Vec<_>>()
                    .join(", ");

                if breakdown.is_empty() {
                    self.total_count.to_string()
                } else {
                    format!("{} total [{}]", self.total_count, breakdown)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum ShellView {
    #[default]
    Name,
    Path,
    Full,
}

impl ShellInfo {
    pub fn render(&self, view: ShellView) -> String {
        match view {
            ShellView::Name => self.name.clone(),
            ShellView::Path => self.path.clone(),
            ShellView::Full => format!("{} ({})", self.name, self.path),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TerminalInfo {
    pub name: String,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum TerminalView {
    #[default]
    Name,
    Command,
    Full,
}

impl TerminalInfo {
    pub fn render(&self, view: TerminalView) -> String {
        match view {
            TerminalView::Name => self.name.clone(),
            TerminalView::Command => self.command.clone().unwrap_or_else(|| self.name.clone()),
            TerminalView::Full => match &self.command {
                Some(command) if command != &self.name => format!("{} ({})", self.name, command),
                _ => self.name.clone(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub model: String,
    pub core_count: Option<usize>,
    pub architecture: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum CpuView {
    Model,
    #[default]
    ModelCores,
    Full,
}

impl CpuInfo {
    pub fn render(&self, view: CpuView) -> String {
        match view {
            CpuView::Model => self.model.clone(),
            CpuView::ModelCores => match self.core_count {
                Some(core_count) => format!("{} ({} cores)", self.model, core_count),
                None => self.model.clone(),
            },
            CpuView::Full => {
                let mut value = self.render(CpuView::ModelCores);

                if let Some(architecture) = &self.architecture {
                    value.push_str(&format!(" [{}]", architecture));
                }

                value
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub model: String,
    pub vendor: Option<String>,
    pub core_count: Option<usize>,
    pub api: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum GpuView {
    Model,
    ModelCores,
    #[default]
    Full,
}

impl GpuInfo {
    pub fn render(&self, view: GpuView) -> String {
        match view {
            GpuView::Model => self.model.clone(),
            GpuView::ModelCores => match self.core_count {
                Some(core_count) => format!("{} ({} cores)", self.model, core_count),
                None => self.model.clone(),
            },
            GpuView::Full => {
                let mut parts = vec![self.render(GpuView::ModelCores)];

                if let Some(vendor) = &self.vendor {
                    parts.push(vendor.clone());
                }

                if let Some(api) = &self.api {
                    parts.push(api.clone());
                }

                parts.join(", ")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub used_bytes: Option<u64>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum MemoryView {
    Total,
    UsedTotal,
    #[default]
    Full,
}

impl MemoryInfo {
    pub fn render(&self, view: MemoryView) -> String {
        let total = format_bytes(self.total_bytes);

        match view {
            MemoryView::Total => total,
            MemoryView::UsedTotal => match self.used_bytes {
                Some(used_bytes) => format!("{} / {}", format_bytes(used_bytes), total),
                None => total,
            },
            MemoryView::Full => match self.used_bytes {
                Some(used_bytes) => {
                    let percent = percent(used_bytes, self.total_bytes);
                    format!("{} / {} ({}%)", format_bytes(used_bytes), total, percent)
                }
                None => total,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percent: u8,
    pub state: BatteryState,
    pub time_remaining: Option<u64>,
    pub condition: Option<String>,
    pub health_percent: Option<u8>,
}

#[derive(Debug, Clone)]
pub enum BatteryState {
    Charging,
    Discharging,
    Charged,
    NotCharging,
    Unknown(String),
}

impl BatteryState {
    fn from_str(value: &str) -> Self {
        let normalized = value.trim().to_ascii_lowercase();

        match normalized.as_str() {
            "charging" => BatteryState::Charging,
            "discharging" => BatteryState::Discharging,
            "charged" | "full" => BatteryState::Charged,
            "not charging" => BatteryState::NotCharging,
            _ => BatteryState::Unknown(normalized),
        }
    }

    fn render(&self) -> String {
        match self {
            BatteryState::Charging => "charging".to_string(),
            BatteryState::Discharging => "discharging".to_string(),
            BatteryState::Charged => "charged".to_string(),
            BatteryState::NotCharging => "not charging".to_string(),
            BatteryState::Unknown(value) => value.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum BatteryView {
    Percent,
    #[default]
    Status,
    Full,
}

impl BatteryInfo {
    pub fn render(&self, view: BatteryView) -> String {
        let percent = format!("{}%", self.percent);

        match view {
            BatteryView::Percent => percent,
            BatteryView::Status => {
                let mut parts = vec![self.state.render()];

                if let Some(time_remaining) = self.time_remaining {
                    parts.push(format!(
                        "{} remaining",
                        format_duration_short(time_remaining)
                    ));
                }

                format!("{} ({})", percent, parts.join(", "))
            }
            BatteryView::Full => {
                let mut parts = vec![self.state.render()];

                if let Some(time_remaining) = self.time_remaining {
                    parts.push(format!(
                        "{} remaining",
                        format_duration_short(time_remaining)
                    ));
                }

                if let Some(health_percent) = self.health_percent {
                    parts.push(format!("health {}%", health_percent));
                }

                if let Some(condition) = &self.condition {
                    parts.push(condition.clone());
                }

                format!("{} ({})", percent, parts.join(", "))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub device: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub usage_percent: Option<u8>,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum DiskView {
    UsedTotal,
    #[default]
    MountUsedTotal,
    Full,
}

impl DiskInfo {
    pub fn render(&self, view: DiskView) -> String {
        let usage = format!(
            "{} / {}",
            format_bytes(self.used_bytes),
            format_bytes(self.total_bytes)
        );

        match view {
            DiskView::UsedTotal => usage,
            DiskView::MountUsedTotal => format!("{} {}", self.mount_point, usage),
            DiskView::Full => {
                let mut value = format!("{} {}", self.mount_point, usage);

                if let Some(percent) = self.usage_percent {
                    value.push_str(&format!(" ({}%)", percent));
                }

                value.push_str(&format!(" on {}", self.device));
                value
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub name: String,
    pub resolution: String,
    pub is_main: bool,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum DisplayView {
    #[default]
    Resolution,
    NameResolution,
    Full,
}

impl DisplayInfo {
    pub fn render(&self, view: DisplayView) -> String {
        match view {
            DisplayView::Resolution => self.resolution.clone(),
            DisplayView::NameResolution => format!("{}: {}", self.name, self.resolution),
            DisplayView::Full => {
                let mut value = format!("{}: {}", self.name, self.resolution);

                if self.is_main {
                    value.push_str(" (main)");
                }

                value
            }
        }
    }
}

#[derive(Debug, Default)]
struct GraphicsBundle {
    gpus: Vec<GpuInfo>,
    displays: Vec<DisplayInfo>,
}

#[derive(Debug, Default)]
struct SystemFieldSelection {
    host: bool,
    os: bool,
    kernel: bool,
    uptime: bool,
    packages: bool,
    shell: bool,
    terminal: bool,
    cpu: bool,
    gpu: bool,
    memory: bool,
    battery: bool,
    disk: bool,
    display: bool,
}

#[derive(Debug)]
struct ProcessSnapshot {
    ppid: u32,
    command: String,
}

#[derive(Debug, Default)]
struct BatteryDetails {
    condition: Option<String>,
    health_percent: Option<u8>,
}

impl SystemFieldSelection {
    fn from_fields(fields: &[SystemField]) -> Self {
        let mut selection = Self::default();

        for field in fields {
            match field {
                SystemField::Host => selection.host = true,
                SystemField::Os => selection.os = true,
                SystemField::Kernel => selection.kernel = true,
                SystemField::Uptime => selection.uptime = true,
                SystemField::Packages => selection.packages = true,
                SystemField::Shell => selection.shell = true,
                SystemField::Terminal => selection.terminal = true,
                SystemField::Cpu => selection.cpu = true,
                SystemField::Gpu => selection.gpu = true,
                SystemField::Memory => selection.memory = true,
                SystemField::Battery => selection.battery = true,
                SystemField::Disk => selection.disk = true,
                SystemField::Display => selection.display = true,
            }
        }

        selection
    }
}

fn join_handle<T>(handle: Option<thread::JoinHandle<T>>) -> Option<T> {
    handle.and_then(|handle| handle.join().ok())
}

fn collect_host() -> Option<HostInfo> {
    #[cfg(target_os = "macos")]
    let name = successful_stdout("scutil", &["--get", "ComputerName"]);

    #[cfg(target_os = "linux")]
    let name = successful_stdout("hostnamectl", &["--static"])
        .or_else(|| successful_stdout("hostname", &[]))
        .or_else(|| {
            std::env::var("HOSTNAME")
                .ok()
                .filter(|value| !value.is_empty())
        });

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let name = successful_stdout("hostname", &[]).or_else(|| {
        std::env::var("HOSTNAME")
            .ok()
            .filter(|value| !value.is_empty())
    });

    let hostname = successful_stdout("hostname", &[]).or_else(|| {
        std::env::var("HOSTNAME")
            .ok()
            .filter(|value| !value.is_empty())
    });

    let name = name.or_else(|| hostname.clone())?;

    Some(HostInfo { name, hostname })
}

fn collect_kernel() -> Result<KernelInfo, process::ProcessError> {
    let output = process::run_checked("uname", &["-srm"])?;
    let mut parts = output.stdout.split_whitespace();

    Ok(KernelInfo {
        name: parts.next().unwrap_or("Unknown").to_string(),
        version: parts.next().unwrap_or("Unknown").to_string(),
        architecture: parts.next().unwrap_or(std::env::consts::ARCH).to_string(),
    })
}

#[cfg(target_os = "macos")]
fn collect_os() -> Result<OsInfo, process::ProcessError> {
    let output = process::run_checked("sw_vers", &[])?;
    let mut name = None;
    let mut version = None;
    let mut build = None;

    for line in output.stdout.lines() {
        if let Some((key, value)) = line.split_once(':') {
            match key.trim() {
                "ProductName" => name = Some(value.trim().to_string()),
                "ProductVersion" => version = Some(value.trim().to_string()),
                "BuildVersion" => build = Some(value.trim().to_string()),
                _ => {}
            }
        }
    }

    Ok(OsInfo {
        name: name.unwrap_or_else(|| "macOS".to_string()),
        version,
        build,
    })
}

#[cfg(target_os = "linux")]
fn collect_os() -> Result<OsInfo, process::ProcessError> {
    let content = fs::read_to_string("/etc/os-release")?;
    let fields = parse_os_release(&content);

    Ok(OsInfo {
        name: fields
            .get("NAME")
            .cloned()
            .unwrap_or_else(|| std::env::consts::OS.to_string()),
        version: fields.get("VERSION_ID").cloned(),
        build: fields.get("PRETTY_NAME").cloned(),
    })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn collect_os() -> Result<OsInfo, process::ProcessError> {
    Ok(OsInfo {
        name: std::env::consts::OS.to_string(),
        version: None,
        build: None,
    })
}

#[cfg(target_os = "macos")]
fn collect_uptime() -> Result<UptimeInfo, process::ProcessError> {
    let output = process::run_checked("sysctl", &["-n", "kern.boottime"])?;
    let boot_seconds = parse_kern_boottime_seconds(&output.stdout)
        .ok_or_else(|| invalid_data_error("failed to parse kern.boottime"))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(UptimeInfo {
        seconds: now.saturating_sub(boot_seconds),
    })
}

#[cfg(target_os = "linux")]
fn collect_uptime() -> Result<UptimeInfo, process::ProcessError> {
    let content = fs::read_to_string("/proc/uptime")?;
    let seconds = content
        .split_whitespace()
        .next()
        .and_then(|value| value.split('.').next())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| invalid_data_error("failed to parse /proc/uptime"))?;

    Ok(UptimeInfo { seconds })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn collect_uptime() -> Result<UptimeInfo, process::ProcessError> {
    Err(invalid_data_error(
        "uptime is not implemented for this platform",
    ))
}

fn collect_packages() -> Option<PackagesInfo> {
    let mut managers = Vec::new();

    #[cfg(target_os = "macos")]
    {
        push_package_count(&mut managers, "brew", "brew", &["list", "--formula"]);
        push_package_count(&mut managers, "cask", "brew", &["list", "--cask"]);
    }

    #[cfg(target_os = "linux")]
    {
        push_package_count(&mut managers, "pacman", "pacman", &["-Qq"]);
        push_package_count(&mut managers, "dpkg", "dpkg-query", &["-W"]);
        push_package_count(&mut managers, "rpm", "rpm", &["-qa"]);
        push_package_count(
            &mut managers,
            "flatpak",
            "flatpak",
            &["list", "--columns=application"],
        );
    }

    let total_count = managers.iter().map(|manager| manager.count).sum();

    (total_count > 0).then_some(PackagesInfo {
        total_count,
        managers,
    })
}

fn collect_shell() -> Option<ShellInfo> {
    let path = std::env::var("SHELL")
        .ok()
        .filter(|value| !value.is_empty())?;
    let name = Path::new(&path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(&path)
        .to_string();

    Some(ShellInfo { name, path })
}

fn collect_terminal() -> Option<TerminalInfo> {
    for variable in ["TERM_PROGRAM", "LC_TERMINAL"] {
        let value = std::env::var(variable)
            .ok()
            .filter(|value| is_meaningful_terminal_value(value));

        if let Some(value) = value {
            return Some(TerminalInfo {
                name: normalize_terminal_name(&value),
                command: None,
            });
        }
    }

    if let Some(command) = find_terminal_ancestor_command() {
        return Some(TerminalInfo {
            name: terminal_name_from_command(&command),
            command: Some(command),
        });
    }

    std::env::var("TERM")
        .ok()
        .filter(|value| is_meaningful_terminal_value(value))
        .map(|value| TerminalInfo {
            name: normalize_terminal_name(&value),
            command: None,
        })
}

#[cfg(target_os = "macos")]
fn collect_cpu() -> Result<CpuInfo, process::ProcessError> {
    let model = match process::run_checked("sysctl", &["-n", "machdep.cpu.brand_string"]) {
        Ok(output) => output.stdout,
        Err(_) => {
            let output = process::run_checked("system_profiler", &["SPHardwareDataType"])?;
            parse_hardware_field(&output.stdout, &["Chip", "Processor Name"])
        }
    };

    let core_count = process::run_checked("sysctl", &["-n", "hw.ncpu"])
        .ok()
        .and_then(|output| output.stdout.parse::<usize>().ok());

    let architecture = process::run_checked("uname", &["-m"])
        .ok()
        .map(|output| output.stdout)
        .filter(|value| !value.is_empty());

    Ok(CpuInfo {
        model,
        core_count,
        architecture,
    })
}

#[cfg(target_os = "linux")]
fn collect_cpu() -> Result<CpuInfo, process::ProcessError> {
    let content = fs::read_to_string("/proc/cpuinfo")?;
    let model = content
        .lines()
        .find_map(|line| {
            line.split_once(':').and_then(|(key, value)| {
                (key.trim() == "model name").then(|| value.trim().to_string())
            })
        })
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let core_count = process::run_checked("nproc", &[])
        .ok()
        .and_then(|output| output.stdout.parse::<usize>().ok());

    Ok(CpuInfo {
        model,
        core_count,
        architecture: Some(std::env::consts::ARCH.to_string()),
    })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn collect_cpu() -> Result<CpuInfo, process::ProcessError> {
    Ok(CpuInfo {
        model: "Unknown CPU".to_string(),
        core_count: None,
        architecture: Some(std::env::consts::ARCH.to_string()),
    })
}

#[cfg(target_os = "macos")]
fn collect_memory() -> Result<MemoryInfo, process::ProcessError> {
    let total_bytes = process::run_checked("sysctl", &["-n", "hw.memsize"])?
        .stdout
        .parse::<u64>()
        .map_err(|_| invalid_data_error("failed to parse hw.memsize"))?;
    let page_size = process::run_checked("sysctl", &["-n", "hw.pagesize"])?
        .stdout
        .parse::<u64>()
        .map_err(|_| invalid_data_error("failed to parse hw.pagesize"))?;
    let output = process::run_checked("vm_stat", &[])?;
    let used_pages = parse_vm_stat_used_pages(&output.stdout);
    let used_bytes = used_pages.map(|pages| pages.saturating_mul(page_size).min(total_bytes));

    Ok(MemoryInfo {
        used_bytes,
        total_bytes,
    })
}

#[cfg(target_os = "linux")]
fn collect_memory() -> Result<MemoryInfo, process::ProcessError> {
    let content = fs::read_to_string("/proc/meminfo")?;
    let total_kib = parse_meminfo_value(&content, "MemTotal")
        .ok_or_else(|| invalid_data_error("failed to parse MemTotal"))?;
    let available_kib = parse_meminfo_value(&content, "MemAvailable");
    let used_bytes = available_kib.map(|available| total_kib.saturating_sub(available) * 1024);

    Ok(MemoryInfo {
        used_bytes,
        total_bytes: total_kib * 1024,
    })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn collect_memory() -> Result<MemoryInfo, process::ProcessError> {
    Err(invalid_data_error(
        "memory is not implemented for this platform",
    ))
}

#[cfg(target_os = "macos")]
fn collect_battery() -> Option<BatteryInfo> {
    let output = process::run("pmset", &["-g", "batt"]).ok()?;

    if !output.success {
        return None;
    }

    let mut battery = parse_pmset_battery(&output.stdout)?;

    if let Ok(power) = process::run_checked(
        "system_profiler",
        &["SPPowerDataType", "-detailLevel", "mini"],
    ) {
        let details = parse_macos_battery_details(&power.stdout);

        battery.condition = details.condition;
        battery.health_percent = details.health_percent;
    }

    Some(battery)
}

#[cfg(target_os = "linux")]
fn collect_battery() -> Option<BatteryInfo> {
    let battery_dir = fs::read_dir("/sys/class/power_supply")
        .ok()?
        .flatten()
        .find(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| name.starts_with("BAT"))
                .unwrap_or(false)
        })?
        .path();

    let percent = read_u8_file(&battery_dir.join("capacity"))?;
    let state = fs::read_to_string(battery_dir.join("status"))
        .ok()
        .map(|value| BatteryState::from_str(value.trim()))
        .unwrap_or_else(|| BatteryState::Unknown("unknown".to_string()));
    let health_percent = read_u64_file(&battery_dir.join("charge_full"))
        .or_else(|| read_u64_file(&battery_dir.join("energy_full")))
        .zip(
            read_u64_file(&battery_dir.join("charge_full_design"))
                .or_else(|| read_u64_file(&battery_dir.join("energy_full_design"))),
        )
        .and_then(|(full, design)| u8::try_from(percent(full, design)).ok());

    Some(BatteryInfo {
        percent,
        state,
        time_remaining: None,
        condition: None,
        health_percent,
    })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn collect_battery() -> Option<BatteryInfo> {
    None
}

fn collect_disks() -> Result<Vec<DiskInfo>, process::ProcessError> {
    let output = process::run_checked("df", &["-kP", "/"])?;
    let root = parse_df_mount(&output.stdout)
        .ok_or_else(|| invalid_data_error("failed to parse df output"))?;

    Ok(vec![root])
}

#[cfg(target_os = "macos")]
fn collect_graphics_bundle() -> Result<GraphicsBundle, process::ProcessError> {
    let output = process::run_checked(
        "system_profiler",
        &["SPDisplaysDataType", "-detailLevel", "mini"],
    )?;

    Ok(GraphicsBundle {
        gpus: parse_macos_gpus(&output.stdout),
        displays: parse_macos_displays(&output.stdout),
    })
}

#[cfg(not(target_os = "macos"))]
fn collect_graphics_bundle() -> Result<GraphicsBundle, process::ProcessError> {
    Ok(GraphicsBundle::default())
}

fn push_package_count(
    managers: &mut Vec<PackageManagerCount>,
    name: &str,
    program: &str,
    args: &[&str],
) {
    if let Some(count) = command_line_count(program, args).filter(|count| *count > 0) {
        managers.push(PackageManagerCount {
            name: name.to_string(),
            count,
        });
    }
}

fn find_terminal_ancestor_command() -> Option<String> {
    let start_pid = std::process::id();
    let current_name = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .map(|value| value.to_ascii_lowercase());
    let mut pid = start_pid;

    for _ in 0..12 {
        let snapshot = inspect_process(pid)?;

        if pid != start_pid {
            let name = terminal_name_from_command(&snapshot.command).to_ascii_lowercase();

            if !is_runtime_process_name(&name, current_name.as_deref()) {
                return Some(snapshot.command);
            }
        }

        if snapshot.ppid == 0 || snapshot.ppid == pid {
            break;
        }

        pid = snapshot.ppid;
    }

    None
}

fn inspect_process(pid: u32) -> Option<ProcessSnapshot> {
    let pid = pid.to_string();
    let output = process::run("ps", &["-o", "ppid=", "-o", "comm=", "-p", pid.as_str()]).ok()?;

    if !output.success {
        return None;
    }

    parse_process_snapshot(&output.stdout)
}

fn parse_process_snapshot(output: &str) -> Option<ProcessSnapshot> {
    let line = output.lines().find(|line| !line.trim().is_empty())?;
    let trimmed = line.trim_start();
    let separator = trimmed.find(char::is_whitespace)?;
    let ppid = trimmed[..separator].trim().parse::<u32>().ok()?;
    let command = trimmed[separator..].trim().to_string();

    (!command.is_empty()).then_some(ProcessSnapshot { ppid, command })
}

fn is_runtime_process_name(name: &str, current_name: Option<&str>) -> bool {
    matches!(
        name,
        "cargo" | "rustc" | "bash" | "zsh" | "fish" | "sh" | "nu" | "sudo" | "env"
    ) || current_name.is_some_and(|current_name| current_name == name)
}

fn terminal_name_from_command(command: &str) -> String {
    if let Some(app_name) = command
        .split('/')
        .find_map(|part| part.strip_suffix(".app"))
    {
        return normalize_terminal_name(app_name);
    }

    let fallback = Path::new(command)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(command);

    normalize_terminal_name(fallback)
}

fn normalize_terminal_name(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches(".app");
    let normalized = trimmed.replace('_', " ").replace('-', " ");
    let should_title_case = normalized
        .chars()
        .all(|char| !char.is_alphabetic() || char.is_lowercase() || char.is_whitespace());

    if should_title_case {
        title_case(&normalized)
    } else {
        normalized
    }
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_uppercase().collect::<String>(),
                    chars.collect::<String>()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_meaningful_terminal_value(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed != "dumb"
}

fn parse_hardware_field(output: &str, keys: &[&str]) -> String {
    output
        .lines()
        .map(str::trim)
        .find_map(|line| {
            line.split_once(':').and_then(|(key, value)| {
                keys.iter()
                    .any(|candidate| *candidate == key.trim())
                    .then(|| value.trim().to_string())
            })
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn parse_macos_gpus(output: &str) -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    let mut current: Option<GpuInfo> = None;

    for line in output.lines() {
        let trimmed = line.trim();

        if let Some(model) = trimmed.strip_prefix("Chipset Model: ") {
            if let Some(gpu) = current.take() {
                gpus.push(gpu);
            }

            current = Some(GpuInfo {
                model: model.to_string(),
                vendor: None,
                core_count: None,
                api: None,
            });

            continue;
        }

        let Some(gpu) = current.as_mut() else {
            continue;
        };

        if let Some(vendor) = trimmed.strip_prefix("Vendor: ") {
            gpu.vendor = Some(vendor.to_string());
        } else if let Some(core_count) = trimmed.strip_prefix("Total Number of Cores: ") {
            gpu.core_count = parse_usize_prefix(core_count);
        } else if let Some(api) = trimmed.strip_prefix("Metal Support: ") {
            gpu.api = Some(api.to_string());
        }
    }

    if let Some(gpu) = current {
        gpus.push(gpu);
    }

    gpus
}

fn parse_macos_displays(output: &str) -> Vec<DisplayInfo> {
    let mut displays = Vec::new();
    let mut current: Option<DisplayInfo> = None;
    let mut in_displays_section = false;
    let mut displays_indent = 0usize;

    for line in output.lines() {
        let indent = line.chars().take_while(|char| char.is_whitespace()).count();
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "Displays:" {
            in_displays_section = true;
            displays_indent = indent;
            continue;
        }

        if in_displays_section && indent <= displays_indent {
            if let Some(display) = current
                .take()
                .filter(|display| !display.resolution.is_empty())
            {
                displays.push(display);
            }

            in_displays_section = false;
        }

        if !in_displays_section {
            continue;
        }

        if trimmed.ends_with(':') && !trimmed.contains(": ") && indent > displays_indent {
            if let Some(display) = current
                .take()
                .filter(|display| !display.resolution.is_empty())
            {
                displays.push(display);
            }

            current = Some(DisplayInfo {
                name: trimmed.trim_end_matches(':').to_string(),
                resolution: String::new(),
                is_main: false,
            });

            continue;
        }

        let Some(display) = current.as_mut() else {
            continue;
        };

        if let Some(resolution) = trimmed.strip_prefix("Resolution: ") {
            display.resolution = resolution.to_string();
        } else if let Some(value) = trimmed.strip_prefix("Main Display: ") {
            display.is_main = value == "Yes";
        }
    }

    if let Some(display) = current.filter(|display| !display.resolution.is_empty()) {
        displays.push(display);
    }

    displays
}

fn parse_vm_stat_used_pages(output: &str) -> Option<u64> {
    let mut active = None;
    let mut wired = None;
    let mut compressed = None;

    for line in output.lines() {
        let trimmed = line.trim();

        if let Some(value) = trimmed.strip_prefix("Pages active:") {
            active = parse_u64_prefix(value);
        } else if let Some(value) = trimmed.strip_prefix("Pages wired down:") {
            wired = parse_u64_prefix(value);
        } else if let Some(value) = trimmed.strip_prefix("Pages occupied by compressor:") {
            compressed = parse_u64_prefix(value);
        }
    }

    if active.is_none() && wired.is_none() && compressed.is_none() {
        return None;
    }

    Some(active.unwrap_or(0) + wired.unwrap_or(0) + compressed.unwrap_or(0))
}

fn parse_kern_boottime_seconds(output: &str) -> Option<u64> {
    let section = output.split("sec = ").nth(1)?;
    parse_u64_prefix(section)
}

fn parse_df_mount(output: &str) -> Option<DiskInfo> {
    let line = output.lines().nth(1)?;
    let parts = line.split_whitespace().collect::<Vec<_>>();

    if parts.len() < 6 {
        return None;
    }

    Some(DiskInfo {
        device: parts[0].to_string(),
        total_bytes: parts[1].parse::<u64>().ok()?.saturating_mul(1024),
        used_bytes: parts[2].parse::<u64>().ok()?.saturating_mul(1024),
        usage_percent: parts[4].trim_end_matches('%').parse::<u8>().ok(),
        mount_point: parts[5].to_string(),
    })
}

fn parse_pmset_battery(output: &str) -> Option<BatteryInfo> {
    if output.contains("No batteries") {
        return None;
    }

    let line = output.lines().find(|line| line.contains('%'))?;
    let percent = line
        .split('%')
        .next()
        .and_then(|section| section.split_whitespace().last())
        .and_then(|value| value.parse::<u8>().ok())?;
    let parts = line.split(';').map(str::trim).collect::<Vec<_>>();
    let state = parts
        .get(1)
        .map(|value| BatteryState::from_str(value))
        .unwrap_or_else(|| BatteryState::Unknown("unknown".to_string()));
    let time_remaining = parts
        .get(2)
        .and_then(|value| parse_pmset_time_remaining(value));

    Some(BatteryInfo {
        percent,
        state,
        time_remaining,
        condition: None,
        health_percent: None,
    })
}

fn parse_pmset_time_remaining(value: &str) -> Option<u64> {
    let normalized = value.trim().to_ascii_lowercase();

    if normalized.contains("no estimate") || normalized.contains("calculating") {
        return None;
    }

    normalized
        .split_whitespace()
        .find(|token| token.contains(':'))
        .and_then(parse_hhmm_to_seconds)
}

fn parse_macos_battery_details(output: &str) -> BatteryDetails {
    let mut details = BatteryDetails::default();

    for line in output.lines().map(str::trim) {
        if let Some(condition) = line.strip_prefix("Condition: ") {
            details.condition = Some(condition.to_string());
        } else if let Some(capacity) = line.strip_prefix("Maximum Capacity: ") {
            details.health_percent =
                parse_u64_prefix(capacity).and_then(|value| u8::try_from(value).ok());
        }
    }

    details
}

#[cfg(target_os = "linux")]
fn parse_meminfo_value(content: &str, key: &str) -> Option<u64> {
    content.lines().find_map(|line| {
        line.strip_prefix(&format!("{}:", key))
            .and_then(parse_u64_prefix)
    })
}

#[cfg(target_os = "linux")]
fn parse_os_release(content: &str) -> std::collections::HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| {
            (
                key.trim().to_string(),
                value.trim().trim_matches('"').to_string(),
            )
        })
        .collect()
}

fn successful_stdout(program: &str, args: &[&str]) -> Option<String> {
    process::run(program, args)
        .ok()
        .filter(|output| output.success)
        .map(|output| output.stdout)
        .filter(|value| !value.is_empty())
}

fn command_line_count(program: &str, args: &[&str]) -> Option<usize> {
    process::run(program, args)
        .ok()
        .filter(|output| output.success)
        .map(|output| {
            output
                .stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
        })
}

#[cfg(target_os = "linux")]
fn read_u64_file(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

#[cfg(target_os = "linux")]
fn read_u8_file(path: &Path) -> Option<u8> {
    fs::read_to_string(path).ok()?.trim().parse::<u8>().ok()
}

fn parse_u64_prefix(value: &str) -> Option<u64> {
    let token = value
        .trim()
        .split_whitespace()
        .next()?
        .trim_end_matches(|char: char| !char.is_ascii_digit());

    token.parse().ok()
}

fn parse_usize_prefix(value: &str) -> Option<usize> {
    parse_u64_prefix(value).and_then(|value| usize::try_from(value).ok())
}

fn parse_hhmm_to_seconds(value: &str) -> Option<u64> {
    let (hours, minutes) = value.split_once(':')?;
    let hours = hours.parse::<u64>().ok()?;
    let minutes = minutes.parse::<u64>().ok()?;

    Some(hours * 3_600 + minutes * 60)
}

fn percent(used: u64, total: u64) -> u64 {
    if total == 0 {
        return 0;
    }

    used.saturating_mul(100) / total
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    if bytes < 1024 {
        return format!("{} B", bytes);
    }

    let mut value = bytes as f64;
    let mut unit_index = 0usize;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", value, UNITS[unit_index])
}

fn format_duration_short(total_seconds: u64) -> String {
    let (days, hours, minutes, seconds) = duration_parts(total_seconds);
    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{}d", days));
    }

    if hours > 0 {
        parts.push(format!("{}h", hours));
    }

    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }

    if parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}

fn format_duration_long(total_seconds: u64) -> String {
    let (days, hours, minutes, seconds) = duration_parts(total_seconds);
    let mut parts = Vec::new();

    if days > 0 {
        parts.push(pluralize(days, "day"));
    }

    if hours > 0 {
        parts.push(pluralize(hours, "hour"));
    }

    if minutes > 0 {
        parts.push(pluralize(minutes, "minute"));
    }

    if parts.is_empty() {
        parts.push(pluralize(seconds, "second"));
    }

    parts.join(", ")
}

fn duration_parts(total_seconds: u64) -> (u64, u64, u64, u64) {
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    (days, hours, minutes, seconds)
}

fn pluralize(value: u64, noun: &str) -> String {
    if value == 1 {
        format!("1 {}", noun)
    } else {
        format!("{} {}s", value, noun)
    }
}

fn invalid_data_error(message: &str) -> process::ProcessError {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message).into()
}

#[cfg(test)]
mod tests {
    use super::{
        BatteryState, DisplayView, GpuView, PackagesInfo, PackagesView, parse_df_mount,
        parse_hhmm_to_seconds, parse_kern_boottime_seconds, parse_macos_battery_details,
        parse_macos_displays, parse_macos_gpus, parse_pmset_battery, parse_process_snapshot,
        parse_u64_prefix, parse_usize_prefix, parse_vm_stat_used_pages, terminal_name_from_command,
    };

    #[test]
    fn parses_gpu_blocks_from_system_profiler() {
        let sample = r#"
Graphics/Displays:

    Apple M2:
      Chipset Model: Apple M2
      Type: GPU
      Total Number of Cores: 8
      Vendor: Apple (0x106b)
      Metal Support: Metal 4

    Radeon Pro 560X:
      Chipset Model: Radeon Pro 560X
      Type: GPU
      Total Number of Cores: 16
      Vendor: AMD (0x1002)
      Metal Support: Supported
"#;

        let gpus = parse_macos_gpus(sample);

        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].render(GpuView::Model), "Apple M2");
        assert_eq!(
            gpus[1].render(GpuView::ModelCores),
            "Radeon Pro 560X (16 cores)"
        );
    }

    #[test]
    fn parses_display_blocks_from_system_profiler() {
        let sample = r#"
Graphics/Displays:

    Apple M2:
      Chipset Model: Apple M2
      Displays:
        Color LCD:
          Resolution: 2560 x 1664 Retina
          Main Display: Yes
        Studio Display:
          Resolution: 5120 x 2880
          Main Display: No
"#;

        let displays = parse_macos_displays(sample);

        assert_eq!(displays.len(), 2);
        assert_eq!(
            displays[0].render(DisplayView::Resolution),
            "2560 x 1664 Retina"
        );
        assert_eq!(
            displays[1].render(DisplayView::NameResolution),
            "Studio Display: 5120 x 2880"
        );
    }

    #[test]
    fn parses_vm_stat_used_pages() {
        let sample = r#"
Mach Virtual Memory Statistics: (page size of 16384 bytes)
Pages active:                            309222.
Pages wired down:                        135551.
Pages occupied by compressor:            215128.
"#;

        assert_eq!(
            parse_vm_stat_used_pages(sample),
            Some(309_222 + 135_551 + 215_128)
        );
    }

    #[test]
    fn parses_boot_time_seconds() {
        let sample = "{ sec = 1775303062, usec = 645741 } Sat Apr  4 14:44:22 2026";

        assert_eq!(parse_kern_boottime_seconds(sample), Some(1_775_303_062));
    }

    #[test]
    fn parses_df_output() {
        let sample = "\
Filesystem     1024-blocks      Used Available Capacity  Mounted on
/dev/disk3s1s1   239362496  24366600  15312340    62%    /
";

        let disk = parse_df_mount(sample).expect("disk info");

        assert_eq!(disk.mount_point, "/");
        assert_eq!(disk.device, "/dev/disk3s1s1");
        assert_eq!(disk.usage_percent, Some(62));
    }

    #[test]
    fn parses_pmset_output() {
        let sample = "\
Now drawing from 'Battery Power'
 -InternalBattery-0 (id=23068771)\t51%; discharging; 2:02 remaining present: true
";

        let battery = parse_pmset_battery(sample).expect("battery info");

        assert_eq!(battery.percent, 51);
        assert!(matches!(battery.state, BatteryState::Discharging));
        assert_eq!(battery.time_remaining, Some(7_320));
    }

    #[test]
    fn parses_battery_details() {
        let sample = r#"
Power:
    Battery Information:
      Health Information:
          Condition: Normal
          Maximum Capacity: 100 %
"#;

        let details = parse_macos_battery_details(sample);

        assert_eq!(details.condition.as_deref(), Some("Normal"));
        assert_eq!(details.health_percent, Some(100));
    }

    #[test]
    fn parses_process_snapshot_with_spaces() {
        let sample = " 3968 /Applications/Visual Studio Code.app/Contents/MacOS/Electron";
        let snapshot = parse_process_snapshot(sample).expect("process snapshot");

        assert_eq!(snapshot.ppid, 3968);
        assert_eq!(
            snapshot.command,
            "/Applications/Visual Studio Code.app/Contents/MacOS/Electron"
        );
    }

    #[test]
    fn normalizes_terminal_name_from_command() {
        let name = terminal_name_from_command("/Applications/Codex.app/Contents/Resources/codex");

        assert_eq!(name, "Codex");
    }

    #[test]
    fn renders_package_breakdown() {
        let packages = PackagesInfo {
            total_count: 145,
            managers: vec![
                super::PackageManagerCount {
                    name: "brew".to_string(),
                    count: 142,
                },
                super::PackageManagerCount {
                    name: "cask".to_string(),
                    count: 3,
                },
            ],
        };

        assert_eq!(packages.render(PackagesView::Total), "145");
        assert_eq!(
            packages.render(PackagesView::Compact),
            "145 (brew 142, cask 3)"
        );
    }

    #[test]
    fn parses_numeric_prefixes() {
        assert_eq!(parse_u64_prefix("42973."), Some(42_973));
        assert_eq!(parse_usize_prefix("12 cores"), Some(12));
        assert_eq!(parse_usize_prefix("unknown"), None);
    }

    #[test]
    fn parses_hhmm_durations() {
        assert_eq!(parse_hhmm_to_seconds("2:02"), Some(7_320));
        assert_eq!(parse_hhmm_to_seconds("0:45"), Some(2_700));
    }
}
