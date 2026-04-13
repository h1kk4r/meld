use mlua::{Lua, Table, Value};

use crate::error::{AppError, AppResult};
use crate::modules::git::GitView;
use crate::modules::spotify::SpotifyConfig;
use crate::modules::system::{
    BatteryView, CpuView, DEFAULT_SYSTEM_FIELD_ORDER, DiskView, DisplayView, GpuView, HostView,
    KernelView, MemoryView, OsView, PackagesView, ShellView, SystemField, TerminalView, UptimeView,
};
use crate::render::blocks::ColorBlocksConfig;
use crate::render::image::{ImageConfig, ImageCropMode, ImageHeight};
use crate::render::layout::LayoutConfig;
use crate::render::logo::{LogoConfig, LogoPreset, LogoSize};
use crate::render::style::{ColorSpec, TextCase, TextColors, TextStyleConfig};

use super::schema::{
    AppConfig, LineKey, LineLabels, OutputConfig, OutputHookConfig, OutputItemConfig,
};

pub(crate) fn parse_source(source: &str, chunk_name: &str) -> AppResult<AppConfig> {
    let lua = Lua::new();
    let table = load_config_table(&lua, source, chunk_name)?;
    parse_table(table)
}

fn load_config_table(lua: &Lua, source: &str, chunk_name: &str) -> AppResult<Table> {
    let globals = lua.globals();
    let config_table = lua.create_table()?;
    globals.set("meld", config_table.clone())?;
    globals.set("config", config_table.clone())?;

    let value = lua.load(source).set_name(chunk_name).eval::<Value>()?;

    match value {
        Value::Table(table) => Ok(table),
        Value::Nil => Ok(globals.get("config")?),
        _ => Err(AppError::Config(
            "init.lua must return a table or assign fields to the global `config`/`meld` table"
                .to_string(),
        )),
    }
}

fn parse_table(table: Table) -> AppResult<AppConfig> {
    let mut config = AppConfig::default();

    if let Some(order) = table.get::<Option<Table>>("order")? {
        config.order = parse_order(order)?;
    }

    if let Some(layout) = table.get::<Option<Table>>("layout")? {
        parse_layout(&mut config.layout, layout)?;
    }

    if let Some(text) = table.get::<Option<Table>>("text")? {
        parse_text_style(&mut config.text_style, text)?;
    }

    if let Some(output) = table.get::<Option<Value>>("output")? {
        parse_output(&mut config.output, output)?;
    }

    if let Some(labels) = table.get::<Option<Table>>("labels")? {
        parse_labels(&mut config.labels, labels)?;
    }

    if let Some(colors) = table.get::<Option<Table>>("colors")? {
        parse_colors(&mut config.colors, colors)?;
    }

    if let Some(logo) = table.get::<Option<Value>>("logo")? {
        parse_logo(&mut config.logo, logo)?;
    }

    if let Some(image) = table.get::<Option<Value>>("image")? {
        parse_image(&mut config.image, image)?;
    }

    if let Some(blocks) = table.get::<Option<Value>>("blocks")? {
        parse_blocks(&mut config.blocks, blocks)?;
    }

    if let Some(value) = table.get::<Option<Value>>("git")? {
        config.git_view = parse_git_view_value(value)?;
    }

    if let Some(value) = table.get::<Option<Value>>("spotify")? {
        parse_spotify(&mut config.spotify, value)?;
    }

    for field in DEFAULT_SYSTEM_FIELD_ORDER {
        if let Some(value) = table.get::<Option<Value>>(field.key())? {
            apply_system_value(&mut config, field, value)?;
        }
    }

    Ok(config)
}

fn parse_order(order: Table) -> AppResult<Vec<LineKey>> {
    let mut resolved = Vec::new();

    for value in order.sequence_values::<String>() {
        let key = parse_line_key(&value?)?;

        if !resolved.contains(&key) {
            resolved.push(key);
        }
    }

    Ok(resolved)
}

