use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use sysinfo::System;

// ── Process info (read from /proc directly for efficiency) ──

#[derive(Clone, Debug)]
pub struct ProcInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub rss: u64,
    pub shared: u64,
}

pub struct ProcScanner {
    cpu_prev: HashMap<u32, u64>,
    last_scan: Instant,
    page_size: u64,
    clock_ticks: u64,
    num_cpus: u64,
    pub by_mem: Vec<ProcInfo>,
    pub by_cpu: Vec<ProcInfo>,
}

impl ProcScanner {
    pub fn new() -> Self {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) as u64 };
        let clock_ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) as u64 };
        let num_cpus = num_cpus_count();

        let mut scanner = Self {
            cpu_prev: HashMap::new(),
            last_scan: Instant::now(),
            page_size,
            clock_ticks,
            num_cpus,
            by_mem: Vec::new(),
            by_cpu: Vec::new(),
        };

        // Seed CPU baselines
        scanner.seed_baselines();
        scanner
    }

    fn seed_baselines(&mut self) {
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if let Ok(pid) = name_str.parse::<u32>() {
                    if let Some(cpu_total) = read_proc_cpu_total(pid) {
                        self.cpu_prev.insert(pid, cpu_total);
                    }
                }
            }
        }
    }

    pub fn scan(&mut self, total_mem: u64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_scan).as_secs_f64();
        let min_wait = if self.by_mem.is_empty() { 1.0 } else { 3.0 };
        if elapsed < min_wait {
            return;
        }
        let dt = elapsed;
        self.last_scan = now;

        let ps = self.page_size;
        let ct = self.clock_ticks as f64;
        let mut procs = Vec::new();

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                let pid = match name_str.parse::<u32>() {
                    Ok(p) if p > 0 => p,
                    _ => continue,
                };

                let stat_path = format!("/proc/{}/stat", pid);
                let stat = match read_file_bytes(&stat_path) {
                    Some(s) => s,
                    None => continue,
                };

                // Parse stat: find last ')' for comm field
                let rparen = match stat.iter().rposition(|&b| b == b')') {
                    Some(i) => i,
                    None => continue,
                };

                let fields_bytes = &stat[rparen + 2..];
                let fields: Vec<&[u8]> = fields_bytes.splitn(23, |&b| b == b' ').collect();
                if fields.len() < 22 {
                    continue;
                }

                let utime = parse_u64(fields[11]);
                let stime = parse_u64(fields[12]);
                let rss = parse_u64(fields[21]) * ps;

                // Extract name
                let lparen = stat.iter().position(|&b| b == b'(').unwrap_or(0);
                let proc_name = String::from_utf8_lossy(&stat[lparen + 1..rparen])
                    .chars()
                    .take(28)
                    .collect::<String>();

                let mem_pct = if total_mem > 0 {
                    rss as f64 / total_mem as f64 * 100.0
                } else {
                    0.0
                };

                let cpu_total = utime + stime;
                let prev = self.cpu_prev.get(&pid).copied().unwrap_or(cpu_total);
                let cpu_delta = cpu_total.saturating_sub(prev);
                self.cpu_prev.insert(pid, cpu_total);

                let cpu_pct = if dt > 0.0 {
                    (cpu_delta as f64 / ct) / dt * 100.0
                } else {
                    0.0
                };

                procs.push(ProcInfo {
                    pid,
                    name: proc_name,
                    cpu_percent: cpu_pct,
                    memory_percent: mem_pct,
                    rss,
                    shared: 0,
                });
            }
        }

        // Sort and take top 10 by each metric
        let mut by_mem = procs.clone();
        by_mem.sort_by(|a, b| b.memory_percent.partial_cmp(&a.memory_percent).unwrap_or(std::cmp::Ordering::Equal));
        by_mem.truncate(10);

        let mut by_cpu = procs.clone();
        by_cpu.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
        by_cpu.truncate(10);

        // Read shared memory for displayed procs
        let displayed: std::collections::HashSet<u32> = by_mem
            .iter()
            .chain(by_cpu.iter())
            .map(|p| p.pid)
            .collect();

        for p in by_mem.iter_mut().chain(by_cpu.iter_mut()) {
            if displayed.contains(&p.pid) {
                p.shared = read_proc_shared(p.pid, self.page_size);
            }
        }

        // Clean stale PIDs
        let current: std::collections::HashSet<u32> = procs.iter().map(|p| p.pid).collect();
        self.cpu_prev.retain(|k, _| current.contains(k));

        self.by_mem = by_mem;
        self.by_cpu = by_cpu;
    }

    pub fn num_cpus(&self) -> u64 {
        self.num_cpus
    }
}

