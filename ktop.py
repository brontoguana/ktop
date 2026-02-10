#!/usr/bin/env python3
"""ktop - Terminal system resource monitor for hybrid LLM workloads."""
from __future__ import annotations

import argparse
import json
import os
import random
import re
import select
import signal
import subprocess
import sys
import termios
import time
import tty
from collections import deque
from datetime import datetime, timedelta
from pathlib import Path
from types import SimpleNamespace

import psutil
from rich.color import Color
from rich.console import Console
from rich.layout import Layout
from rich.live import Live
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

try:
    import warnings

    with warnings.catch_warnings():
        warnings.simplefilter("ignore", FutureWarning)
        from pynvml import (
            NVML_TEMPERATURE_GPU,
            NVML_TEMPERATURE_THRESHOLD_SLOWDOWN,
            nvmlDeviceGetCount,
            nvmlDeviceGetHandleByIndex,
            nvmlDeviceGetMemoryInfo,
            nvmlDeviceGetName,
            nvmlDeviceGetTemperature,
            nvmlDeviceGetTemperatureThreshold,
            nvmlDeviceGetUtilizationRates,
            nvmlInit,
            nvmlShutdown,
        )

    _PYNVML = True
except ImportError:
    _PYNVML = False

# ── constants ────────────────────────────────────────────────────────────────
SPARK = " ▁▂▃▄▅▆▇█"
SPARK_DOWN = " ▔\U0001FB82\U0001FB83▀\U0001FB84\U0001FB85\U0001FB86█"
HISTORY_LEN = 300
CONFIG_DIR = Path.home() / ".config" / "ktop"
CONFIG_FILE = CONFIG_DIR / "config.json"

# ── themes ───────────────────────────────────────────────────────────────────
# Each theme: (gpu, cpu, mem, proc_mem, proc_cpu, bar_low, bar_mid, bar_high)
THEMES: dict[str, dict] = {}


def _t(name, gpu, cpu, mem, pm, pc, lo, mid, hi, net=None, net_up=None, net_down=None):
    THEMES[name] = dict(gpu=gpu, cpu=cpu, mem=mem, proc_mem=pm, proc_cpu=pc, bar_low=lo, bar_mid=mid, bar_high=hi, net=net or cpu, net_up=net_up or gpu, net_down=net_down or (net or cpu))


# ── Classic & editor themes ──
_t("Default",            "magenta",      "cyan",         "green",        "green",        "cyan",         "green",    "yellow",   "red")
_t("Monokai",           "bright_magenta","bright_cyan",  "bright_green", "bright_green", "bright_cyan",  "green",    "yellow",   "red")
_t("Dracula",           "#bd93f9",       "#8be9fd",      "#50fa7b",      "#50fa7b",      "#8be9fd",      "#50fa7b",  "#f1fa8c",  "#ff5555")
_t("Nord",              "#b48ead",       "#88c0d0",      "#a3be8c",      "#a3be8c",      "#88c0d0",      "#a3be8c",  "#ebcb8b",  "#bf616a")
_t("Solarized",         "#d33682",       "#2aa198",      "#859900",      "#859900",      "#2aa198",      "#859900",  "#b58900",  "#dc322f")
_t("Gruvbox",           "#d3869b",       "#83a598",      "#b8bb26",      "#b8bb26",      "#83a598",      "#b8bb26",  "#fabd2f",  "#fb4934")
_t("One Dark",          "#c678dd",       "#56b6c2",      "#98c379",      "#98c379",      "#56b6c2",      "#98c379",  "#e5c07b",  "#e06c75")
_t("Tokyo Night",       "#bb9af7",       "#7dcfff",      "#9ece6a",      "#9ece6a",      "#7dcfff",      "#9ece6a",  "#e0af68",  "#f7768e")
_t("Catppuccin Mocha",  "#cba6f7",       "#89dceb",      "#a6e3a1",      "#a6e3a1",      "#89dceb",      "#a6e3a1",  "#f9e2af",  "#f38ba8")
_t("Catppuccin Latte",  "#8839ef",       "#04a5e5",      "#40a02b",      "#40a02b",      "#04a5e5",      "#40a02b",  "#df8e1d",  "#d20f39")
_t("Rosé Pine",         "#c4a7e7",       "#9ccfd8",      "#31748f",      "#31748f",      "#9ccfd8",      "#31748f",  "#f6c177",  "#eb6f92")
_t("Everforest",        "#d699b6",       "#7fbbb3",      "#a7c080",      "#a7c080",      "#7fbbb3",      "#a7c080",  "#dbbc7f",  "#e67e80")
_t("Kanagawa",          "#957fb8",       "#7e9cd8",      "#98bb6c",      "#98bb6c",      "#7e9cd8",      "#98bb6c",  "#e6c384",  "#c34043")

# ── Monochrome / minimal ──
_t("Monochrome",        "white",         "white",        "white",        "white",        "white",        "bright_white","white",  "#888888")
_t("Green Screen",      "green",         "green",        "green",        "green",        "green",        "bright_green","green",  "dark_green")
_t("Amber",             "#ffbf00",       "#ffbf00",      "#ffbf00",      "#ffbf00",      "#ffbf00",      "#ffd700",  "#ffbf00",  "#ff8c00")
_t("Phosphor",          "#33ff00",       "#33ff00",      "#33ff00",      "#33ff00",      "#33ff00",      "#66ff33",  "#33ff00",  "#009900")

