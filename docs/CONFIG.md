# Config Guide

Meld reads one standard config file by default:

```text
~/.config/meld/init.lua
```

You can point to a different file for one run:

```bash
meld --config /path/to/init.lua
```

## Basic Shape

The config is a normal Lua script:

```lua
local config = meld

config.layout = {
  align = true,
  separator = ":",
}

config.order = {
  "directory",
  "git",
  "spotify",
  "os",
  "kernel",
  "shell",
  "terminal",
}
```

Only fields listed in `config.order` are rendered.

## Order

Supported line keys:

```lua
config.order = {
  "directory",
  "git",
  "spotify",
  "host",
  "os",
  "kernel",
  "uptime",
  "packages",
  "shell",
  "terminal",
  "cpu",
  "gpu",
  "memory",
  "battery",
  "disk",
  "display",
}
```

Commenting a line out hides it.

## Labels

Labels control the left side of each info row:

```lua
config.labels = {
  shell = "Shell",
  terminal = "Terminal",
  spotify = "Now",
}
```

You can use ANSI escapes inside labels if you want per-symbol color:

```lua
local esc = string.char(27)
config.labels.shell = esc .. "[31mS" .. esc .. "[32mh" .. esc .. "[34mℓ" .. esc .. "[0m"
```

Avoid wrapping the same label again through `config.colors.label` when you hand-color it.

## Colors

Info line colors:

```lua
config.colors = {
  label = "blue",
  separator = "bright_black",
  value = "white",
}
```

Supported color names include:

```text
black red green yellow blue magenta cyan white
bright_black bright_red bright_green bright_yellow
bright_blue bright_magenta bright_cyan bright_white
```

ANSI indexes `0..15` are also accepted.

## Text Case

```lua
config.text = {
  case = "default",
}
```

Values:

- `default`
- `lower`
- `upper`

## Visuals

ASCII logo preset:

```lua
config.logo = {
  enabled = true,
  preset = "macos",
  size = "large",
  padding = 3,
}
```

Custom ASCII logo:

```lua
config.logo = {
  enabled = true,
  padding = 2,
  text = [[
 /\_/\\
 ( o.o )
  > ^ <
]],
}
```

Colored custom logo:

```lua
local esc = string.char(27)

config.logo.text = table.concat({
  esc .. "[34m██" .. esc .. "[0m",
  esc .. "[36m██" .. esc .. "[0m",
}, "\n")
```

## WezTerm Image

```lua
config.image = {
  enabled = true,
  path = "assets/avatar.png",
  height = 10,
  crop = "center",
  padding = 3,
}
```

Relative paths are resolved relative to the directory containing `init.lua`.

Crop values:

- `center`
- `none`

## Spotify

```lua
config.spotify = {
  format = "$artist — $track",
  cover_as_image = true,
}
```

Supported placeholders:

- `$artist`
- `$track`
- `$album`
- `$state`
- `$id`

To show the Spotify row, include `"spotify"` in `config.order`.

If `cover_as_image = true`, the current track cover is tried before the normal image/logo backends.

## Color Blocks

```lua
config.blocks = {
  enabled = true,
  width = 3,
  height = 1,
  symbol = "█",
}
```

This renders two rows of ANSI colors after the info column.

## Printing an Updated Config

To print the current effective config as a documented file:

```bash
meld --print-config
```

To rewrite your config with current values and fresh comments:

```bash
make update-config
```
