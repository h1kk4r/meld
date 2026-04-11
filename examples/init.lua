-- Meld init.lua
--
-- This file is a real Lua script. It can either:
--   1. assign fields on the global `meld` / `config` table, or
--   2. return a table at the end.
--
-- Useful CLI flags:
--   meld --show-config-path
--   meld --diagnostics
--   meld --config /path/to/init.lua
--
-- Standard config path:
--   ~/.config/meld/init.lua
--
-- Why this path on macOS too?
-- Native macOS GUI apps usually prefer:
--   ~/Library/Application Support/<AppName>/
-- But terminal tools often live in one of these places instead:
--   ~/.config/<app>/
--   ~/.<app>rc
--   ~/.<app>/
-- For a cross-platform terminal program, ~/.config/meld/init.lua is a very normal choice.

local config = meld

-- Layout
-- Controls how labels and values are glued together.
--
-- align:
--   true  -> labels are padded to the longest label
--   false -> labels keep their natural width
--
-- separator:
--   Literal separator symbol. Spaces are added automatically around it.
--   ":" -> Shell : fish
--   "|" -> Shell | fish
--   ""  -> Shell fish
config.layout = {
  align = true,
  separator = ":",
}

-- Text transform
-- Force the rendered info text into one case.
--
-- Values:
--   "default" -> keep the original casing
--   "lower"   -> everything becomes lowercase
--   "upper"   -> everything becomes uppercase
config.text = {
  case = "default",
}

-- Extra output hooks
-- Add plain text or shell command output before/after the whole Meld block.
--
-- This is the safe place for "hello before fetch" style output.
-- A raw `print("hello")` in this init.lua may run while Meld is only parsing
-- the config, so it can leak into diagnostics or config regeneration.
--
-- Commands run through:
--   $SHELL -lc "<command>"
--
-- Use an ordered list when you need multiple items:
--   "plain text"
--   { command = "shell command" }
--
-- Do not repeat `text = ...` keys in the same table: Lua keeps only the last
-- repeated key before Meld can read it.
config.output = {
  before = {
    -- "hello",
    -- { command = "date '+%H:%M'" },
    -- "this text appears after the command",
  },
  after = {
    -- "\n", -- add one empty line after the whole Meld block
    -- "bye",
    -- { command = "printf 'done'" },
  },
}

-- Text colors
-- Color each part of the rendered line separately.
--
-- Supported values:
--   color names: "red", "blue", "bright_black", "bright_white"
--   ansi indexes: 0..15
-- Omit a key to leave that part uncolored.
--
-- Notes:
-- - `label` colors the label before the separator.
-- - `separator` colors only the separator string.
-- - `value` colors the text after the separator.
-- - By default the separator and value stay uncolored.
-- - This is the easiest way to color labels like "Shell", "CPU", "Memory".
config.colors = {
  label = "blue",
  separator = "bright_black",
}

-- Logo / custom visual
-- Left-side visual, similar to the distro logo in neofetch.
--
-- Supported presets:
--   "macos"
--
-- Supported sizes:
--   "small", "large"
--
-- `padding` controls the gap between the logo and the info column.
config.logo = {
  enabled = true,
  preset = "macos",
  size = "large",
  padding = 3,
}

-- WezTerm image
-- If enabled and running inside WezTerm, this replaces the ASCII logo block.
-- Backend priority is:
--   spotify cover -> image -> logo text -> logo preset -> none
--
-- path:
--   absolute path, or a path relative to this init.lua file
--
-- height:
--   number -> fixed height in terminal rows
--   "auto" -> match the rendered info block height
--
-- crop:
--   "center" -> center-crop to a square-like fetch slot
--   "none"   -> keep the full image without cropping
--
-- padding:
--   gap between the image area and the info column
--
-- Example:
-- config.image = {
--   enabled = true,
--   path = "assets/avatar.png",
--   height = "auto",
--   crop = "center",
--   padding = 3,
-- }
--
-- Custom text example:
-- config.logo = {
--   enabled = true,
--   padding = 2,
--   text = [[
-- /\\_/\\\\
-- ( o.o )
--  > ^ <
-- ]],
-- }
--
-- If you want to color individual symbols in custom ASCII art,
-- use normal Lua strings with ANSI escapes instead of a [[...]] block:
--
--   local esc = string.char(27)
--   config.logo.text = table.concat({
--     esc .. "[34m██" .. esc .. "[0m",
--     esc .. "[36m██" .. esc .. "[0m",
--   }, "\n")