fn read_file_bytes(path: &str) -> Option<Vec<u8>> {
    let mut f = fs::File::open(path).ok()?;
    let mut buf = Vec::with_capacity(512);
    f.read_to_end(&mut buf).ok()?;
    Some(buf)
}

fn parse_u64(bytes: &[u8]) -> u64 {
    let s = std::str::from_utf8(bytes).unwrap_or("0");
    s.parse().unwrap_or(0)
}

fn read_proc_cpu_total(pid: u32) -> Option<u64> {
    let stat = read_file_bytes(&format!("/proc/{}/stat", pid))?;
    let rparen = stat.iter().rposition(|&b| b == b')')?;
    let fields: Vec<&[u8]> = stat[rparen + 2..].splitn(14, |&b| b == b' ').collect();
    if fields.len() < 13 {
        return None;
    }
    Some(parse_u64(fields[11]) + parse_u64(fields[12]))
}

fn read_proc_shared(pid: u32, page_size: u64) -> u64 {
    let path = format!("/proc/{}/statm", pid);
    if let Some(data) = read_file_bytes(&path) {
        let fields: Vec<&[u8]> = data.splitn(4, |&b| b == b' ').collect();
        if fields.len() >= 3 {
            return parse_u64(fields[2]) * page_size;
        }
    }
    0
}

fn num_cpus_count() -> u64 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u64)
        .unwrap_or(1)
}

// ── OOM kill detection ──

pub struct OomTracker {
    last_check: Instant,
    pub last_oom: Option<String>,
    sim: bool,
}

static SIM_PROCS: &[&str] = &[
    "python3", "node", "java", "ollama", "vllm", "ffmpeg", "cc1plus", "rustc", "chrome", "mysqld",
];