# ── Color themes ──
_t("Ocean",             "#6c5ce7",       "#0984e3",      "#00b894",      "#00b894",      "#0984e3",      "#00b894",  "#fdcb6e",  "#d63031")
_t("Sunset",            "#e17055",       "#fdcb6e",      "#fab1a0",      "#fab1a0",      "#fdcb6e",      "#ffeaa7",  "#e17055",  "#d63031")
_t("Forest",            "#00b894",       "#55efc4",      "#00cec9",      "#00cec9",      "#55efc4",      "#55efc4",  "#ffeaa7",  "#e17055")
_t("Lava",              "#ff6348",       "#ff4757",      "#ff6b81",      "#ff6b81",      "#ff4757",      "#ffa502",  "#ff6348",  "#ff3838")
_t("Arctic",            "#dfe6e9",       "#74b9ff",      "#81ecec",      "#81ecec",      "#74b9ff",      "#81ecec",  "#74b9ff",  "#a29bfe")
_t("Sakura",            "#fd79a8",       "#e84393",      "#fab1a0",      "#fab1a0",      "#e84393",      "#fab1a0",  "#fd79a8",  "#e84393")
_t("Mint",              "#00b894",       "#00cec9",      "#55efc4",      "#55efc4",      "#00cec9",      "#55efc4",  "#81ecec",  "#ff7675")
_t("Lavender",          "#a29bfe",       "#6c5ce7",      "#dfe6e9",      "#dfe6e9",      "#6c5ce7",      "#a29bfe",  "#6c5ce7",  "#fd79a8")
_t("Coral",             "#ff7675",       "#fab1a0",      "#ffeaa7",      "#ffeaa7",      "#fab1a0",      "#ffeaa7",  "#ff7675",  "#d63031")
_t("Cyberpunk",         "#ff00ff",       "#00ffff",      "#ff00aa",      "#ff00aa",      "#00ffff",      "#00ff00",  "#ffff00",  "#ff0000")
_t("Neon",              "#ff6ec7",       "#00ffff",      "#39ff14",      "#39ff14",      "#00ffff",      "#39ff14",  "#ffff00",  "#ff073a")
_t("Synthwave",         "#f72585",       "#4cc9f0",      "#7209b7",      "#7209b7",      "#4cc9f0",      "#4cc9f0",  "#f72585",  "#ff0a54")
_t("Vaporwave",         "#ff71ce",       "#01cdfe",      "#05ffa1",      "#05ffa1",      "#01cdfe",      "#05ffa1",  "#b967ff",  "#ff71ce")
_t("Matrix",            "#00ff41",       "#008f11",      "#003b00",      "#003b00",      "#008f11",      "#00ff41",  "#008f11",  "#003b00")

# ── Pastel & soft ──
_t("Pastel",            "#c39bd3",       "#85c1e9",      "#82e0aa",      "#82e0aa",      "#85c1e9",      "#82e0aa",  "#f9e79f",  "#f1948a")
_t("Soft",              "#bb8fce",       "#76d7c4",      "#7dcea0",      "#7dcea0",      "#76d7c4",      "#7dcea0",  "#f0b27a",  "#ec7063")
_t("Cotton Candy",      "#ffb3ba",       "#bae1ff",      "#baffc9",      "#baffc9",      "#bae1ff",      "#baffc9",  "#ffffba",  "#ffb3ba")
_t("Ice Cream",         "#ff9a9e",       "#a1c4fd",      "#c2e9fb",      "#c2e9fb",      "#a1c4fd",      "#c2e9fb",  "#ffecd2",  "#ff9a9e")

# ── Bold & vivid ──
_t("Electric",          "#7b2ff7",       "#00d4ff",      "#00ff87",      "#00ff87",      "#00d4ff",      "#00ff87",  "#ffd000",  "#ff0055")
_t("Inferno",           "#ff4500",       "#ff6a00",      "#ff8c00",      "#ff8c00",      "#ff6a00",      "#ffd700",  "#ff8c00",  "#ff0000")
_t("Glacier",           "#e0f7fa",       "#80deea",      "#4dd0e1",      "#4dd0e1",      "#80deea",      "#80deea",  "#4dd0e1",  "#00838f")
_t("Twilight",          "#7c4dff",       "#448aff",      "#18ffff",      "#18ffff",      "#448aff",      "#18ffff",  "#7c4dff",  "#ff1744")
_t("Autumn",            "#d35400",       "#e67e22",      "#f39c12",      "#f39c12",      "#e67e22",      "#f1c40f",  "#e67e22",  "#c0392b")
_t("Spring",            "#e91e63",       "#00bcd4",      "#8bc34a",      "#8bc34a",      "#00bcd4",      "#8bc34a",  "#ffeb3b",  "#f44336")
_t("Summer",            "#ff9800",       "#03a9f4",      "#4caf50",      "#4caf50",      "#03a9f4",      "#4caf50",  "#ffeb3b",  "#f44336")
_t("Winter",            "#9c27b0",       "#3f51b5",      "#607d8b",      "#607d8b",      "#3f51b5",      "#607d8b",  "#9c27b0",  "#e91e63")

# ── High contrast / accessibility ──
_t("High Contrast",     "bright_magenta","bright_cyan",  "bright_green", "bright_green", "bright_cyan",  "bright_green","bright_yellow","bright_red")
_t("Blueprint",         "#4fc3f7",       "#29b6f6",      "#03a9f4",      "#03a9f4",      "#29b6f6",      "#4fc3f7",  "#0288d1",  "#01579b")
_t("Redshift",          "#ef5350",       "#e53935",      "#c62828",      "#c62828",      "#e53935",      "#ef9a9a",  "#ef5350",  "#b71c1c")
_t("Emerald",           "#66bb6a",       "#43a047",      "#2e7d32",      "#2e7d32",      "#43a047",      "#a5d6a7",  "#66bb6a",  "#1b5e20")
_t("Royal",             "#7e57c2",       "#5c6bc0",      "#42a5f5",      "#42a5f5",      "#5c6bc0",      "#42a5f5",  "#7e57c2",  "#d32f2f")
_t("Bubblegum",         "#ff77a9",       "#ff99cc",      "#ffb3d9",      "#ffb3d9",      "#ff99cc",      "#ffb3d9",  "#ff77a9",  "#ff3385")
_t("Horizon",           "#e95678",       "#fab795",      "#25b0bc",      "#25b0bc",      "#fab795",      "#25b0bc",  "#fab795",  "#e95678")

THEME_NAMES = list(THEMES.keys())


# ── config persistence ───────────────────────────────────────────────────────
def _load_config() -> dict:
    try:
        return json.loads(CONFIG_FILE.read_text())
    except Exception:
        return {}


