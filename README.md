# ktop

![ktop screenshot](static/screenshot.png?v=2)

A terminal-based system resource monitor built for tracking resource usage when running hybrid LLM workloads.

![Python 3.8+](https://img.shields.io/badge/python-3.8%2B-blue)

## Features

- **GPU Monitoring** — Per-GPU utilization and memory usage with color-coded sparkline history (NVIDIA)
- **Network Monitoring** — Upload and download speeds with auto-scaling bar charts and sparklines
- **CPU Monitoring** — Overall CPU usage with gradient bar chart and sparkline history
- **Memory Monitoring** — RAM and swap usage with gradient progress bars
- **Process Tables** — Top 10 processes by memory (Used/Shared) and CPU usage, updated in real-time
- **50 Color Themes** — Press `t` to browse and switch themes with live preview; persists across sessions
- **Gradient Bar Charts** — Smooth per-block color gradients from low to high across all bars
- **Responsive UI** — 50ms input polling for snappy keyboard navigation

## Install

```bash
# Install with uv (recommended)
uv tool install --from=git+https://github.com/brontoguana/ktop ktop
```

Or run directly without (separately) installing:

```bash
uvx --from=git+https://github.com/brontoguana/ktop ktop
```

**Note:** `uvx` is included with [`uv`](https://astral.sh/uv). See the uv documentation for installation instructions.

## Usage

```bash
# Run with defaults (1s refresh)
ktop

# Custom refresh rate
ktop -r 2

# Start with a specific theme
ktop --theme "Tokyo Night"
```

### Keybindings

| Key | Action |
|-----|--------|
| `q` / `ESC` | Quit |
| `t` | Open theme picker |
| Arrow keys | Navigate theme picker |
| `Enter` | Select theme |

## Requirements

- Python 3.8+
- NVIDIA GPU + drivers (optional — CPU/memory monitoring works without a GPU)
- Dependencies: `psutil`, `rich`, `pynvml`

## License

MIT