-- Spotify sync
-- Adds a music line to `order` and can also replace the image slot
-- with the current track cover when running inside WezTerm.
--
-- Supported placeholders in `format`:
--   $artist
--   $track
--   $album
--   $state
--   $id
--
-- Example:
-- config.spotify = {
--   format = "$artist — $track",
--   cover_as_image = true,
-- }

-- ANSI color blocks
-- Prints 2 rows of the standard 0..15 ANSI colors after the info column.
--
-- width:
--   how many times to repeat `symbol` per color
--
-- height:
--   how many identical rows to print per palette row
--
-- symbol:
--   the character(s) that make up each block
--   "█" gives a classic filled block look
config.blocks = {
  enabled = true,
  width = 3,
  height = 1,
  symbol = "█",
}

-- Labels
-- Custom labels for each line.
--
-- Example:
--   shell = "Shell"
-- With the default layout and shell value placeholder `$shell`,
-- the resulting line shape is:
--   Shell : $shell
config.labels = {
  directory = "Directory",
  git = "Git",
  spotify = "Spotify",
  host = "Host",
  os = "System",
  kernel = "Kernel",
  uptime = "Uptime",
  packages = "Packages",
  shell = "Shell",
  terminal = "Terminal",
  cpu = "CPU",
  gpu = "GPU",
  memory = "Memory",
  battery = "Battery",
  disk = "Disk",
  display = "Resolution",
}

-- Order
-- Choose which lines appear first.
--
-- Notes:
-- - Only the lines listed here are shown.
-- - Comment a line out here to hide it.
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

-- Git
-- Render the current git context.
--
-- Default: "branch_or_commit"
-- Values:  "branch", "branch_or_commit"
config.git = "branch_or_commit"

-- Spotify
-- Default format: "$artist - $track"
-- cover_as_image:
--   true  -> use the current cover art instead of the regular image
--   false -> keep the regular image backend
--
-- Tip:
--   Set config.image.height = "auto" to make the cover match the info block.
config.spotify = {
  format = "$artist — $track",
  cover_as_image = false,
}

-- Host
-- Default: "full"
-- Values:  "name", "hostname", "full"
config.host = "full"

-- OS
-- Default: "name_version"
-- Values:  "name", "version", "name_version", "full"
config.os = "name_version"

-- Kernel
-- Default: "full"
-- Values:  "name", "version", "name_version", "full"
config.kernel = "full"

-- Uptime
-- Default: "short"
-- Values:  "short", "full"
config.uptime = "short"

-- Packages
-- Default: "compact"
-- Values:  "total", "compact", "full"
config.packages = "compact"

-- Shell
-- Default: "name"
-- Values:  "name", "path", "full"
config.shell = "name"

-- Terminal
-- Default: "name"
-- Values:  "name", "command", "full"
config.terminal = "full"

-- CPU
-- Default: "model_cores"
-- Values:  "model", "model_cores", "full"
config.cpu = "model_cores"

-- GPU
-- Default: "full"
-- Values:  "model", "model_cores", "full"
config.gpu = "full"

-- Memory
-- Default: "full"
-- Values:  "total", "used_total", "full"
config.memory = "full"

-- Battery
-- Default: "status"
-- Values:  "percent", "status", "full"
config.battery = "status"

-- Disk
-- Default: "mount_used_total"
-- Values:  "used_total", "mount_used_total", "full"
config.disk = "mount_used_total"

-- Display
-- Default: "resolution"
-- Values:  "resolution", "name_resolution", "full"
config.display = "resolution"
