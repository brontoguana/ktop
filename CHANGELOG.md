# Changelog

## 0.5.0 — 2026-02-10

- Added Network panel to the second row (layout is now Network, CPU, Memory)
- Shows upload and download bar charts with auto-scaling and sparkline history
- Displays current speed (B/s, KB/s, MB/s, GB/s) and peak observed speed
- Added `net` color to theme system (defaults to CPU color for all existing themes)
- Theme picker preview and swatches updated to include network color
- Added spacing between GPU utilization sparkline and memory bar chart
- Tested: reinstalled via setup.sh

## 0.4.3 — 2026-02-10

- Memory values shown in GB or MB with max 1 decimal place
- Tested: reinstalled via setup.sh

## 0.4.2 — 2026-02-10

- Bar charts now render a smooth per-block gradient from bar_low to bar_high across the full 0-100% width
- Each filled block gets its own interpolated hex color via linear RGB lerp
- RGB conversion cached to avoid re-parsing color names every frame
- Tested: reinstalled via setup.sh

## 0.4.1 — 2026-02-10

- Memory process table now shows Used (RSS−shared) + Shared columns instead of just RSS
- Uses `memory_info().shared` from `/proc/statm` (instant) instead of `memory_full_info()` which reads `/proc/smaps` and was extremely slow on systems with large memory maps, causing ktop to hang on launch
- Tested: reinstalled via setup.sh, launches instantly

## 0.4.0 — 2026-02-10

- Fixed arrow key input: rewrote `_read_key` to use `os.read()` on raw fd instead of buffered `sys.stdin.read()`, so escape sequences are captured atomically
- Responsive input loop: keys polled at 50ms with immediate redraw on keypress
- Theme picker: color swatches (background-colored chips for gpu/cpu/mem/bar) right-aligned next to each theme name with gaps between colors
- Sparklines aligned with bar chart left edge in GPU and CPU panels
- Bar charts now render as gradients: green→yellow→red across the filled region (thresholds at 50% and 80%)
- Tested: reinstalled via setup.sh

## 0.3.1 — 2026-02-10

- Sparklines now match the color of their metric (GPU util, GPU mem, CPU) instead of default white
- Sparklines dynamically fill the width of their enclosing panel (with margin) instead of fixed 20 chars
- Increased history buffer from 60 to 300 samples to support wide terminals
- Tested: reinstalled via setup.sh

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