fn parse_line_key(value: &str) -> AppResult<LineKey> {
    match value {
        "directory" => Ok(LineKey::Directory),
        "git" => Ok(LineKey::Git),
        "spotify" => Ok(LineKey::Spotify),
        "host" => Ok(LineKey::System(SystemField::Host)),
        "os" => Ok(LineKey::System(SystemField::Os)),
        "kernel" => Ok(LineKey::System(SystemField::Kernel)),
        "uptime" => Ok(LineKey::System(SystemField::Uptime)),
        "packages" => Ok(LineKey::System(SystemField::Packages)),
        "shell" => Ok(LineKey::System(SystemField::Shell)),
        "terminal" => Ok(LineKey::System(SystemField::Terminal)),
        "cpu" => Ok(LineKey::System(SystemField::Cpu)),
        "gpu" => Ok(LineKey::System(SystemField::Gpu)),
        "memory" => Ok(LineKey::System(SystemField::Memory)),
        "battery" => Ok(LineKey::System(SystemField::Battery)),
        "disk" => Ok(LineKey::System(SystemField::Disk)),
        "display" => Ok(LineKey::System(SystemField::Display)),
        _ => Err(AppError::Config(format!(
            "unknown line in order: `{}`",
            value
        ))),
    }
}

fn parse_layout(layout_config: &mut LayoutConfig, table: Table) -> AppResult<()> {
    if let Some(value) = table.get::<Option<bool>>("align")? {
        layout_config.align = value;
    }

    if let Some(value) = table.get::<Option<String>>("separator")? {
        layout_config.separator = value;
    }

    Ok(())
}

fn parse_text_style(text_style: &mut TextStyleConfig, table: Table) -> AppResult<()> {
    for pair in table.pairs::<String, Value>() {
        let (key, value) = pair?;

        match key.as_str() {
            "case" => {
                let value = match value {
                    Value::String(text) => text.to_str()?.to_string(),
                    _ => {
                        return Err(AppError::Config(
                            "`text.case` must be a string in init.lua".to_string(),
                        ));
                    }
                };

                text_style.case = TextCase::from_name(&value).ok_or_else(|| {
                    AppError::Config(format!("invalid text case `{}` in init.lua", value))
                })?;
            }
            _ => {
                return Err(AppError::Config(format!(
                    "unknown key `{}` in `text` table",
                    key
                )));
            }
        }
    }

    Ok(())
}

fn parse_output(output: &mut OutputConfig, value: Value) -> AppResult<()> {
    match value {
        Value::Nil | Value::Boolean(false) => {
            *output = OutputConfig::default();
            Ok(())
        }
        Value::Table(table) => {
            for pair in table.pairs::<String, Value>() {
                let (key, value) = pair?;

                match key.as_str() {
                    "before" => parse_output_hook(&mut output.before, "output.before", value)?,
                    "after" => parse_output_hook(&mut output.after, "output.after", value)?,
                    _ => {
                        return Err(AppError::Config(format!(
                            "unknown key `{}` in `output` table",
                            key
                        )));
                    }
                }
            }

            Ok(())
        }
        _ => Err(AppError::Config(
            "`output` must be a table or false in init.lua".to_string(),
        )),
    }
}

fn parse_output_hook(hook: &mut OutputHookConfig, key: &str, value: Value) -> AppResult<()> {
    match value {
        Value::Nil | Value::Boolean(false) => {
            *hook = OutputHookConfig::default();
            Ok(())
        }
        Value::String(text) => {
            hook.items = parse_text_value(text.to_str()?.as_ref())
                .map(OutputItemConfig::Text)
                .into_iter()
                .collect();
            Ok(())
        }
        Value::Table(table) => {
            hook.items = parse_output_hook_table(key, table)?;
            Ok(())
        }
        _ => Err(AppError::Config(format!(
            "`{}` must be a string, table, or false in init.lua",
            key
        ))),
    }
}