// Regex-like UUID pattern for stripping from oomd kill names
fn strip_uuids(s: &str) -> String {
    // Match -XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX patterns
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '-' && i + 36 <= chars.len() {
            // Check if this is a UUID pattern: -8-4-4-4-12 hex digits
            let candidate: String = chars[i..i + 37.min(chars.len())].iter().collect();
            if candidate.len() >= 37 {
                let parts: Vec<&str> = candidate[1..].splitn(6, '-').collect();
                if parts.len() == 5
                    && parts[0].len() == 8
                    && parts[1].len() == 4
                    && parts[2].len() == 4
                    && parts[3].len() == 4
                    && parts[4].len() == 12
                    && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
                {
                    i += 37;
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

impl OomTracker {
    pub fn new(sim: bool) -> Self {
        Self {
            last_check: Instant::now() - std::time::Duration::from_secs(10),
            last_oom: None,
            sim,
        }
    }

    pub fn check(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_check).as_secs_f64() < 5.0 {
            return;
        }
        self.last_check = now;

        if self.sim {
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            if seed % 2 == 0 {
                self.last_oom = None;
            } else {
                let proc_idx = (seed / 3) as usize % SIM_PROCS.len();
                let fake_offset = (seed % (8 * 3600)) as u64;
                let fake_epoch = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    - fake_offset;
                let dt = chrono_format_full(fake_epoch as f64);
                self.last_oom = Some(format!("{} {}", dt, SIM_PROCS[proc_idx]));
            }
            return;
        }

        let mut candidates: Vec<(f64, String)> = Vec::new();

        // Kernel OOM kills
        if let Ok(output) = Command::new("journalctl")
            .args([
                "-k",
                "--since",
                "8 hours ago",
                "--no-pager",
                "-o",
                "short-unix",
                "--grep",
                "Killed process",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.trim().lines().last() {
                    if let Some((epoch, proc_name)) = parse_oom_line(line) {
                        candidates.push((epoch, proc_name));
                    }
                }
            }
        }

        // systemd-oomd kills
        if let Ok(output) = Command::new("journalctl")
            .args([
                "-u",
                "systemd-oomd",
                "--since",
                "8 hours ago",
                "--no-pager",
                "-o",
                "short-unix",
                "--grep",
                "Killed",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.trim().lines().last() {
                    if let Some((epoch, proc_name)) = parse_oomd_line(line) {
                        candidates.push((epoch, proc_name));
                    }
                }
            }
        }

        if candidates.is_empty() {
            self.last_oom = None;
        } else {
            // Pick the most recent by timestamp
            candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            let (epoch, proc_name) = &candidates[0];
            let dt = chrono_format_full(*epoch);
            self.last_oom = Some(format!("{} {}", dt, proc_name));
        }
    }
}

fn parse_oom_line(line: &str) -> Option<(f64, String)> {
    // Format: "1234567890.123 hostname kernel: Killed process 1234 (name)"
    let proc_start = line.find("Killed process ")?;
    let rest = &line[proc_start..];
    let lparen = rest.find('(')?;
    let rparen = rest.find(')')?;
    if rparen > lparen {
        let ts = line.split_whitespace().next()?;
        let epoch: f64 = ts.parse().ok()?;
        let name = rest[lparen + 1..rparen].to_string();
        Some((epoch, name))
    } else {
        None
    }
}

fn parse_oomd_line(line: &str) -> Option<(f64, String)> {
    let ts = line.split_whitespace().next()?;
    let epoch: f64 = ts.parse().ok()?;
    // "Killed /some/path/unit.scope due to memory"
    if let Some(start) = line.find("Killed ") {
        let rest = &line[start + 7..];
        if let Some(due) = rest.find(" due to") {
            let path = &rest[..due];
            let name = path.rsplit('/').next().unwrap_or("oomd-kill");
            let name = name
                .replace(".scope", "")
                .replace(".service", "");
            let name = strip_uuids(&name);
            return Some((epoch, name));
        }
    }
    Some((epoch, "oomd-kill".to_string()))
}

fn chrono_format_full(epoch: f64) -> String {
    // Convert epoch to local time with month, day, and time
    // Using libc localtime for proper timezone handling
    let secs = epoch as i64;
    let tm = unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&secs, &mut tm);
        tm
    };
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                   "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let month = months.get(tm.tm_mon as usize).unwrap_or(&"???");
    format!("{} {:02} {:02}:{:02}:{:02}",
            month, tm.tm_mday, tm.tm_hour, tm.tm_min, tm.tm_sec)
}

// ── Available/Cached from /proc/meminfo ──

pub fn read_available_cached() -> (u64, u64) {
    let mut available: u64 = 0;
    let mut cached: u64 = 0;
    if let Ok(contents) = fs::read_to_string("/proc/meminfo") {
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("MemAvailable:") {
                if let Ok(kb) = rest.trim().trim_end_matches(" kB").trim().parse::<u64>() {
                    available = kb * 1024;
                }
            } else if let Some(rest) = line.strip_prefix("Cached:") {
                if let Ok(kb) = rest.trim().trim_end_matches(" kB").trim().parse::<u64>() {
                    cached = kb * 1024;
                }
            }
        }
    }
    (available, cached)
}

// ── CPU frequency from sysfs ──

pub fn read_cpu_freq() -> Option<String> {
    if let Ok(freq_str) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
    {
        if let Ok(khz) = freq_str.trim().parse::<u64>() {
            return Some(format!("{} MHz", khz / 1000));
        }
    }
    None
}

// ── Power estimation from Linux sensors ──

struct RaplZone {
    energy_uj_path: PathBuf,
    max_energy_uj: u64,
    last_energy_uj: Option<u64>,
}

pub struct PowerEstimator {
    rapl_zones: Vec<RaplZone>,
    hwmon_cpu_power_paths: Vec<PathBuf>,
    last_sample: Instant,
    pub cpu_watts: Option<f64>,
}

impl PowerEstimator {
    pub fn new() -> Self {
        Self {
            rapl_zones: detect_rapl_zones(),
            hwmon_cpu_power_paths: detect_hwmon_cpu_power_paths(),
            last_sample: Instant::now(),
            cpu_watts: None,
        }
    }

    pub fn sample_cpu_watts(&mut self) -> Option<f64> {
        let now = Instant::now();
        let dt = now.duration_since(self.last_sample).as_secs_f64();
        self.last_sample = now;

        let rapl = if dt > 0.0 {
            sample_rapl_cpu_watts(&mut self.rapl_zones, dt)
        } else {
            None
        };
        let hwmon = sample_hwmon_cpu_watts(&self.hwmon_cpu_power_paths);

        self.cpu_watts = rapl.or(hwmon);
        self.cpu_watts
    }
}

