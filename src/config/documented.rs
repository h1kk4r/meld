use std::fmt::Write;

use crate::modules::git::GitView;
use crate::modules::spotify::SpotifyConfig;
use crate::modules::system::{
    BatteryView, CpuView, DiskView, DisplayView, GpuView, HostView, KernelView, MemoryView, OsView,
    PackagesView, ShellView, SystemField, SystemRenderConfig, TerminalView, UptimeView,
};
use crate::render::blocks::ColorBlocksConfig;
use crate::render::image::{ImageConfig, ImageCropMode, ImageHeight};
use crate::render::logo::{LogoConfig, LogoPreset, LogoSize};
use crate::render::style::{ColorSpec, TextCase, TextColors};

use super::schema::{LineLabels, OutputConfig, OutputHookConfig, OutputItemConfig};
use super::{AppConfig, LineKey};

pub(crate) fn render(config: &AppConfig) -> String {
    let mut output = String::new();

    writeln!(&mut output, "-- Meld init.lua").unwrap();
    writeln!(&mut output, "--").unwrap();
    writeln!(
        &mut output,
        "-- This file is a real Lua script. It can either:"
    )
    .unwrap();
    writeln!(
        &mut output,
        "--   1. assign fields on the global `meld` / `config` table, or"
    )
    .unwrap();
    writeln!(&mut output, "--   2. return a table at the end.").unwrap();
    writeln!(&mut output, "--").unwrap();
    writeln!(&mut output, "-- Useful CLI flags:").unwrap();
    writeln!(&mut output, "--   meld --show-config-path").unwrap();
    writeln!(&mut output, "--   meld --diagnostics").unwrap();
    writeln!(&mut output, "--   meld --config /path/to/init.lua").unwrap();
    writeln!(&mut output, "--   meld --print-config").unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "local config = meld").unwrap();
    writeln!(&mut output).unwrap();

    render_layout_section(&mut output, config);
    render_text_section(&mut output, config);
    render_output_section(&mut output, &config.output);
    render_colors_section(&mut output, &config.colors);
    render_logo_section(&mut output, &config.logo);
    render_image_section(&mut output, &config.image);
    render_spotify_section(&mut output, &config.spotify);
    render_blocks_section(&mut output, &config.blocks);
    render_labels_section(&mut output, &config.labels);
    render_order_section(&mut output, &config.order);
    render_git_section(&mut output, config.git_view);
    render_system_sections(&mut output, &config.system_views);
    render_logo_color_help(&mut output);

    output
}