fn parse_output_hook_table(key: &str, table: Table) -> AppResult<Vec<OutputItemConfig>> {
    let sequence_len = table.raw_len();

    if sequence_len > 0 {
        let mut items = Vec::new();

        for index in 1..=sequence_len {
            let value = table.get::<Value>(index)?;
            items.extend(parse_output_item(&format!("{key}[{index}]"), value)?);
        }

        for pair in table.pairs::<Value, Value>() {
            let (field, _) = pair?;

            if let Value::String(field) = field {
                return Err(AppError::Config(format!(
                    "`{}` mixes ordered items with keyed field `{}`; use only the ordered list form",
                    key,
                    field.to_str()?
                )));
            }
        }

        return Ok(items);
    }

    let mut text = None;
    let mut command = None;

    for pair in table.pairs::<String, Value>() {
        let (field, value) = pair?;

        match field.as_str() {
            "text" => text = parse_optional_text(&format!("{key}.text"), value)?,
            "command" => command = parse_optional_command(&format!("{key}.command"), value)?,
            _ => {
                return Err(AppError::Config(format!(
                    "unknown key `{}` in `{}` table",
                    field, key
                )));
            }
        }
    }

    let mut items = Vec::new();

    if let Some(text) = text {
        items.push(OutputItemConfig::Text(text));
    }

    if let Some(command) = command {
        items.push(OutputItemConfig::Command(command));
    }

    Ok(items)
}

fn parse_output_item(key: &str, value: Value) -> AppResult<Vec<OutputItemConfig>> {
    match value {
        Value::Nil | Value::Boolean(false) => Ok(Vec::new()),
        Value::String(text) => Ok(parse_text_value(text.to_str()?.as_ref())
            .map(OutputItemConfig::Text)
            .into_iter()
            .collect()),
        Value::Table(table) => {
            let mut text = None;
            let mut command = None;

            for pair in table.pairs::<String, Value>() {
                let (field, value) = pair?;

                match field.as_str() {
                    "text" => text = parse_optional_text(&format!("{key}.text"), value)?,
                    "command" => {
                        command = parse_optional_command(&format!("{key}.command"), value)?
                    }
                    _ => {
                        return Err(AppError::Config(format!(
                            "unknown key `{}` in `{}` item",
                            field, key
                        )));
                    }
                }
            }

            let mut items = Vec::new();

            if let Some(text) = text {
                items.push(OutputItemConfig::Text(text));
            }

            if let Some(command) = command {
                items.push(OutputItemConfig::Command(command));
            }

            Ok(items)
        }
        _ => Err(AppError::Config(format!(
            "`{}` must be a string, command table, text table, or false in init.lua",
            key
        ))),
    }
}

fn parse_optional_text(key: &str, value: Value) -> AppResult<Option<String>> {
    match value {
        Value::Nil | Value::Boolean(false) => Ok(None),
        Value::String(text) => Ok(parse_text_value(text.to_str()?.as_ref())),
        _ => Err(AppError::Config(format!(
            "`{}` must be a string or false in init.lua",
            key
        ))),
    }
}

fn parse_optional_command(key: &str, value: Value) -> AppResult<Option<String>> {
    match value {
        Value::Nil | Value::Boolean(false) => Ok(None),
        Value::String(text) => {
            let command = text.to_str()?.trim().to_string();
            Ok((!command.is_empty()).then_some(command))
        }
        _ => Err(AppError::Config(format!(
            "`{}` must be a string or false in init.lua",
            key
        ))),
    }
}

fn parse_text_value(value: &str) -> Option<String> {
    (!value.is_empty()).then_some(value.to_string())
}

fn parse_labels(labels: &mut LineLabels, table: Table) -> AppResult<()> {
    for pair in table.pairs::<String, String>() {
        let (key, value) = pair?;
        let line = parse_line_key(&key)?;
        labels.set(line, value);
    }

    Ok(())
}

fn parse_colors(colors: &mut TextColors, table: Table) -> AppResult<()> {
    for pair in table.pairs::<String, Value>() {
        let (key, value) = pair?;

        match key.as_str() {
            "label" => colors.label = parse_color_value("colors.label", value)?,
            "separator" => colors.separator = parse_color_value("colors.separator", value)?,
            "value" => colors.value = parse_color_value("colors.value", value)?,
            _ => {
                return Err(AppError::Config(format!(
                    "unknown key `{}` in `colors` table",
                    key
                )));
            }
        }
    }

    Ok(())
}

