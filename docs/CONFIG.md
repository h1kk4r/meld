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

## Extra Output

Use `config.output` to print text or shell command output before/after the whole Meld block:

```lua
config.output = {
  before = {
    "hello",
    { command = "date '+%H:%M'" },
    "this text appears after the command",
  },
  after = {
    "\n",
  },
}
```

Commands run through `$SHELL -lc` during normal rendering. Prefer this over `print("hello")` in `init.lua`, because direct Lua prints happen while Meld parses the config and can leak into `--diagnostics` or `--print-config`.

`"\n"` is a valid way to add one empty line after the rendered block.

This duplicate-key form does not work in Lua, because the first `text` is overwritten before Meld reads the config:

```lua
before = {
  text = "hello",
  command = "ls",
  text = "this is ls command",
}
```

Use the ordered list form instead:

```lua
before = {
  "hello",
  { command = "ls" },
  "this is ls command",
}
```

The old simple form still works for one text and one command:

```lua
before = {
  text = "hello",
  command = "ls",
}
```

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
  height = "auto",
  crop = "center",
  padding = 3,
}
```

Relative paths are resolved relative to the directory containing `init.lua`.

`height` can be a fixed number of terminal rows or `"auto"`. With `"auto"`, WezTerm images and Spotify cover art match the rendered info block height on the right, including color blocks.

Crop values:

- `center`
- `none`

## Spotify

```lua
config.spotify = {
  client_id = "your Spotify app client ID",
  redirect_uri = "http://127.0.0.1:8888/callback",
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

To use the Spotify Web API instead of the local macOS fallback, set `client_id`
to your Spotify app client ID (`api_key` is also accepted as an alias), add the
same `redirect_uri` to the app settings in the Spotify dashboard, then run:

```bash
meld --spotify-login
```

The login stores `spotify-token.json` next to `init.lua`; Meld reads this file
automatically on later runs. If the client ID, token or Web API request is
unavailable, Meld silently falls back to the local `osascript` lookup on macOS.
Runtime Web API checks use a short timeout so Spotify network issues do not
delay the rest of the output for long.

If `cover_as_image = true`, the current track cover is tried before the normal image/logo backends.

Spotify cover art uses `config.image.height`, so `height = "auto"` makes the cover follow the height of the text block instead of a fixed row count.

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