fn render_layout_section(output: &mut String, config: &AppConfig) {
    writeln!(output, "-- Layout").unwrap();
    writeln!(
        output,
        "-- Controls how labels and values are glued together."
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- align:").unwrap();
    writeln!(
        output,
        "--   true  -> labels are padded to the longest label"
    )
    .unwrap();
    writeln!(output, "--   false -> labels keep their natural width").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- separator:").unwrap();
    writeln!(
        output,
        "--   Literal separator symbol. Spaces are added automatically around it."
    )
    .unwrap();
    writeln!(output, "--   \":\" -> Shell : fish").unwrap();
    writeln!(output, "--   \"|\" -> Shell | fish").unwrap();
    writeln!(output, "--   \"\"  -> Shell fish").unwrap();
    writeln!(output, "config.layout = {{").unwrap();
    writeln!(output, "  align = {},", lua_bool(config.layout.align)).unwrap();
    writeln!(
        output,
        "  separator = {},",
        lua_inline_string(&config.layout.separator)
    )
    .unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_text_section(output: &mut String, config: &AppConfig) {
    writeln!(output, "-- Text transform").unwrap();
    writeln!(output, "-- Force the rendered info text into one case.").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Values:").unwrap();
    writeln!(output, "--   \"default\" -> keep the original casing").unwrap();
    writeln!(output, "--   \"lower\"   -> everything becomes lowercase").unwrap();
    writeln!(output, "--   \"upper\"   -> everything becomes uppercase").unwrap();
    writeln!(output, "config.text = {{").unwrap();
    writeln!(
        output,
        "  case = {},",
        lua_inline_string(text_case_name(config.text_style.case))
    )
    .unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_output_section(output: &mut String, config: &OutputConfig) {
    writeln!(output, "-- Extra output hooks").unwrap();
    writeln!(
        output,
        "-- Add plain text or shell command output before/after the Meld block."
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(
        output,
        "-- Commands run through `$SHELL -lc` during normal rendering."
    )
    .unwrap();
    writeln!(
        output,
        "-- Prefer this section over calling `print()` directly in init.lua:"
    )
    .unwrap();
    writeln!(
        output,
        "-- direct prints happen while the config is parsed and can pollute diagnostics."
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Ordered hook items:").unwrap();
    writeln!(output, "--   \"hello\"").unwrap();
    writeln!(output, "--   {{ command = \"ls\" }}").unwrap();
    writeln!(output, "config.output = {{").unwrap();
    render_output_hook(output, "before", &config.before);
    render_output_hook(output, "after", &config.after);
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_output_hook(output: &mut String, key: &str, hook: &OutputHookConfig) {
    writeln!(output, "  {} = {{", key).unwrap();

    if hook.items.is_empty() {
        writeln!(output, "    -- {},", lua_inline_string("hello")).unwrap();
        writeln!(
            output,
            "    -- {{ command = {} }},",
            lua_inline_string("date '+%H:%M'")
        )
        .unwrap();
    } else {
        for item in &hook.items {
            match item {
                OutputItemConfig::Text(text) => {
                    writeln!(output, "    {},", lua_output_text_value(text)).unwrap();
                }
                OutputItemConfig::Command(command) => {
                    writeln!(
                        output,
                        "    {{ command = {} }},",
                        lua_inline_string(command)
                    )
                    .unwrap();
                }
            }
        }
    }

    writeln!(output, "  }},").unwrap();
}

fn lua_output_text_value(value: &str) -> String {
    if value
        .chars()
        .all(|character| matches!(character, '\r' | '\n'))
    {
        return lua_inline_string(value);
    }

    lua_string_value(value, 4)
}

fn render_colors_section(output: &mut String, colors: &TextColors) {
    writeln!(output, "-- Text colors").unwrap();
    writeln!(
        output,
        "-- Color each part of the rendered line separately."
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Supported values:").unwrap();
    writeln!(
        output,
        "--   color names: \"red\", \"blue\", \"bright_black\", \"bright_white\""
    )
    .unwrap();
    writeln!(output, "--   ansi indexes: 0..15").unwrap();
    writeln!(output, "-- Omit a key to leave that part uncolored.").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Notes:").unwrap();
    writeln!(
        output,
        "-- - `label` colors the label before the separator."
    )
    .unwrap();
    writeln!(output, "-- - `separator` colors only the separator string.").unwrap();
    writeln!(output, "-- - `value` colors the text after the separator.").unwrap();
    writeln!(output, "config.colors = {{").unwrap();
    if let Some(color) = colors.label {
        writeln!(
            output,
            "  label = {},",
            lua_inline_string(color_name(color))
        )
        .unwrap();
    }
    if let Some(color) = colors.separator {
        writeln!(
            output,
            "  separator = {},",
            lua_inline_string(color_name(color))
        )
        .unwrap();
    }
    if let Some(color) = colors.value {
        writeln!(
            output,
            "  value = {},",
            lua_inline_string(color_name(color))
        )
        .unwrap();
    }
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_logo_section(output: &mut String, logo: &LogoConfig) {
    writeln!(output, "-- Logo / custom visual").unwrap();
    writeln!(
        output,
        "-- Left-side visual, similar to the distro logo in neofetch."
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Supported presets:").unwrap();
    writeln!(output, "--   \"macos\"").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Supported sizes:").unwrap();
    writeln!(output, "--   \"small\", \"large\"").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(
        output,
        "-- `padding` controls the gap between the logo and the info column."
    )
    .unwrap();
    writeln!(output, "config.logo = {{").unwrap();
    writeln!(output, "  enabled = {},", lua_bool(logo.enabled)).unwrap();
    writeln!(
        output,
        "  preset = {},",
        lua_inline_string(logo_preset_name(logo.preset))
    )
    .unwrap();
    writeln!(
        output,
        "  size = {},",
        lua_inline_string(logo_size_name(logo.size))
    )
    .unwrap();
    writeln!(output, "  padding = {},", logo.padding).unwrap();
    if let Some(text) = &logo.text {
        writeln!(output, "  text = {},", lua_string_value(text, 2)).unwrap();
    }
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_image_section(output: &mut String, image: &ImageConfig) {
    writeln!(output, "-- WezTerm image").unwrap();
    writeln!(
        output,
        "-- If enabled and running inside WezTerm, this replaces the ASCII logo block."
    )
    .unwrap();
    writeln!(output, "-- Backend priority is:").unwrap();
    writeln!(
        output,
        "--   spotify cover -> image -> logo text -> logo preset -> none"
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- path:").unwrap();
    writeln!(
        output,
        "--   absolute path, or a path relative to this init.lua file"
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- height:").unwrap();
    writeln!(output, "--   number -> fixed height in terminal rows").unwrap();
    writeln!(
        output,
        "--   \"auto\" -> match the rendered info block height"
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- crop:").unwrap();
    writeln!(
        output,
        "--   \"center\" -> center-crop to the terminal image slot"
    )
    .unwrap();
    writeln!(
        output,
        "--   \"none\"   -> keep the full image without cropping"
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- padding:").unwrap();
    writeln!(
        output,
        "--   gap between the image area and the info column"
    )
    .unwrap();
    writeln!(output, "config.image = {{").unwrap();
    writeln!(output, "  enabled = {},", lua_bool(image.enabled)).unwrap();
    if let Some(path) = image.path.as_ref() {
        writeln!(
            output,
            "  path = {},",
            lua_inline_string(&path.to_string_lossy())
        )
        .unwrap();
    }
    writeln!(output, "  height = {},", lua_image_height(image.height)).unwrap();
    writeln!(
        output,
        "  crop = {},",
        lua_inline_string(image_crop_name(image.crop))
    )
    .unwrap();
    writeln!(output, "  padding = {},", image.padding).unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_spotify_section(output: &mut String, spotify: &SpotifyConfig) {
    writeln!(output, "-- Spotify sync").unwrap();
    writeln!(
        output,
        "-- Adds a music line to `order` and can also replace the image slot"
    )
    .unwrap();
    writeln!(
        output,
        "-- with the current track cover when running inside WezTerm."
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Supported placeholders in `format`:").unwrap();
    writeln!(output, "--   $artist").unwrap();
    writeln!(output, "--   $track").unwrap();
    writeln!(output, "--   $album").unwrap();
    writeln!(output, "--   $state").unwrap();
    writeln!(output, "--   $id").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(
        output,
        "-- Set `client_id` to your Spotify app client ID (`api_key` is also accepted),"
    )
    .unwrap();
    writeln!(output, "-- then run:").unwrap();
    writeln!(output, "--   meld --spotify-login").unwrap();
    writeln!(
        output,
        "-- The OAuth token is stored as spotify-token.json next to init.lua and read automatically."
    )
    .unwrap();
    writeln!(
        output,
        "-- API timeouts fall back silently, so normal output does not wait long."
    )
    .unwrap();
    writeln!(output, "config.spotify = {{").unwrap();
    writeln!(
        output,
        "  client_id = {},",
        lua_optional_inline_string(spotify.client_id.as_deref())
    )
    .unwrap();
    writeln!(
        output,
        "  redirect_uri = {},",
        lua_inline_string(&spotify.redirect_uri)
    )
    .unwrap();
    writeln!(output, "  format = {},", lua_inline_string(&spotify.format)).unwrap();
    writeln!(
        output,
        "  cover_as_image = {},",
        lua_bool(spotify.cover_as_image)
    )
    .unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_blocks_section(output: &mut String, blocks: &ColorBlocksConfig) {
    writeln!(output, "-- ANSI color blocks").unwrap();
    writeln!(
        output,
        "-- Prints 2 rows of the standard 0..15 ANSI colors after the info column."
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- width:").unwrap();
    writeln!(output, "--   how many times to repeat `symbol` per color").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- height:").unwrap();
    writeln!(
        output,
        "--   how many identical rows to print per palette row"
    )
    .unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- symbol:").unwrap();
    writeln!(output, "--   the character(s) that make up each block").unwrap();
    writeln!(output, "--   \"█\" gives a classic filled block look").unwrap();
    writeln!(output, "config.blocks = {{").unwrap();
    writeln!(output, "  enabled = {},", lua_bool(blocks.enabled)).unwrap();
    writeln!(output, "  width = {},", blocks.width).unwrap();
    writeln!(output, "  height = {},", blocks.height).unwrap();
    writeln!(output, "  symbol = {},", lua_inline_string(&blocks.symbol)).unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_labels_section(output: &mut String, labels: &LineLabels) {
    writeln!(output, "-- Labels").unwrap();
    writeln!(output, "-- Custom labels for each line.").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Example:").unwrap();
    writeln!(output, "--   shell = \"Shell\"").unwrap();
    writeln!(
        output,
        "-- With the default layout and shell value placeholder `$shell`,"
    )
    .unwrap();
    writeln!(output, "-- the resulting line shape is:").unwrap();
    writeln!(output, "--   Shell : $shell").unwrap();
    writeln!(output, "config.labels = {{").unwrap();
    writeln!(
        output,
        "  directory = {},",
        lua_inline_string(&labels.directory)
    )
    .unwrap();
    writeln!(output, "  git = {},", lua_inline_string(&labels.git)).unwrap();
    writeln!(
        output,
        "  spotify = {},",
        lua_inline_string(&labels.spotify)
    )
    .unwrap();
    writeln!(output, "  host = {},", lua_inline_string(&labels.host)).unwrap();
    writeln!(output, "  os = {},", lua_inline_string(&labels.os)).unwrap();
    writeln!(output, "  kernel = {},", lua_inline_string(&labels.kernel)).unwrap();
    writeln!(output, "  uptime = {},", lua_inline_string(&labels.uptime)).unwrap();
    writeln!(
        output,
        "  packages = {},",
        lua_inline_string(&labels.packages)
    )
    .unwrap();
    writeln!(output, "  shell = {},", lua_inline_string(&labels.shell)).unwrap();
    writeln!(
        output,
        "  terminal = {},",
        lua_inline_string(&labels.terminal)
    )
    .unwrap();
    writeln!(output, "  cpu = {},", lua_inline_string(&labels.cpu)).unwrap();
    writeln!(output, "  gpu = {},", lua_inline_string(&labels.gpu)).unwrap();
    writeln!(output, "  memory = {},", lua_inline_string(&labels.memory)).unwrap();
    writeln!(
        output,
        "  battery = {},",
        lua_inline_string(&labels.battery)
    )
    .unwrap();
    writeln!(output, "  disk = {},", lua_inline_string(&labels.disk)).unwrap();
    writeln!(
        output,
        "  display = {},",
        lua_inline_string(&labels.display)
    )
    .unwrap();
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_order_section(output: &mut String, order: &[LineKey]) {
    writeln!(output, "-- Order").unwrap();
    writeln!(output, "-- Choose which lines appear first.").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Notes:").unwrap();
    writeln!(output, "-- - Only the lines listed here are shown.").unwrap();
    writeln!(output, "-- - Comment a line out here to hide it.").unwrap();
    writeln!(output, "config.order = {{").unwrap();
    for line in order {
        writeln!(output, "  {},", lua_inline_string(line_key_name(*line))).unwrap();
    }
    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn render_git_section(output: &mut String, git: GitView) {
    writeln!(output, "-- Git").unwrap();
    writeln!(output, "-- Render the current git context.").unwrap();
    writeln!(output, "--").unwrap();
    writeln!(output, "-- Default: \"branch_or_commit\"").unwrap();
    writeln!(output, "-- Values:  \"branch\", \"branch_or_commit\"").unwrap();
    writeln!(
        output,
        "config.git = {}",
        lua_inline_string(git_view_name(git))
    )
    .unwrap();
    writeln!(output).unwrap();
}

fn render_system_sections(output: &mut String, views: &SystemRenderConfig) {
    render_system_view(
        output,
        "Host",
        "host",
        "full",
        &["name", "hostname", "full"],
        host_view_name(views.host),
    );
    render_system_view(
        output,
        "OS",
        "os",
        "name_version",
        &["name", "version", "name_version", "full"],
        os_view_name(views.os),
    );
    render_system_view(
        output,
        "Kernel",
        "kernel",
        "full",
        &["name", "version", "name_version", "full"],
        kernel_view_name(views.kernel),
    );
    render_system_view(
        output,
        "Uptime",
        "uptime",
        "short",
        &["short", "full"],
        uptime_view_name(views.uptime),
    );
    render_system_view(
        output,
        "Packages",
        "packages",
        "compact",
        &["total", "compact", "full"],
        packages_view_name(views.packages),
    );
    render_system_view(
        output,
        "Shell",
        "shell",
        "name",
        &["name", "path", "full"],
        shell_view_name(views.shell),
    );
    render_system_view(
        output,
        "Terminal",
        "terminal",
        "name",
        &["name", "command", "full"],
        terminal_view_name(views.terminal),
    );
    render_system_view(
        output,
        "CPU",
        "cpu",
        "model_cores",
        &["model", "model_cores", "full"],
        cpu_view_name(views.cpu),
    );
    render_system_view(
        output,
        "GPU",
        "gpu",
        "full",
        &["model", "model_cores", "full"],
        gpu_view_name(views.gpu),
    );
    render_system_view(
        output,
        "Memory",
        "memory",
        "full",
        &["total", "used_total", "full"],
        memory_view_name(views.memory),
    );
    render_system_view(
        output,
        "Battery",
        "battery",
        "status",
        &["percent", "status", "full"],
        battery_view_name(views.battery),
    );
    render_system_view(
        output,
        "Disk",
        "disk",
        "mount_used_total",
        &["used_total", "mount_used_total", "full"],
        disk_view_name(views.disk),
    );
    render_system_view(
        output,
        "Display",
        "display",
        "resolution",
        &["resolution", "name_resolution", "full"],
        display_view_name(views.display),
    );
}

fn render_system_view(
    output: &mut String,
    title: &str,
    key: &str,
    default: &str,
    values: &[&str],
    current: &str,
) {
    writeln!(output, "-- {}", title).unwrap();
    writeln!(output, "-- Default: {}", lua_inline_string(default)).unwrap();
    writeln!(
        output,
        "-- Values:  {}",
        values
            .iter()
            .map(|value| lua_inline_string(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();
    writeln!(output, "config.{} = {}", key, lua_inline_string(current)).unwrap();
    writeln!(output).unwrap();
}

fn render_logo_color_help(output: &mut String) {
    writeln!(output, "-- Custom ANSI coloring").unwrap();
    writeln!(
        output,
        "-- You can color custom logo text with raw ANSI escapes in normal Lua strings."
    )
    .unwrap();
    writeln!(output, "-- Example:").unwrap();
    writeln!(output, "--   local esc = string.char(27)").unwrap();
    writeln!(output, "--   config.logo.text = table.concat({{").unwrap();
    writeln!(output, "--     esc .. \"[34m██\" .. esc .. \"[0m\",").unwrap();
    writeln!(output, "--     esc .. \"[36m██\" .. esc .. \"[0m\",").unwrap();
    writeln!(output, "--   }}, \"\\n\")").unwrap();
}

fn lua_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn lua_inline_string(value: &str) -> String {
    format!("\"{}\"", escape_lua(value))
}

fn lua_optional_inline_string(value: Option<&str>) -> String {
    value
        .map(lua_inline_string)
        .unwrap_or_else(|| "\"\"".to_string())
}

fn lua_string_value(value: &str, indent: usize) -> String {
    if !value.contains('\n') {
        return lua_inline_string(value);
    }

    let indentation = " ".repeat(indent);
    let mut rendered = String::new();
    rendered.push_str("table.concat({\n");

    for line in value.split('\n') {
        let _ = writeln!(
            &mut rendered,
            "{}  {},",
            indentation,
            lua_inline_string(line)
        );
    }

    let _ = write!(&mut rendered, "{}}}, \"\\n\")", indentation);
    rendered
}

fn escape_lua(value: &str) -> String {
    let mut escaped = String::new();

    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{1b}' => escaped.push_str("\\27"),
            _ => escaped.push(character),
        }
    }

    escaped
}

fn line_key_name(line: LineKey) -> &'static str {
    match line {
        LineKey::Directory => "directory",
        LineKey::Git => "git",
        LineKey::Spotify => "spotify",
        LineKey::System(SystemField::Host) => "host",
        LineKey::System(SystemField::Os) => "os",
        LineKey::System(SystemField::Kernel) => "kernel",
        LineKey::System(SystemField::Uptime) => "uptime",
        LineKey::System(SystemField::Packages) => "packages",
        LineKey::System(SystemField::Shell) => "shell",
        LineKey::System(SystemField::Terminal) => "terminal",
        LineKey::System(SystemField::Cpu) => "cpu",
        LineKey::System(SystemField::Gpu) => "gpu",
        LineKey::System(SystemField::Memory) => "memory",
        LineKey::System(SystemField::Battery) => "battery",
        LineKey::System(SystemField::Disk) => "disk",
        LineKey::System(SystemField::Display) => "display",
    }
}

fn color_name(color: ColorSpec) -> &'static str {
    match color {
        ColorSpec::Ansi(0) => "black",
        ColorSpec::Ansi(1) => "red",
        ColorSpec::Ansi(2) => "green",
        ColorSpec::Ansi(3) => "yellow",
        ColorSpec::Ansi(4) => "blue",
        ColorSpec::Ansi(5) => "magenta",
        ColorSpec::Ansi(6) => "cyan",
        ColorSpec::Ansi(7) => "white",
        ColorSpec::Ansi(8) => "bright_black",
        ColorSpec::Ansi(9) => "bright_red",
        ColorSpec::Ansi(10) => "bright_green",
        ColorSpec::Ansi(11) => "bright_yellow",
        ColorSpec::Ansi(12) => "bright_blue",
        ColorSpec::Ansi(13) => "bright_magenta",
        ColorSpec::Ansi(14) => "bright_cyan",
        ColorSpec::Ansi(15) => "bright_white",
        _ => "white",
    }
}

fn text_case_name(value: TextCase) -> &'static str {
    match value {
        TextCase::Default => "default",
        TextCase::Lower => "lower",
        TextCase::Upper => "upper",
    }
}

fn logo_preset_name(value: LogoPreset) -> &'static str {
    match value {
        LogoPreset::Macos => "macos",
    }
}

fn logo_size_name(value: LogoSize) -> &'static str {
    match value {
        LogoSize::Small => "small",
        LogoSize::Large => "large",
    }
}

fn image_crop_name(value: ImageCropMode) -> &'static str {
    match value {
        ImageCropMode::Center => "center",
        ImageCropMode::None => "none",
    }
}

fn lua_image_height(value: ImageHeight) -> String {
    match value {
        ImageHeight::Fixed(rows) => rows.to_string(),
        ImageHeight::Auto => lua_inline_string("auto"),
    }
}

fn git_view_name(value: GitView) -> &'static str {
    match value {
        GitView::Branch => "branch",
        GitView::BranchOrCommit => "branch_or_commit",
    }
}

fn host_view_name(value: HostView) -> &'static str {
    match value {
        HostView::Name => "name",
        HostView::Hostname => "hostname",
        HostView::Full => "full",
    }
}

fn os_view_name(value: OsView) -> &'static str {
    match value {
        OsView::Name => "name",
        OsView::Version => "version",
        OsView::NameVersion => "name_version",
        OsView::Full => "full",
    }
}

fn kernel_view_name(value: KernelView) -> &'static str {
    match value {
        KernelView::Name => "name",
        KernelView::Version => "version",
        KernelView::NameVersion => "name_version",
        KernelView::Full => "full",
    }
}

fn uptime_view_name(value: UptimeView) -> &'static str {
    match value {
        UptimeView::Short => "short",
        UptimeView::Full => "full",
    }
}

fn packages_view_name(value: PackagesView) -> &'static str {
    match value {
        PackagesView::Total => "total",
        PackagesView::Compact => "compact",
        PackagesView::Full => "full",
    }
}

fn shell_view_name(value: ShellView) -> &'static str {
    match value {
        ShellView::Name => "name",
        ShellView::Path => "path",
        ShellView::Full => "full",
    }
}

fn terminal_view_name(value: TerminalView) -> &'static str {
    match value {
        TerminalView::Name => "name",
        TerminalView::Command => "command",
        TerminalView::Full => "full",
    }
}

fn cpu_view_name(value: CpuView) -> &'static str {
    match value {
        CpuView::Model => "model",
        CpuView::ModelCores => "model_cores",
        CpuView::Full => "full",
    }
}

fn gpu_view_name(value: GpuView) -> &'static str {
    match value {
        GpuView::Model => "model",
        GpuView::ModelCores => "model_cores",
        GpuView::Full => "full",
    }
}

fn memory_view_name(value: MemoryView) -> &'static str {
    match value {
        MemoryView::Total => "total",
        MemoryView::UsedTotal => "used_total",
        MemoryView::Full => "full",
    }
}

fn battery_view_name(value: BatteryView) -> &'static str {
    match value {
        BatteryView::Percent => "percent",
        BatteryView::Status => "status",
        BatteryView::Full => "full",
    }
}

fn disk_view_name(value: DiskView) -> &'static str {
    match value {
        DiskView::UsedTotal => "used_total",
        DiskView::MountUsedTotal => "mount_used_total",
        DiskView::Full => "full",
    }
}

fn display_view_name(value: DisplayView) -> &'static str {
    match value {
        DisplayView::Resolution => "resolution",
        DisplayView::NameResolution => "name_resolution",
        DisplayView::Full => "full",
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::config::{AppConfig, LineKey};
    use crate::render::style::{ColorSpec, TextCase};

    #[test]
    fn renders_documented_config_with_current_values() {
        let mut config = AppConfig::default();
        config.order = vec![LineKey::Directory, LineKey::Spotify];
        config.text_style.case = TextCase::Upper;
        config.colors.label = Some(ColorSpec::Ansi(4));
        config.spotify.format = "$artist :: $track".to_string();

        let output = render(&config);

        assert!(output.contains("config.order = {"));
        assert!(output.contains("\"spotify\""));
        assert!(output.contains("case = \"upper\""));
        assert!(output.contains("config.output = {"));
        assert!(output.contains("format = \"$artist :: $track\""));
        assert!(output.contains("label = \"blue\""));
    }
}