fn parse_logo(logo: &mut LogoConfig, value: Value) -> AppResult<()> {
    match value {
        Value::Boolean(enabled) => {
            logo.enabled = enabled;
            Ok(())
        }
        Value::Table(table) => {
            logo.enabled = table.get::<Option<bool>>("enabled")?.unwrap_or(true);

            if let Some(value) = table.get::<Option<String>>("preset")? {
                logo.preset = parse_logo_preset(&value)?;
            }

            if let Some(value) = table.get::<Option<String>>("size")? {
                logo.size = parse_logo_size(&value)?;
            }

            if let Some(value) = table.get::<Option<String>>("text")? {
                let text = value.trim_end_matches('\n').to_string();
                logo.text = (!text.is_empty()).then_some(text);
            }

            if let Some(value) = table.get::<Option<u32>>("padding")? {
                logo.padding = value as usize;
            }

            Ok(())
        }
        _ => Err(AppError::Config(
            "`logo` must be a boolean or table in init.lua".to_string(),
        )),
    }
}

fn parse_image(image: &mut ImageConfig, value: Value) -> AppResult<()> {
    match value {
        Value::Boolean(enabled) => {
            image.enabled = enabled;
            Ok(())
        }
        Value::Table(table) => {
            image.enabled = table.get::<Option<bool>>("enabled")?.unwrap_or(true);

            if let Some(value) = table.get::<Option<String>>("path")? {
                image.path = Some(value.into());
            }

            if let Some(value) = table.get::<Option<Value>>("height")? {
                image.height = parse_image_height(value)?;
            }

            if let Some(value) = table.get::<Option<String>>("crop")? {
                image.crop = parse_image_crop(&value)?;
            }

            if let Some(value) = table.get::<Option<u32>>("padding")? {
                image.padding = value as usize;
            }

            Ok(())
        }
        _ => Err(AppError::Config(
            "`image` must be a boolean or table in init.lua".to_string(),
        )),
    }
}

fn parse_blocks(blocks: &mut ColorBlocksConfig, value: Value) -> AppResult<()> {
    match value {
        Value::Boolean(enabled) => {
            blocks.enabled = enabled;
            Ok(())
        }
        Value::Table(table) => {
            blocks.enabled = table.get::<Option<bool>>("enabled")?.unwrap_or(true);

            if let Some(value) = table.get::<Option<u32>>("width")? {
                blocks.width = value as usize;
            }

            if let Some(value) = table.get::<Option<u32>>("height")? {
                blocks.height = value as usize;
            }

            if let Some(value) = table.get::<Option<String>>("symbol")? {
                blocks.symbol = value;
            }

            Ok(())
        }
        _ => Err(AppError::Config(
            "`blocks` must be a boolean or table in init.lua".to_string(),
        )),
    }
}

fn parse_spotify(spotify: &mut SpotifyConfig, value: Value) -> AppResult<()> {
    match value {
        Value::Table(table) => {
            if let Some(value) = table.get::<Option<String>>("api_key")? {
                spotify.client_id = empty_string_to_none(value);
            }

            if let Some(value) = table.get::<Option<String>>("client_id")? {
                spotify.client_id = empty_string_to_none(value);
            }

            if let Some(value) = table.get::<Option<String>>("redirect_uri")? {
                if let Some(value) = empty_string_to_none(value) {
                    spotify.redirect_uri = value;
                }
            }

            if let Some(value) = table.get::<Option<String>>("format")? {
                spotify.format = value;
            }

            if let Some(value) = table.get::<Option<bool>>("cover_as_image")? {
                spotify.cover_as_image = value;
            }

            Ok(())
        }
        _ => Err(AppError::Config(
            "`spotify` must be a table in init.lua".to_string(),
        )),
    }
}

