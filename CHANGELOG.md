# Changelog

## 0.3.0 — 2026-02-10

- Added theme system with 50 color themes (press `t` to open theme picker)
- Arrow keys + Enter to select theme, ESC to cancel
- Theme preference saved to `~/.config/ktop/config.json` and persists across sessions
- `--theme` CLI flag to set theme from command line
- Bottom status bar showing keybindings (q/ESC quit, t themes)
- Proper arrow key handling for theme picker navigation
- Tested: both main view and theme picker render correctly; config persistence works

## 0.2.0 — 2026-02-10

- GPU panels now laid out horizontally — all GPUs visible side by side
- Added q and ESC keys to quit the app (in addition to Ctrl+C)
- Tested: horizontal layout renders correctly with 3 GPUs at 140-col width

## 0.1.1 — 2026-02-10

- Added `setup.sh` installer script — installs `ktop` as a command to `~/.local/bin` (or `/usr/local/bin` with `--system`)
- Suppressed pynvml FutureWarning deprecation noise
- Updated README with quick-install instructions
- Tested: `ktop` command works system-wide after `./setup.sh`

## 0.1.0 — 2026-02-10

- Initial release of ktop
- GPU utilization and memory monitoring with per-GPU sparkline history (NVIDIA)
- CPU usage monitoring with overall bar chart and sparkline history
- RAM and swap usage bar charts
- Top 10 processes by memory usage table
- Top 10 processes by CPU usage table
- Color-coded thresholds (green/yellow/red)
- Configurable refresh rate via `-r` flag
- Tested: renders correctly with 3x NVIDIA RTX 2000 Ada GPUs, 128-core CPU, ~1 TB RAM