def _save_config(cfg: dict) -> None:
    CONFIG_DIR.mkdir(parents=True, exist_ok=True)
    CONFIG_FILE.write_text(json.dumps(cfg, indent=2) + "\n")


# ── helpers ──────────────────────────────────────────────────────────────────
_rgb_cache: dict[str, tuple[int, int, int]] = {}


def _color_to_rgb(name: str) -> tuple[int, int, int]:
    """Parse a Rich color name or hex string to (r, g, b). Cached."""
    if name not in _rgb_cache:
        tc = Color.parse(name).get_truecolor()
        _rgb_cache[name] = (tc.red, tc.green, tc.blue)
    return _rgb_cache[name]


def _lerp_rgb(c1: tuple[int, int, int], c2: tuple[int, int, int], t: float) -> str:
    """Linearly interpolate two RGB tuples, return hex string."""
    r = int(c1[0] + (c2[0] - c1[0]) * t)
    g = int(c1[1] + (c2[1] - c1[1]) * t)
    b = int(c1[2] + (c2[2] - c1[2]) * t)
    return f"#{r:02x}{g:02x}{b:02x}"


def _bar(pct: float, width: int = 25, theme: dict | None = None) -> str:
    """Render a smooth gradient progress bar as Rich markup."""
    filled = int(pct / 100 * width)
    empty = width - filled
    rgb_lo = _color_to_rgb(theme["bar_low"] if theme else "green")
    rgb_hi = _color_to_rgb(theme["bar_high"] if theme else "red")

    parts = []
    for i in range(filled):
        t = i / max(width - 1, 1)
        c = _lerp_rgb(rgb_lo, rgb_hi, t)
        parts.append(f"[{c}]█[/{c}]")
    if empty:
        parts.append(f"[dim]{'░' * empty}[/dim]")
    return "".join(parts)


def _color_for(pct: float, theme: dict | None = None) -> str:
    if theme:
        if pct < 50:
            return theme["bar_low"]
        if pct < 80:
            return theme["bar_mid"]
        return theme["bar_high"]
    if pct < 50:
        return "green"
    if pct < 80:
        return "yellow"
    return "red"


def _sparkline(values, width: int | None = None) -> str:
    if not values:
        return ""
    vals = list(values)
    if width and len(vals) > width:
        vals = vals[-width:]
    out = []
    for v in vals:
        v = max(0.0, min(100.0, v))
        idx = int(v / 100 * (len(SPARK) - 1))
        out.append(SPARK[idx])
    return "".join(out)


def _sparkline_down(values, width: int | None = None) -> str:
    """Sparkline with blocks extending downward from the top."""
    if not values:
        return ""
    vals = list(values)
    if width and len(vals) > width:
        vals = vals[-width:]
    out = []
    for v in vals:
        v = max(0.0, min(100.0, v))
        idx = int(v / 100 * (len(SPARK_DOWN) - 1))
        out.append(SPARK_DOWN[idx])
    return "".join(out)


def _fmt_bytes(b: float) -> str:
    mb = b / 1024**2
    if mb >= 1000:
        return f"{mb / 1024:.1f} GB"
    return f"{mb:.1f} MB"


def _fmt_speed(b: float) -> str:
    """Format bytes/sec as human-readable speed."""
    if b >= 1024**3:
        return f"{b / 1024**3:.1f} GB/s"
    if b >= 1024**2:
        return f"{b / 1024**2:.1f} MB/s"
    if b >= 1024:
        return f"{b / 1024:.1f} KB/s"
    return f"{b:.0f} B/s"


# ── keyboard input ───────────────────────────────────────────────────────────
def _read_key() -> str | None:
    """Non-blocking read of a single keypress. Returns key name or None."""
    fd = sys.stdin.fileno()
    if not select.select([fd], [], [], 0)[0]:
        return None
    data = os.read(fd, 64)
    if not data:
        return None
    if data[0] == 0x1B:
        if len(data) >= 3 and data[1] == ord("["):
            code = data[2]
            if code == ord("A"):
                return "UP"
            if code == ord("B"):
                return "DOWN"
            if code == ord("C"):
                return "RIGHT"
            if code == ord("D"):
                return "LEFT"
            return None
        if len(data) == 1:
            return "ESC"
        return None
    ch = chr(data[0])
    if ch in ("\r", "\n"):
        return "ENTER"
    return ch