fn empty_string_to_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn parse_color_value(key: &str, value: Value) -> AppResult<Option<ColorSpec>> {
    match value {
        Value::Nil | Value::Boolean(false) => Ok(None),
        Value::Integer(number) => Ok(Some(parse_color_index(key, number)?)),
        Value::Number(number) if number.fract() == 0.0 => {
            Ok(Some(parse_color_index(key, number as i64)?))
        }
        Value::String(text) => Ok(Some(parse_color_name(key, text.to_str()?.as_ref())?)),
        _ => Err(AppError::Config(format!(
            "`{}` must be a color name, ANSI index, or false in init.lua",
            key
        ))),
    }
}

fn parse_color_index(key: &str, value: i64) -> AppResult<ColorSpec> {
    let index = u8::try_from(value)
        .map_err(|_| AppError::Config(format!("`{}` must be between 0 and 15 in init.lua", key)))?;

    ColorSpec::from_index(index)
        .ok_or_else(|| AppError::Config(format!("`{}` must be between 0 and 15 in init.lua", key)))
}

fn parse_color_name(key: &str, value: &str) -> AppResult<ColorSpec> {
    ColorSpec::from_name(value).ok_or_else(|| {
        AppError::Config(format!(
            "invalid color `{}` for `{}` in init.lua",
            value, key
        ))
    })
}

fn parse_logo_preset(value: &str) -> AppResult<LogoPreset> {
    LogoPreset::from_name(value)
        .ok_or_else(|| AppError::Config(format!("invalid logo preset `{}` in init.lua", value)))
}

fn parse_logo_size(value: &str) -> AppResult<LogoSize> {
    LogoSize::from_name(value)
        .ok_or_else(|| AppError::Config(format!("invalid logo size `{}` in init.lua", value)))
}

fn parse_image_crop(value: &str) -> AppResult<ImageCropMode> {
    ImageCropMode::from_name(value)
        .ok_or_else(|| AppError::Config(format!("invalid image crop mode `{}` in init.lua", value)))
}

fn parse_image_height(value: Value) -> AppResult<ImageHeight> {
    match value {
        Value::Integer(number) => {
            let rows = usize::try_from(number).map_err(|_| {
                AppError::Config("`image.height` must be positive in init.lua".to_string())
            })?;
            Ok(ImageHeight::Fixed(rows.max(1)))
        }
        Value::Number(number) if number.fract() == 0.0 && number > 0.0 => {
            Ok(ImageHeight::Fixed(number as usize))
        }
        Value::String(text) => {
            let value = text.to_str()?;
            ImageHeight::from_name(value.as_ref()).ok_or_else(|| {
                AppError::Config(format!(
                    "invalid image height `{}` in init.lua; use a number or \"auto\"",
                    value
                ))
            })
        }
        _ => Err(AppError::Config(
            "`image.height` must be a number or \"auto\" in init.lua".to_string(),
        )),
    }
}

fn parse_git_view_value(value: Value) -> AppResult<GitView> {
    match value {
        Value::String(text) => parse_git_view(text.to_str()?.as_ref()),
        _ => Err(AppError::Config(
            "`git` must be a string in init.lua; hide it by removing it from `order`".to_string(),
        )),
    }
}