fn detect_rapl_zones() -> Vec<RaplZone> {
    let mut zones = Vec::new();
    let base = PathBuf::from("/sys/class/powercap");
    let entries = match fs::read_dir(&base) {
        Ok(entries) => entries,
        Err(_) => return zones,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("intel-rapl:") {
            continue;
        }
        if name.matches(':').count() != 1 {
            continue;
        }

        let zone_name = fs::read_to_string(path.join("name"))
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if zone_name.contains("psys") {
            continue;
        }

        let energy_uj_path = path.join("energy_uj");
        let max_energy_path = path.join("max_energy_range_uj");
        if !energy_uj_path.is_file() || !max_energy_path.is_file() {
            continue;
        }
        let max_energy_uj = fs::read_to_string(max_energy_path)
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(0);
        if max_energy_uj == 0 {
            continue;
        }

        zones.push(RaplZone {
            energy_uj_path,
            max_energy_uj,
            last_energy_uj: None,
        });
    }

    zones
}

fn sample_rapl_cpu_watts(zones: &mut [RaplZone], dt: f64) -> Option<f64> {
    let mut total_watts = 0.0;
    let mut found = false;

    for zone in zones {
        let energy_uj = match fs::read_to_string(&zone.energy_uj_path)
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
        {
            Some(value) => value,
            None => continue,
        };

        if let Some(prev) = zone.last_energy_uj {
            let delta_uj = if energy_uj >= prev {
                energy_uj - prev
            } else {
                (zone.max_energy_uj.saturating_sub(prev)) + energy_uj
            };
            total_watts += delta_uj as f64 / 1_000_000.0 / dt;
            found = true;
        }

        zone.last_energy_uj = Some(energy_uj);
    }

    found.then_some(total_watts)
}

fn detect_hwmon_cpu_power_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let entries = match fs::read_dir("/sys/class/hwmon") {
        Ok(entries) => entries,
        Err(_) => return paths,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = fs::read_to_string(path.join("name"))
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !matches!(name.as_str(), "zenpower" | "k10temp" | "coretemp") {
            continue;
        }

        let power1_average = path.join("power1_average");
        let power1_input = path.join("power1_input");
        if power1_average.is_file() {
            paths.push(power1_average);
        } else if power1_input.is_file() {
            paths.push(power1_input);
        }
    }

    paths
}

fn sample_hwmon_cpu_watts(paths: &[PathBuf]) -> Option<f64> {
    let mut total = 0.0;
    let mut found = false;
    for path in paths {
        let value = match fs::read_to_string(path)
            .ok()
            .and_then(|v| v.trim().parse::<f64>().ok())
        {
            Some(value) => value,
            None => continue,
        };
        total += value / 1_000_000.0;
        found = true;
    }
    found.then_some(total)
}

// ── Network speeds ──

pub struct NetTracker {
    last_sent: u64,
    last_recv: u64,
    last_time: Instant,
    pub max_speed: f64,
}

impl NetTracker {
    pub fn new(_sys: &System) -> Self {
        let networks = sysinfo::Networks::new_with_refreshed_list();
        let (sent, recv) = net_totals(&networks);
        Self {
            last_sent: sent,
            last_recv: recv,
            last_time: Instant::now(),
            max_speed: 1.0,
        }
    }

    pub fn sample(&mut self, networks: &sysinfo::Networks) -> (f64, f64) {
        let (sent, recv) = net_totals(networks);
        let now = Instant::now();
        let dt = now.duration_since(self.last_time).as_secs_f64();
        let dt = if dt <= 0.0 { 1.0 } else { dt };

        let up = (sent.saturating_sub(self.last_sent)) as f64 / dt;
        let down = (recv.saturating_sub(self.last_recv)) as f64 / dt;

        self.last_sent = sent;
        self.last_recv = recv;
        self.last_time = now;
        self.max_speed = self.max_speed.max(up).max(down).max(1.0);

        (up, down)
    }
}

fn net_totals(networks: &sysinfo::Networks) -> (u64, u64) {
    let mut sent = 0u64;
    let mut recv = 0u64;
    for (_name, data) in networks.iter() {
        sent += data.total_transmitted();
        recv += data.total_received();
    }
    (sent, recv)
}