# ── main monitor ─────────────────────────────────────────────────────────────
class KTop:
    def __init__(self, refresh: float = 1.0, sim: bool = False):
        self.refresh = refresh
        self.sim = sim
        self.console = Console()

        # theme
        cfg = _load_config()
        theme_name = cfg.get("theme", "Vaporwave")
        if theme_name not in THEMES:
            theme_name = "Default"
        self.theme_name = theme_name
        self.theme = THEMES[self.theme_name]

        # theme picker state
        self.picking_theme = False
        self.theme_cursor = THEME_NAMES.index(self.theme_name)
        self.theme_scroll = 0

        # rolling histories
        self.cpu_hist: deque[float] = deque(maxlen=HISTORY_LEN)

        # network state
        self.net_up_hist: deque[float] = deque(maxlen=HISTORY_LEN)
        self.net_down_hist: deque[float] = deque(maxlen=HISTORY_LEN)
        self.net_max_speed: float = 1.0  # auto-scale ceiling in bytes/sec
        counters = psutil.net_io_counters()
        self._last_net_sent = counters.bytes_sent
        self._last_net_recv = counters.bytes_recv
        self._last_net_time = time.monotonic()

        # GPU init
        self.gpu_ok = False
        self.gpu_count = 0
        self.gpu_util_hist: dict[int, deque] = {}
        self.gpu_mem_hist: dict[int, deque] = {}

        if _PYNVML:
            try:
                nvmlInit()
                self.gpu_count = nvmlDeviceGetCount()
                self.gpu_ok = True
                for i in range(self.gpu_count):
                    self.gpu_util_hist[i] = deque(maxlen=HISTORY_LEN)
                    self.gpu_mem_hist[i] = deque(maxlen=HISTORY_LEN)
            except Exception:
                pass

        # process cache (scanned at most every 5s, read from /proc directly)
        self._procs_by_mem: list[dict] = []
        self._procs_by_cpu: list[dict] = []
        self._last_proc_scan = 0.0
        self._proc_cpu_prev: dict[int, int] = {}
        self._page_size = os.sysconf("SC_PAGE_SIZE")
        self._clock_ticks = os.sysconf("SC_CLK_TCK")
        self._num_cpus = os.cpu_count() or 1

        # OOM kill tracking
        self._last_oom_check = 0.0
        self._last_oom_str: str | None = None

        # profiling (sim mode only)
        self._prof_log: Path | None = None
        self._prof_accum: dict[str, list[float]] = {}
        self._prof_last_flush = 0.0
        self._prof_frame = 0
        if self.sim:
            self._prof_log = Path("/tmp/ktop_profile.log")
            self._prof_log.write_text(f"ktop profile started {datetime.now().isoformat()}\n")

        # CPU info (static, cache once)
        self._cpu_cores = psutil.cpu_count(logical=True)
        self._cpu_freq_str = "N/A"
        self._last_freq_check = 0.0

        psutil.cpu_percent(interval=None)

    # ── data collectors ──────────────────────────────────────────────────
    def _sample_cpu(self) -> float:
        pct = psutil.cpu_percent(interval=None)
        self.cpu_hist.append(pct)
        return pct

    def _sample_net(self) -> tuple[float, float]:
        """Sample network and return (upload_bytes_sec, download_bytes_sec)."""
        counters = psutil.net_io_counters()
        now = time.monotonic()
        dt = now - self._last_net_time
        if dt <= 0:
            dt = 1.0
        up = (counters.bytes_sent - self._last_net_sent) / dt
        down = (counters.bytes_recv - self._last_net_recv) / dt
        self._last_net_sent = counters.bytes_sent
        self._last_net_recv = counters.bytes_recv
        self._last_net_time = now
        # Auto-scale: track max observed speed
        self.net_max_speed = max(self.net_max_speed, up, down, 1.0)
        self.net_up_hist.append(up)
        self.net_down_hist.append(down)
        return up, down

    def _sample_temps(self) -> dict:
        """Collect temperature readings with hardware limits."""
        temps = {"cpu": None, "cpu_max": 100.0, "mem": None, "mem_max": 85.0, "gpus": []}
        # CPU and memory temps from psutil (includes high/critical thresholds)
        try:
            sensor_temps = psutil.sensors_temperatures()
            for key in ("coretemp", "k10temp", "cpu_thermal", "zenpower", "acpitz"):
                if key in sensor_temps:
                    for s in sensor_temps[key]:
                        if temps["cpu"] is None or s.current > temps["cpu"]:
                            temps["cpu"] = s.current
                        # Use reported critical/high as the max
                        if s.critical and s.critical > temps["cpu_max"]:
                            temps["cpu_max"] = s.critical
                        elif s.high and s.high > temps["cpu_max"]:
                            temps["cpu_max"] = s.high
            for key in ("SODIMM", "dimm", "memory"):
                if key in sensor_temps:
                    for s in sensor_temps[key]:
                        if temps["mem"] is None or s.current > temps["mem"]:
                            temps["mem"] = s.current
                        if s.critical:
                            temps["mem_max"] = s.critical
        except (AttributeError, Exception):
            pass
        # GPU temps + slowdown threshold from pynvml
        if self.gpu_ok:
            for i in range(self.gpu_count):
                try:
                    h = nvmlDeviceGetHandleByIndex(i)
                    t = nvmlDeviceGetTemperature(h, NVML_TEMPERATURE_GPU)
                    try:
                        t_max = nvmlDeviceGetTemperatureThreshold(h, NVML_TEMPERATURE_THRESHOLD_SLOWDOWN)
                    except Exception:
                        t_max = 95
                    temps["gpus"].append({"temp": t, "max": t_max})
                except Exception:
                    temps["gpus"].append(None)
        return temps

    def _gpu_info(self) -> list[dict]:
        gpus = []
        if not self.gpu_ok:
            return gpus
        for i in range(self.gpu_count):
            try:
                h = nvmlDeviceGetHandleByIndex(i)
                name = nvmlDeviceGetName(h)
                if isinstance(name, bytes):
                    name = name.decode()
                util = nvmlDeviceGetUtilizationRates(h)
                mem = nvmlDeviceGetMemoryInfo(h)
                mem_pct = mem.used / mem.total * 100 if mem.total else 0
                self.gpu_util_hist[i].append(util.gpu)
                self.gpu_mem_hist[i].append(mem_pct)
                gpus.append(
                    {
                        "id": i,
                        "name": name,
                        "util": util.gpu,
                        "mem_used_gb": mem.used / 1024**3,
                        "mem_total_gb": mem.total / 1024**3,
                        "mem_pct": mem_pct,
                    }
                )
            except Exception:
                pass
        return gpus

    def _scan_procs(self) -> None:
        """Scan process list from /proc directly, cached for 5 seconds."""
        now = time.monotonic()
        if now - self._last_proc_scan < 5.0 and self._procs_by_mem:
            return
        dt = now - self._last_proc_scan if self._last_proc_scan > 0 else 1.0
        self._last_proc_scan = now
        total_mem = psutil.virtual_memory().total
        ps = self._page_size
        procs = []
        for pid_str in os.listdir("/proc"):
            if not pid_str.isdigit():
                continue
            pid = int(pid_str)
            if pid == 0:
                continue
            try:
                with open(f"/proc/{pid}/stat") as f:
                    stat = f.read()
                # Name is between first ( and last ) to handle parens in names
                i0 = stat.index("(") + 1
                i1 = stat.rindex(")")
                name = stat[i0:i1]
                fields = stat[i1 + 2:].split()
                utime = int(fields[11])   # field 14
                stime = int(fields[12])   # field 15
                rss_pages = int(fields[21])  # field 24
                with open(f"/proc/{pid}/statm") as f:
                    shared_pages = int(f.read().split()[2])
                rss = rss_pages * ps
                shared = shared_pages * ps
                mem_pct = rss / total_mem * 100 if total_mem else 0
                cpu_total = utime + stime
                prev = self._proc_cpu_prev.get(pid, cpu_total)
                cpu_delta = cpu_total - prev
                self._proc_cpu_prev[pid] = cpu_total
                cpu_pct = (cpu_delta / self._clock_ticks) / dt / self._num_cpus * 100 if dt > 0 else 0
                procs.append({
                    "pid": pid, "name": name[:28],
                    "cpu_percent": cpu_pct, "memory_percent": mem_pct,
                    "memory_info": SimpleNamespace(rss=rss, shared=shared),
                })
            except (FileNotFoundError, PermissionError, IndexError, ValueError, ProcessLookupError, OSError):
                continue
        # Clean stale PIDs
        current = {p["pid"] for p in procs}
        self._proc_cpu_prev = {k: v for k, v in self._proc_cpu_prev.items() if k in current}
        self._procs_by_mem = sorted(procs, key=lambda x: x.get("memory_percent", 0) or 0, reverse=True)[:10]
        self._procs_by_cpu = sorted(procs, key=lambda x: x.get("cpu_percent", 0) or 0, reverse=True)[:10]

    def _top_procs(self, key: str) -> list[dict]:
        if key == "memory_percent":
            return self._procs_by_mem
        return self._procs_by_cpu

    _SIM_PROCS = ["python3", "node", "java", "ollama", "vllm", "ffmpeg", "cc1plus", "rustc", "chrome", "mysqld"]

    def _check_oom(self) -> str | None:
        """Return most recent OOM kill in last 8h via journalctl, cached for 5s."""
        now = time.monotonic()
        if now - self._last_oom_check < 5.0:
            return self._last_oom_str
        self._last_oom_check = now
        if self.sim:
            if random.random() < 0.5:
                self._last_oom_str = None
            else:
                fake_time = datetime.now() - timedelta(seconds=random.randint(0, 8 * 3600))
                proc = random.choice(self._SIM_PROCS)
                self._last_oom_str = f"{fake_time.strftime('%b %d %H:%M:%S')} {proc}"
            return self._last_oom_str
        try:
            result = subprocess.run(
                ["journalctl", "-k", "--since", "8 hours ago",
                 "--no-pager", "-o", "short", "--grep", "Killed process"],
                capture_output=True, text=True, timeout=3,
                stderr=subprocess.DEVNULL,
            )
            if result.returncode != 0 or not result.stdout.strip():
                self._last_oom_str = None
                return None
            last_oom = result.stdout.strip().splitlines()[-1]
            # Format: "Feb 10 14:23:01 host kernel: ... Killed process 1234 (name) ..."
            ts_match = re.match(r"^(\w+ \d+ \d+:\d+:\d+)", last_oom)
            proc_match = re.search(r"Killed process \d+ \(([^)]+)\)", last_oom)
            ts = ts_match.group(1) if ts_match else "?"
            proc = proc_match.group(1) if proc_match else "?"
            self._last_oom_str = f"{ts} {proc}"
        except Exception:
            self._last_oom_str = None
        return self._last_oom_str

    # ── panel builders ───────────────────────────────────────────────────
    def _gpu_panels(self) -> Layout:
        t = self.theme
        gpus = self._gpu_info()
        if not gpus:
            return Panel(
                Text("No GPUs detected (install pynvml for GPU monitoring)", style="dim italic"),
                title=f"[bold {t['gpu']}] GPU [/bold {t['gpu']}]",
                border_style=t["gpu"],
            )

        gpu_layout = Layout()
        # Panel inner width: total / num_gpus, minus border(2) + padding(2) + safety(2)
        panel_w = max(20, self.console.width // max(len(gpus), 1) - 6)
        # "Util " / "Mem  " = 5 chars, " XX.X%" = 7 chars
        bar_w = max(5, panel_w - 5 - 7)
        spark_w = max(10, panel_w - 5)
        children = []
        for g in gpus:
            uc = _color_for(g["util"], t)
            mc = _color_for(g["mem_pct"], t)
            spark_u = _sparkline(self.gpu_util_hist[g["id"]], width=spark_w)
            spark_m = _sparkline(self.gpu_mem_hist[g["id"]], width=spark_w)
            body = (
                f"[bold]Util[/bold] {_bar(g['util'], bar_w, t)} [{uc}]{g['util']:5.1f}%[/{uc}]\n"
                f"     [{uc}]{spark_u}[/{uc}]\n"
                f"\n"
                f"[bold]Mem [/bold] {_bar(g['mem_pct'], bar_w, t)} [{mc}]{g['mem_pct']:5.1f}%[/{mc}]\n"
                f"     {g['mem_used_gb']:.1f}/{g['mem_total_gb']:.1f} GB\n"
                f"     [{mc}]{spark_m}[/{mc}]"
            )
            name_short = g["name"].replace("NVIDIA ", "").replace(" Generation", "")
            panel = Panel(
                Text.from_markup(body),
                title=f"[bold {t['gpu']}] GPU {g['id']} [/bold {t['gpu']}]",
                subtitle=f"[dim]{name_short}[/dim]",
                border_style=t["gpu"],
            )
            child = Layout(name=f"gpu{g['id']}", ratio=1)
            child.update(panel)
            children.append(child)

        gpu_layout.split_row(*children)
        return gpu_layout

    def _cpu_panel(self) -> Panel:
        t = self.theme
        pct = self._sample_cpu()
        c = _color_for(pct, t)
        # Refresh frequency every 5s, core count is cached from init
        now = time.monotonic()
        if now - self._last_freq_check >= 5.0:
            self._last_freq_check = now
            freq = psutil.cpu_freq()
            if freq:
                self._cpu_freq_str = f"{freq.current:.0f} MHz"

        # Panel inner width: third of terminal minus border(2) + padding(2) + safety(2)
        panel_w = max(20, self.console.width // 3 - 6)
        # "Overall  " = 9 chars, " XX.X%" = 7 chars (space + 5-wide float + %)
        bar_w = max(5, panel_w - 9 - 7)
        spark_w = max(10, panel_w - 9)
        spark = _sparkline(self.cpu_hist, width=spark_w)
        body = (
            f"[bold]Overall[/bold]  {_bar(pct, bar_w, t)} [{c}]{pct:5.1f}%[/{c}]\n"
            f"[dim]Cores: {self._cpu_cores}  Freq: {self._cpu_freq_str}[/dim]\n\n"
            f"[bold]History[/bold]\n         [{c}]{spark}[/{c}]"
        )
        return Panel(
            Text.from_markup(body),
            title=f"[bold {t['cpu']}] CPU [/bold {t['cpu']}]",
            border_style=t["cpu"],
        )

    def _net_panel(self) -> Panel:
        t = self.theme
        up, down = self._sample_net()
        mx = self.net_max_speed
        up_pct = min(100.0, up / mx * 100) if mx else 0
        down_pct = min(100.0, down / mx * 100) if mx else 0

        # Panel inner width: third of terminal minus border(2) + padding(2) + safety(2)
        panel_w = max(20, self.console.width // 3 - 6)
        # "Up   " / "Down " = 5 chars, " XXXXXXX" speed = 11 chars
        bar_w = max(5, panel_w - 5 - 11)
        spark_w = max(10, panel_w - 5)
        spark_up = _sparkline(
            [min(100.0, v / mx * 100) if mx else 0 for v in self.net_up_hist],
            width=spark_w,
        )
        spark_dn = _sparkline_down(
            [min(100.0, v / mx * 100) if mx else 0 for v in self.net_down_hist],
            width=spark_w,
        )

        nc = t["net"]
        uc = t["net_up"]
        dc = t["net_down"]
        body = (
            f"[bold]Up  [/bold] {_bar(up_pct, bar_w, t)} [{uc}]{_fmt_speed(up):>10}[/{uc}]\n"
            f"\n"
            f"     [{uc}]{spark_up}[/{uc}]\n"
            f"     [{dc}]{spark_dn}[/{dc}]\n"
            f"\n"
            f"[bold]Down[/bold] {_bar(down_pct, bar_w, t)} [{dc}]{_fmt_speed(down):>10}[/{dc}]\n"
            f"\n"
            f"[dim]Peak: {_fmt_speed(mx)}[/dim]"
        )
        return Panel(
            Text.from_markup(body),
            title=f"[bold {nc}] Network [/bold {nc}]",
            border_style=nc,
        )

    def _mem_panel(self) -> Panel:
        t = self.theme
        vm = psutil.virtual_memory()
        sw = psutil.swap_memory()

        used_pct = vm.percent
        c = _color_for(used_pct, t)

        # Panel inner width: third of terminal minus border(2) + padding(2) + safety(2)
        panel_w = max(20, self.console.width // 3 - 6)
        # "RAM  " / "Swap " = 5 chars, " XX.X%" = 7 chars (space + 5-wide float + %)
        bar_w = max(5, panel_w - 5 - 7)
        body = (
            f"[bold]RAM[/bold]  {_bar(used_pct, bar_w, t)} [{c}]{used_pct:5.1f}%[/{c}]\n"
            f"  {_fmt_bytes(vm.used)} used / {_fmt_bytes(vm.total)}\n\n"
            f"[bold]Swap[/bold] {_bar(sw.percent, bar_w, t)} [dim]{sw.percent:5.1f}%[/dim]\n"
            f"  {_fmt_bytes(sw.used)} used / {_fmt_bytes(sw.total)}"
        )
        return Panel(
            Text.from_markup(body),
            title=f"[bold {t['mem']}] Memory [/bold {t['mem']}]",
            border_style=t["mem"],
        )

    def _proc_table(self, by: str) -> Panel:
        t = self.theme
        is_mem = by == "memory_percent"
        procs = self._top_procs(by)
        title = "Top Processes by Memory" if is_mem else "Top Processes by CPU"
        colour = t["proc_mem"] if is_mem else t["proc_cpu"]

        table = Table(expand=True, box=None, pad_edge=False)
        table.add_column("PID", style="dim", width=8, justify="right")
        table.add_column("Name", ratio=2)
        if is_mem:
            table.add_column("Used", justify="right", width=10)
            table.add_column("Shared", justify="right", width=10)
            table.add_column("Mem %", justify="right", width=7)
        else:
            table.add_column("CPU %", justify="right", width=8)
            table.add_column("Mem %", justify="right", width=8)

        for p in procs:
            pid = str(p.get("pid", ""))
            name = (p.get("name") or "?")[:28]
            mem_pct = p.get("memory_percent") or 0
            cpu_pct = p.get("cpu_percent") or 0
            if is_mem:
                mi = p.get("memory_info")
                if mi:
                    mi_shared = getattr(mi, "shared", 0) or 0
                    used = _fmt_bytes(max(0, mi.rss - mi_shared))
                    shared = _fmt_bytes(mi_shared)
                else:
                    used = "N/A"
                    shared = "N/A"
                table.add_row(pid, name, used, shared, f"{mem_pct:.1f}%")
            else:
                table.add_row(pid, name, f"{cpu_pct:.1f}%", f"{mem_pct:.1f}%")

        return Panel(table, title=f"[bold {colour}] {title} [/bold {colour}]", border_style=colour)

    def _status_bar(self) -> Table:
        t = self.theme
        left = Text()
        left.append(" q", style=f"bold {t['cpu']}")
        left.append("/", style="dim")
        left.append("ESC", style=f"bold {t['cpu']}")
        left.append(" Quit  ", style="dim")
        left.append(" t", style=f"bold {t['gpu']}")
        left.append(f" Theme ({self.theme_name})  ", style="dim")

        right = Text()
        oom = self._check_oom()
        if oom:
            right.append("█ ", style=t["bar_high"])
            right.append("OOM Kill: ", style=f"bold {t['bar_high']}")
            right.append(oom + " ", style=t["bar_high"])
        else:
            right.append("░ ", style="dim")
            right.append("No OOM kills ", style="dim")

        bar = Table(box=None, pad_edge=False, show_header=False, expand=True)
        bar.add_column(ratio=1)
        bar.add_column(justify="right")
        bar.add_row(left, right)
        return bar

    def _temp_strip(self) -> Panel:
        """Temperature strip with mini bar charts, evenly spaced."""
        temps = self._sample_temps()
        t = self.theme

        def _temp_cell(label: str, temp_c: float | None, max_c: float = 100.0) -> Text:
            cell = Text()
            cell.append(f"{label} ", style="bold dim")
            if temp_c is None:
                cell.append("N/A", style="dim")
                return cell
            pct = min(100.0, temp_c / max_c * 100)
            ratio = temp_c / max_c
            if ratio < 0.6:
                c = t["bar_low"]
            elif ratio < 0.85:
                c = t["bar_mid"]
            else:
                c = t["bar_high"]
            filled = int(pct / 100 * 8)
            cell.append("█" * filled, style=c)
            cell.append("░" * (8 - filled), style="dim")
            cell.append(f" {temp_c:.0f}/{max_c:.0f}°C", style=c)
            return cell

        # Collect all temp cells
        cells = []
        if temps["cpu"] is not None:
            cells.append(_temp_cell("CPU", temps["cpu"], temps["cpu_max"]))
        if temps["mem"] is not None:
            cells.append(_temp_cell("MEM", temps["mem"], temps["mem_max"]))
        for i, gpu_t in enumerate(temps["gpus"]):
            if gpu_t is not None:
                cells.append(_temp_cell(f"GPU{i}", gpu_t["temp"], gpu_t["max"]))
            else:
                cells.append(_temp_cell(f"GPU{i}", None))

        # Build table with equal-width columns
        table = Table(expand=True, box=None, pad_edge=True, show_header=False)
        for _ in cells:
            table.add_column(ratio=1)
        if cells:
            table.add_row(*cells)

        tc = t["bar_mid"]
        return Panel(table, title=f"[bold {tc}] Temps [/bold {tc}]", border_style=tc, height=3)

    # ── theme picker ─────────────────────────────────────────────────────
    def _theme_picker(self) -> Layout:
        """Full-screen theme picker overlay."""
        cols = 3
        visible_rows = 18
        total = len(THEME_NAMES)
        cursor = self.theme_cursor

        # Calculate scroll to keep cursor visible
        cursor_row = cursor // cols
        if cursor_row < self.theme_scroll:
            self.theme_scroll = cursor_row
        elif cursor_row >= self.theme_scroll + visible_rows:
            self.theme_scroll = cursor_row - visible_rows + 1

        table = Table(expand=True, box=None, pad_edge=True, show_header=False)
        for _ in range(cols):
            table.add_column(ratio=1)

        total_rows = (total + cols - 1) // cols
        for row_idx in range(self.theme_scroll, min(self.theme_scroll + visible_rows, total_rows)):
            cells = []
            for col_idx in range(cols):
                i = row_idx * cols + col_idx
                if i < total:
                    name = THEME_NAMES[i]
                    th = THEMES[name]
                    # Name column
                    name_text = Text()
                    if i == cursor:
                        name_text.append(" > ", style="bold")
                        name_text.append(name, style=f"bold reverse {th['gpu']}")
                    elif name == self.theme_name:
                        name_text.append("   ", style="")
                        name_text.append(name, style=f"bold {th['gpu']}")
                        name_text.append(" *", style="dim")
                    else:
                        name_text.append("   ", style="")
                        name_text.append(name, style=f"{th['gpu']}")
                    # Swatch column — background-colored chips with gaps
                    swatch = Text()
                    swatch.append("  ", style=f"on {th['gpu']}")
                    swatch.append(" ")
                    swatch.append("  ", style=f"on {th['net_up']}")
                    swatch.append("  ", style=f"on {th['net_down']}")
                    swatch.append(" ")
                    swatch.append("  ", style=f"on {th['cpu']}")
                    swatch.append(" ")
                    swatch.append("  ", style=f"on {th['mem']}")
                    swatch.append(" ")
                    swatch.append("  ", style=f"on {th['bar_mid']}")
                    # Nested table for left name + right-aligned swatches
                    cell = Table(box=None, pad_edge=False, show_header=False, expand=True)
                    cell.add_column(ratio=1)
                    cell.add_column(justify="right")
                    cell.add_row(name_text, swatch)
                    cells.append(cell)
                else:
                    cells.append(Text(""))
            table.add_row(*cells)

        # Preview the hovered theme
        preview_name = THEME_NAMES[cursor]
        preview = THEMES[preview_name]
        sample_bar = _bar(65, 20, preview)
        preview_text = Text.from_markup(
            f"\n[bold]Preview:[/bold] {preview_name}\n"
            f"  GPU [{preview['gpu']}]{'━' * 6}[/{preview['gpu']}]  "
            f"Net [{preview['net']}]{'━' * 6}[/{preview['net']}]  "
            f"CPU [{preview['cpu']}]{'━' * 6}[/{preview['cpu']}]  "
            f"Mem [{preview['mem']}]{'━' * 6}[/{preview['mem']}]\n"
            f"  Bar: {sample_bar}"
        )

        inner = Layout()
        inner.split_column(
            Layout(name="list", ratio=8),
            Layout(name="preview", ratio=2),
        )
        inner["list"].update(table)
        inner["preview"].update(Panel(preview_text, border_style="dim"))

        hint = Text.from_markup(
            " [bold]UP/DOWN/LEFT/RIGHT[/bold] Navigate  "
            "[bold]ENTER[/bold] Select  "
            "[bold]ESC[/bold] Cancel"
        )

        outer = Layout()
        outer.split_column(
            Layout(name="body", ratio=9),
            Layout(name="hint", size=1),
        )
        outer["body"].update(
            Panel(inner, title="[bold] Select Theme [/bold]", border_style="bright_white")
        )
        outer["hint"].update(hint)
        return outer

    # ── profiling helpers ────────────────────────────────────────────────
    def _prof_time(self, label: str, fn):
        """Time a callable, accumulate result if profiling."""
        if not self._prof_log:
            return fn()
        t0 = time.perf_counter()
        result = fn()
        dt = (time.perf_counter() - t0) * 1000  # ms
        self._prof_accum.setdefault(label, []).append(dt)
        return result

    def _prof_flush(self) -> None:
        """Write accumulated profile stats to log every 5 seconds."""
        if not self._prof_log:
            return
        now = time.monotonic()
        if now - self._prof_last_flush < 5.0:
            return
        self._prof_last_flush = now
        if not self._prof_accum:
            return
        lines = [f"\n── frame {self._prof_frame} @ {datetime.now().strftime('%H:%M:%S')} ──\n"]
        lines.append(f"{'Section':<20} {'avg ms':>8} {'max ms':>8} {'calls':>6}\n")
        lines.append(f"{'─' * 20} {'─' * 8} {'─' * 8} {'─' * 6}\n")
        total_avg = 0.0
        for label, times in sorted(self._prof_accum.items()):
            avg = sum(times) / len(times)
            mx = max(times)
            total_avg += avg
            lines.append(f"{label:<20} {avg:8.2f} {mx:8.2f} {len(times):6d}\n")
        lines.append(f"{'─' * 20} {'─' * 8} {'─' * 8} {'─' * 6}\n")
        lines.append(f"{'TOTAL':<20} {total_avg:8.2f}\n")
        with open(self._prof_log, "a") as f:
            f.writelines(lines)
        self._prof_accum.clear()

    # ── main layout ──────────────────────────────────────────────────────
    def _build(self) -> Layout:
        if self.picking_theme:
            return self._theme_picker()

        self._prof_frame += 1

        layout = Layout()
        layout.split_column(
            Layout(name="gpu", ratio=2),
            Layout(name="mid", ratio=2),
            Layout(name="temps", size=3),
            Layout(name="bot", ratio=3),
            Layout(name="status", size=1),
        )
        layout["mid"].split_row(
            Layout(name="net", ratio=1),
            Layout(name="cpu", ratio=1),
            Layout(name="mem", ratio=1),
        )
        layout["bot"].split_row(
            Layout(name="mem_procs", ratio=1),
            Layout(name="cpu_procs", ratio=1),
        )

        self._prof_time("scan_procs", self._scan_procs)
        layout["gpu"].update(self._prof_time("gpu_panels", self._gpu_panels))
        layout["net"].update(self._prof_time("net_panel", self._net_panel))
        layout["cpu"].update(self._prof_time("cpu_panel", self._cpu_panel))
        layout["mem"].update(self._prof_time("mem_panel", self._mem_panel))
        layout["mem_procs"].update(self._prof_time("proc_table_mem", lambda: self._proc_table("memory_percent")))
        layout["cpu_procs"].update(self._prof_time("proc_table_cpu", lambda: self._proc_table("cpu_percent")))
        layout["temps"].update(self._prof_time("temp_strip", self._temp_strip))
        layout["status"].update(self._prof_time("status_bar", self._status_bar))

        self._prof_flush()
        return layout

    # ── input handling ───────────────────────────────────────────────────
    def _handle_key(self, key: str | None) -> bool:
        """Handle a keypress. Returns True to quit."""
        if key is None:
            return False

        if self.picking_theme:
            cols = 3
            total = len(THEME_NAMES)
            if key == "ESC":
                self.picking_theme = False
            elif key == "ENTER":
                self.theme_name = THEME_NAMES[self.theme_cursor]
                self.theme = THEMES[self.theme_name]
                self.picking_theme = False
                _save_config({"theme": self.theme_name})
            elif key == "UP":
                self.theme_cursor = max(0, self.theme_cursor - cols)
            elif key == "DOWN":
                self.theme_cursor = min(total - 1, self.theme_cursor + cols)
            elif key == "LEFT":
                self.theme_cursor = max(0, self.theme_cursor - 1)
            elif key == "RIGHT":
                self.theme_cursor = min(total - 1, self.theme_cursor + 1)
            return False

        if key in ("q", "Q", "ESC"):
            return True
        if key in ("t", "T"):
            self.picking_theme = True
            self.theme_cursor = THEME_NAMES.index(self.theme_name)
        return False

    # ── run loop ─────────────────────────────────────────────────────────
    def run(self) -> None:
        def _cleanup():
            if self.gpu_ok:
                try:
                    nvmlShutdown()
                except Exception:
                    pass

        def _quit(*_):
            _cleanup()
            sys.exit(0)

        signal.signal(signal.SIGINT, _quit)
        signal.signal(signal.SIGTERM, _quit)

        fd = sys.stdin.fileno()
        old_settings = termios.tcgetattr(fd)
        try:
            tty.setcbreak(fd)
            with Live(
                self._build(),
                console=self.console,
                screen=True,
                refresh_per_second=4,
            ) as live:
                last_refresh = time.monotonic()
                while True:
                    # Poll for keys at ~50ms intervals for responsive input
                    time.sleep(0.05)
                    key = _read_key()
                    if self._handle_key(key):
                        break
                    # Redraw immediately on keypress, or on refresh interval
                    now = time.monotonic()
                    if key or now - last_refresh >= self.refresh:
                        built = self._build()
                        self._prof_time("rich_render", lambda: live.update(built))
                        if not key:
                            last_refresh = now
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
            _cleanup()


# ── CLI ──────────────────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(description="ktop — system monitor for hybrid LLM workloads")
    parser.add_argument(
        "-r", "--refresh", type=float, default=1.0,
        help="Refresh interval in seconds (default: 1.0)",
    )
    parser.add_argument(
        "--theme", type=str, default=None,
        help=f"Color theme (see theme picker with 't' key)",
    )
    parser.add_argument(
        "--sim", action="store_true",
        help="Simulation mode (fake OOM kills for testing)",
    )
    args = parser.parse_args()

    k = KTop(refresh=args.refresh, sim=args.sim)
    if args.theme and args.theme in THEMES:
        k.theme_name = args.theme
        k.theme = THEMES[args.theme]
    k.run()


if __name__ == "__main__":
    main()