fn apply_system_value(config: &mut AppConfig, field: SystemField, value: Value) -> AppResult<()> {
    match field {
        SystemField::Host => {
            apply_view_value(value, field, &mut config.system_views.host, parse_host_view)
        }
        SystemField::Os => {
            apply_view_value(value, field, &mut config.system_views.os, parse_os_view)
        }
        SystemField::Kernel => apply_view_value(
            value,
            field,
            &mut config.system_views.kernel,
            parse_kernel_view,
        ),
        SystemField::Uptime => apply_view_value(
            value,
            field,
            &mut config.system_views.uptime,
            parse_uptime_view,
        ),
        SystemField::Packages => apply_view_value(
            value,
            field,
            &mut config.system_views.packages,
            parse_packages_view,
        ),
        SystemField::Shell => apply_view_value(
            value,
            field,
            &mut config.system_views.shell,
            parse_shell_view,
        ),
        SystemField::Terminal => apply_view_value(
            value,
            field,
            &mut config.system_views.terminal,
            parse_terminal_view,
        ),
        SystemField::Cpu => {
            apply_view_value(value, field, &mut config.system_views.cpu, parse_cpu_view)
        }
        SystemField::Gpu => {
            apply_view_value(value, field, &mut config.system_views.gpu, parse_gpu_view)
        }
        SystemField::Memory => apply_view_value(
            value,
            field,
            &mut config.system_views.memory,
            parse_memory_view,
        ),
        SystemField::Battery => apply_view_value(
            value,
            field,
            &mut config.system_views.battery,
            parse_battery_view,
        ),
        SystemField::Disk => {
            apply_view_value(value, field, &mut config.system_views.disk, parse_disk_view)
        }
        SystemField::Display => apply_view_value(
            value,
            field,
            &mut config.system_views.display,
            parse_display_view,
        ),
    }
}

fn apply_view_value<T: Copy>(
    value: Value,
    field: SystemField,
    view: &mut T,
    parse_view: fn(&str) -> AppResult<T>,
) -> AppResult<()> {
    match value {
        Value::String(text) => {
            *view = parse_view(text.to_str()?.as_ref())?;
        }
        _ => {
            return Err(AppError::Config(format!(
                "`{}` must be a string in init.lua; hide it by removing it from `order`",
                field.key()
            )));
        }
    }

    Ok(())
}

fn parse_git_view(value: &str) -> AppResult<GitView> {
    match value {
        "branch" => Ok(GitView::Branch),
        "branch_or_commit" => Ok(GitView::BranchOrCommit),
        _ => Err(invalid_view("git", value)),
    }
}

fn parse_host_view(value: &str) -> AppResult<HostView> {
    match value {
        "name" => Ok(HostView::Name),
        "hostname" => Ok(HostView::Hostname),
        "full" => Ok(HostView::Full),
        _ => Err(invalid_view("host", value)),
    }
}

fn parse_os_view(value: &str) -> AppResult<OsView> {
    match value {
        "name" => Ok(OsView::Name),
        "version" => Ok(OsView::Version),
        "name_version" => Ok(OsView::NameVersion),
        "full" => Ok(OsView::Full),
        _ => Err(invalid_view("os", value)),
    }
}

fn parse_kernel_view(value: &str) -> AppResult<KernelView> {
    match value {
        "name" => Ok(KernelView::Name),
        "version" => Ok(KernelView::Version),
        "name_version" => Ok(KernelView::NameVersion),
        "full" => Ok(KernelView::Full),
        _ => Err(invalid_view("kernel", value)),
    }
}

fn parse_uptime_view(value: &str) -> AppResult<UptimeView> {
    match value {
        "short" => Ok(UptimeView::Short),
        "full" => Ok(UptimeView::Full),
        _ => Err(invalid_view("uptime", value)),
    }
}

fn parse_packages_view(value: &str) -> AppResult<PackagesView> {
    match value {
        "total" => Ok(PackagesView::Total),
        "compact" => Ok(PackagesView::Compact),
        "full" => Ok(PackagesView::Full),
        _ => Err(invalid_view("packages", value)),
    }
}

fn parse_shell_view(value: &str) -> AppResult<ShellView> {
    match value {
        "name" => Ok(ShellView::Name),
        "path" => Ok(ShellView::Path),
        "full" => Ok(ShellView::Full),
        _ => Err(invalid_view("shell", value)),
    }
}

fn parse_terminal_view(value: &str) -> AppResult<TerminalView> {
    match value {
        "name" => Ok(TerminalView::Name),
        "command" => Ok(TerminalView::Command),
        "full" => Ok(TerminalView::Full),
        _ => Err(invalid_view("terminal", value)),
    }
}

fn parse_cpu_view(value: &str) -> AppResult<CpuView> {
    match value {
        "model" => Ok(CpuView::Model),
        "model_cores" => Ok(CpuView::ModelCores),
        "full" => Ok(CpuView::Full),
        _ => Err(invalid_view("cpu", value)),
    }
}

