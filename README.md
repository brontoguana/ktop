# ktop

![ktop screenshot](screenshot.png?v=6fadd5e)

A terminal-based system resource monitor built for tracking resource usage when running hybrid LLM workloads.

![Linux](https://img.shields.io/badge/platform-linux-blue)

## Features

### Latest

- Rewritten from Python to Rust — single static binary, near-zero CPU overhead, instant startup
- One-line install and upgrade: `curl -sSfL https://raw.githubusercontent.com/brontoguana/ktop/master/install.sh | bash`
- No runtime dependencies — no Python, no pip, no venv

### Core

- **GPU Monitoring** — Per-GPU utilization and memory usage with color-coded sparkline history (NVIDIA + AMD)
- **Network Monitoring** — Upload/download speeds with separate colored sparklines (upload extends up, download extends down)
- **CPU Monitoring** — Overall CPU usage with gradient bar chart and sparkline history
- **Memory Monitoring** — RAM and swap usage with gradient progress bars
- **Temperature Strip** — CPU, memory, and per-GPU temps with mini bar charts and hardware-accurate thresholds
- **OOM Kill Tracker** — Status bar shows the most recent OOM kill from the last 8 hours (kernel OOM and systemd-oomd)
- **Process Tables** — Top 10 processes by memory (Used/Shared) and CPU usage (Core % + system-wide CPU %)
- **50 Color Themes** — Press `t` to browse and switch themes with live preview; persists across sessions
- **Gradient Bar Charts** — Smooth per-block color gradients from low to high across all bars
- **Responsive UI** — 50ms input polling for snappy keyboard navigation

## Install

```bash
curl -sSfL https://raw.githubusercontent.com/brontoguana/ktop/master/install.sh | bash
```

Downloads the latest binary and installs it to `/usr/local/bin` (will prompt for sudo if needed). Run the same command again to upgrade.

### Build from source

```bash
git clone https://github.com/brontoguana/ktop.git
cd ktop/ktop-rs
cargo build --release
sudo cp target/release/ktop /usr/local/bin/
```

## Usage

```bash
# Run with defaults (1s refresh)
ktop

# Custom refresh rate
ktop -r 2

# Start with a specific theme
ktop --theme "Tokyo Night"

# Simulation mode (fake OOM kills, profiling to /tmp/ktop_profile.log)
ktop --sim

# Show version
ktop --version
```

### Keybindings

| Key | Action |
|-----|--------|
| `q` / `ESC` | Quit |
| `t` | Open theme picker |
| Arrow keys | Navigate theme picker |
| `Enter` | Select theme |

## Requirements

- Linux (reads `/proc` and sysfs directly)
- NVIDIA GPU + drivers (optional — for NVIDIA monitoring)
- AMD GPU + `amdgpu` driver (optional — for AMD monitoring)
- No runtime dependencies — single static binary

## License

MIT
