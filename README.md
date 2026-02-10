# ktop

A terminal-based system resource monitor built for tracking resource usage when running hybrid LLM workloads.

![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue)

## Features

- **GPU Monitoring** — Per-GPU utilization and memory usage with rolling sparkline history (NVIDIA)
- **CPU Monitoring** — Overall CPU usage with bar chart and sparkline history
- **Memory Monitoring** — RAM and swap usage with progress bars
- **Process Tables** — Top 10 processes by memory and CPU usage, updated in real-time
- **Color-coded** — Green/yellow/red thresholds so you can spot pressure at a glance

## Screenshot

```
╭──────────────────── GPU Utilization & Memory ─────────────────────╮
│ GPU 0  NVIDIA RTX 2000 Ada Generation                             │
│   Util  ████████░░░░░░░ 52.0%  ▁▃▅▇█▇▅▃                         │
│   Mem   ██████████░░░░░ 8.2/16.0 GB                              │
│ GPU 1  NVIDIA RTX 2000 Ada Generation                             │
│   Util  ████░░░░░░░░░░░ 28.0%  ▁▂▃▄▃▂                           │
│   Mem   ███░░░░░░░░░░░░ 4.1/16.0 GB                              │
╰───────────────────────────────────────────────────────────────────╯
╭────────── CPU ──────────╮╭────────── Memory ─────────╮
│ Overall ██████░░░ 45.1% ││ RAM  ████░░░░░░ 16.6%     │
│ Cores: 128              ││ Used: 165 GB / 995 GB     │
│ History ▁▃▅▇█▇▅▃▁▃▅    ││ Swap  ░░░░░░░░░  0.0%     │
╰─────────────────────────╯╰───────────────────────────╯
╭── Top Processes (Memory) ─╮╭── Top Processes (CPU) ────╮
│ PID     Name      Mem %   ││ PID     Name      CPU %   │
│ 12345   python3   37.9%   ││ 12345   python    106.3%  │
│ ...                       ││ ...                       │
╰───────────────────────────╯╰───────────────────────────╯
```

## Install

```bash
git clone https://github.com/aemiguel/ktop.git
cd ktop
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Usage

```bash
# Run with defaults (1s refresh)
python ktop.py

# Custom refresh rate
python ktop.py -r 2

# Or make it executable
chmod +x ktop.py
./ktop.py
```

Press `Ctrl+C` to exit.

## Requirements

- Python 3.10+
- NVIDIA GPU + drivers (optional — CPU/memory monitoring works without a GPU)
- Dependencies: `psutil`, `rich`, `pynvml`

## License

MIT