fn parse_gpu_view(value: &str) -> AppResult<GpuView> {
    match value {
        "model" => Ok(GpuView::Model),
        "model_cores" => Ok(GpuView::ModelCores),
        "full" => Ok(GpuView::Full),
        _ => Err(invalid_view("gpu", value)),
    }
}

fn parse_memory_view(value: &str) -> AppResult<MemoryView> {
    match value {
        "total" => Ok(MemoryView::Total),
        "used_total" => Ok(MemoryView::UsedTotal),
        "full" => Ok(MemoryView::Full),
        _ => Err(invalid_view("memory", value)),
    }
}

fn parse_battery_view(value: &str) -> AppResult<BatteryView> {
    match value {
        "percent" => Ok(BatteryView::Percent),
        "status" => Ok(BatteryView::Status),
        "full" => Ok(BatteryView::Full),
        _ => Err(invalid_view("battery", value)),
    }
}

fn parse_disk_view(value: &str) -> AppResult<DiskView> {
    match value {
        "used_total" => Ok(DiskView::UsedTotal),
        "mount_used_total" => Ok(DiskView::MountUsedTotal),
        "full" => Ok(DiskView::Full),
        _ => Err(invalid_view("disk", value)),
    }
}

fn parse_display_view(value: &str) -> AppResult<DisplayView> {
    match value {
        "resolution" => Ok(DisplayView::Resolution),
        "name_resolution" => Ok(DisplayView::NameResolution),
        "full" => Ok(DisplayView::Full),
        _ => Err(invalid_view("display", value)),
    }
}

fn invalid_view(key: &str, value: &str) -> AppError {
    AppError::Config(format!(
        "invalid view `{}` for `{}` in init.lua",
        value, key
    ))
}

#[cfg(test)]
mod tests {
    use super::{parse_line_key, parse_order, parse_source};
    use crate::config::{LineKey, OutputItemConfig};
    use crate::modules::git::GitView;
    use crate::modules::spotify::SpotifyConfig;
    use crate::modules::system::{MemoryView, SystemField};
    use crate::render::image::{ImageCropMode, ImageHeight};
    use crate::render::logo::LogoSize;
    use crate::render::style::{ColorSpec, TextCase};
    use mlua::Lua;

    #[test]
    fn parses_line_keys() {
        assert_eq!(parse_line_key("directory").unwrap(), LineKey::Directory);
        assert_eq!(
            parse_line_key("battery").unwrap(),
            LineKey::System(SystemField::Battery)
        );
    }

