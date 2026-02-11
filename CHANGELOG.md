# Changelog

## 0.8.1 — 2026-02-11

- OOM tracker now detects `systemd-oomd` kills in addition to kernel OOM kills
- Fixed `capture_output=True` + `stderr=DEVNULL` conflict that silently broke OOM detection entirely
- Uses `short-unix` journal output for reliable timestamp comparison between OOM sources
- Scope names cleaned up: strips `.scope` suffix and UUIDs for readable display (e.g. `tmux-spawn`)
- Tested: verified both kernel OOM and systemd-oomd kills detected from real journal entries, reinstalled via setup.sh

## 0.8.0 — 2026-02-10

- CPU process table now shows both Core % (per-core, matches `top`) and CPU % (system-wide)
- Fixed per-process CPU calculation: no longer divides by num_cpus
- Raw binary I/O for `/proc` reads: `os.open`/`os.read`/`os.close` instead of Python file objects
- Binary mode (`rb`) and `split(None, 22)` to avoid decoding overhead and unnecessary allocations
- CPU frequency read from sysfs (`scaling_cur_freq`) instead of `psutil.cpu_freq()` (9ms → 0.02ms), with psutil fallback
- Deferred `/proc/pid/statm` reads: only read for the top 20 displayed processes instead of all ~1690
- `cpu_count` cached at init; `cpu_freq` polled every 5s
- Process CPU baselines seeded at startup for accurate first-frame deltas
- Process tables populate after 1s, refresh every 3s
- Total frame time: 228ms → 20ms (11x improvement)
- Tested: profiled with `--sim`, reinstalled via setup.sh

## 0.7.0 — 2026-02-10

- Process scanning optimized: replaced `psutil.process_iter` with direct `/proc/pid/stat` + `/proc/pid/statm` reads (214ms → 23ms per scan, ~9x faster)
- Process list cached for 5 seconds instead of rescanning every frame
- Added `--sim` flag for simulation mode (fake OOM kills, profiling output to `/tmp/ktop_profile.log`)
- Profiler logs avg/max/calls per section every 5s in sim mode
- OOM kill tracker uses `journalctl` for persistent 8-hour lookback instead of `dmesg` kernel ring buffer
- OOM status shows solid block `█` when OOM detected, hollow `░` when clear
- Tested: profiled with `--sim`, reinstalled via setup.sh

## 0.6.0 — 2026-02-10

- Network panel: sparklines now centered between bar charts — upload sparkline extends upward, download sparkline extends downward using upper-block Unicode characters
- Added `SPARK_DOWN` character set and `_sparkline_down()` for top-down sparklines
- Network upload and download now have separate theme colors (`net_up` defaults to GPU color, `net_down` defaults to net color)
- Theme picker swatches updated to show net_up/net_down colors
- Status bar now shows most recent OOM kill (process name + timestamp) on the right side
- Temperature strip between charts and process tables with hardware-accurate thresholds (GPU slowdown from NVML, CPU critical from psutil, 85°C JEDEC for memory)
- Temperature strip border uses theme `bar_mid` color; entries evenly spaced
- GPU bar charts now use dynamic width matching CPU/memory panel sizing
- Tested: reinstalled via setup.sh

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