    #[test]
    fn parses_order_exactly_as_listed() {
        let lua = Lua::new();
        let table = lua
            .load(r#"return { "battery", "git", "directory" }"#)
            .eval()
            .unwrap();

        let order = parse_order(table).unwrap();

        assert_eq!(order[0], LineKey::System(SystemField::Battery));
        assert_eq!(order[1], LineKey::Git);
        assert_eq!(order[2], LineKey::Directory);
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn parses_returned_table_config() {
        let config = parse_source(
            r#"
return {
  git = "branch",
  spotify = {
    client_id = "test-client",
    format = "$artist / $track",
    cover_as_image = true,
  },
  terminal = "full",
  text = {
    case = "lower",
  },
  order = { "directory", "git", "spotify", "terminal" },
}
"#,
            "init.lua",
        )
        .unwrap();

        assert!(matches!(config.git_view, GitView::Branch));
        assert_eq!(config.order[0], LineKey::Directory);
        assert_eq!(config.order[1], LineKey::Git);
        assert_eq!(config.order[2], LineKey::Spotify);
        assert_eq!(
            config.spotify,
            SpotifyConfig {
                format: "$artist / $track".to_string(),
                cover_as_image: true,
                client_id: Some("test-client".to_string()),
                ..SpotifyConfig::default()
            }
        );
        assert_eq!(config.text_style.case, TextCase::Lower);
    }

    #[test]
    fn parses_script_style_global_config() {
        let config = parse_source(
            r#"
local config = meld

config.git = "branch"
config.memory = "used_total"
"#,
            "init.lua",
        )
        .unwrap();

        assert!(matches!(config.git_view, GitView::Branch));
        assert!(matches!(config.system_views.memory, MemoryView::UsedTotal));
    }

    #[test]
    fn parses_spotify_api_key_alias() {
        let config = parse_source(
            r#"
return {
  spotify = {
    api_key = "alias-client",
    redirect_uri = "",
  },
}
"#,
            "init.lua",
        )
        .unwrap();

        assert_eq!(config.spotify.client_id.as_deref(), Some("alias-client"));
        assert_eq!(
            config.spotify.redirect_uri,
            SpotifyConfig::default().redirect_uri
        );
    }

    #[test]
    fn parses_output_hooks() {
        let config = parse_source(
            r#"
return {
  output = {
    before = "hello",
    after = {
      text = "bye",
      command = "printf done",
    },
  },
}
"#,
            "init.lua",
        )
        .unwrap();

        assert_eq!(
            config.output.before.items,
            vec![OutputItemConfig::Text("hello".to_string())]
        );
        assert_eq!(
            config.output.after.items,
            vec![
                OutputItemConfig::Text("bye".to_string()),
                OutputItemConfig::Command("printf done".to_string()),
            ]
        );
    }

    #[test]
    fn parses_ordered_output_hook_items() {
        let config = parse_source(
            r#"
return {
  output = {
    before = {
      "hello",
      { command = "printf command" },
      { text = "this is ls command" },
    },
  },
}
"#,
            "init.lua",
        )
        .unwrap();

        assert_eq!(
            config.output.before.items,
            vec![
                OutputItemConfig::Text("hello".to_string()),
                OutputItemConfig::Command("printf command".to_string()),
                OutputItemConfig::Text("this is ls command".to_string()),
            ]
        );
    }

    #[test]
    fn preserves_output_hook_newline_text() {
        let config = parse_source(
            r#"
return {
  output = {
    after = {
      text = "\n",
    },
  },
}
"#,
            "init.lua",
        )
        .unwrap();

        assert_eq!(
            config.output.after.items,
            vec![OutputItemConfig::Text("\n".to_string())]
        );
    }

    #[test]
    fn parses_logo_color_and_block_tables() {
        let config = parse_source(
            r#"
return {
  colors = {
    label = "blue",
    separator = 8,
    value = false,
  },
  logo = {
    size = "small",
    padding = 4,
  },
  image = {
    enabled = true,
    path = "logo.png",
    height = 12,
    crop = "none",
    padding = 5,
  },
  blocks = {
    enabled = true,
    width = 2,
    height = 2,
    symbol = "[]",
  },
}
"#,
            "init.lua",
        )
        .unwrap();

        assert_eq!(config.colors.label, Some(ColorSpec::Ansi(4)));
        assert_eq!(config.colors.separator, Some(ColorSpec::Ansi(8)));
        assert_eq!(config.colors.value, None);
        assert!(config.logo.enabled);
        assert_eq!(config.logo.size, LogoSize::Small);
        assert_eq!(config.logo.padding, 4);
        assert!(config.image.enabled);
        assert_eq!(
            config.image.path.as_deref(),
            Some(std::path::Path::new("logo.png"))
        );
        assert_eq!(config.image.height, ImageHeight::Fixed(12));
        assert_eq!(config.image.crop, ImageCropMode::None);
        assert_eq!(config.image.padding, 5);
        assert!(config.blocks.enabled);
        assert_eq!(config.blocks.width, 2);
        assert_eq!(config.blocks.height, 2);
        assert_eq!(config.blocks.symbol, "[]");
    }

    #[test]
    fn parses_auto_image_height() {
        let config = parse_source(
            r#"
return {
  image = {
    enabled = true,
    height = "auto",
  },
}
"#,
            "init.lua",
        )
        .unwrap();

        assert_eq!(config.image.height, ImageHeight::Auto);
    }
}
