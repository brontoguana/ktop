use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub id: usize,
    pub name: String,
    pub util: f64,
    pub mem_used_gb: f64,
    pub mem_total_gb: f64,
    pub mem_pct: f64,
}

#[derive(Clone, Debug)]
pub struct GpuTemp {
    pub temp: f64,
    pub max: f64,
}

// ── NVIDIA via NVML ──

pub struct NvidiaState {
    nvml: nvml_wrapper::Nvml,
    pub count: usize,
}

impl NvidiaState {
    pub fn init() -> Option<Self> {
        // Try default path first, then versioned .so.1 (common on Linux where
        // the unversioned symlink is only in -dev packages)
        let nvml = nvml_wrapper::Nvml::init()
            .or_else(|_| {
                nvml_wrapper::Nvml::builder()
                    .lib_path(OsStr::new("libnvidia-ml.so.1"))
                    .init()
            })
            .ok()?;
        let count = nvml.device_count().ok()? as usize;
        if count == 0 {
            return None;
        }
        Some(Self { nvml, count })
    }

    pub fn sample(&self, base_id: usize) -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        for i in 0..self.count {
            if let Ok(dev) = self.nvml.device_by_index(i as u32) {
                let name = dev.name().unwrap_or_else(|_| format!("GPU {}", i));
                let util = dev
                    .utilization_rates()
                    .map(|u| u.gpu as f64)
                    .unwrap_or(0.0);
                let (mem_used, mem_total) = dev
                    .memory_info()
                    .map(|m| (m.used as f64, m.total as f64))
                    .unwrap_or((0.0, 1.0));
                let mem_pct = if mem_total > 0.0 {
                    mem_used / mem_total * 100.0
                } else {
                    0.0
                };
                gpus.push(GpuInfo {
                    id: base_id + i,
                    name,
                    util,
                    mem_used_gb: mem_used / (1024.0 * 1024.0 * 1024.0),
                    mem_total_gb: mem_total / (1024.0 * 1024.0 * 1024.0),
                    mem_pct,
                });
            }
        }
        gpus
    }

    pub fn power_watts(&self) -> Vec<Option<f64>> {
        let mut powers = Vec::new();
        for i in 0..self.count {
            let power = self
                .nvml
                .device_by_index(i as u32)
                .ok()
                .and_then(|dev| dev.power_usage().ok())
                .map(|mw| mw as f64 / 1000.0);
            powers.push(power);
        }
        powers
    }

    pub fn temps(&self) -> Vec<Option<GpuTemp>> {
        let mut temps = Vec::new();
        for i in 0..self.count {
            if let Ok(dev) = self.nvml.device_by_index(i as u32) {
                let temp = dev
                    .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                    .ok()
                    .map(|t| t as f64);
                let max = dev
                    .temperature_threshold(
                        nvml_wrapper::enum_wrappers::device::TemperatureThreshold::Slowdown,
                    )
                    .ok()
                    .map(|t| t as f64)
                    .unwrap_or(95.0);
                temps.push(temp.map(|t| GpuTemp { temp: t, max }));
            } else {
                temps.push(None);
            }
        }
        temps
    }
}

// ── AMD via sysfs ──

#[derive(Clone, Debug)]
pub struct AmdCard {
    pub name: String,
    pub util_path: PathBuf,
    pub has_util: bool,
    pub vram_used_path: PathBuf,
    pub vram_total_bytes: u64,
    pub has_vram: bool,
    pub temp_path: Option<PathBuf>,
    pub temp_crit_path: Option<PathBuf>,
    pub power_path: Option<PathBuf>,
    pub power_scale: f64,
}

pub fn detect_amd_gpus() -> Vec<AmdCard> {
    let mut cards = Vec::new();
    let mut seen_devices: HashSet<PathBuf> = HashSet::new();

    let pattern = "/sys/class/drm/card*/device/vendor";
    let entries: Vec<PathBuf> = glob_paths(pattern);

    for vendor_path in entries {
        let vendor = match fs::read_to_string(&vendor_path) {
            Ok(v) => v.trim().to_string(),
            Err(_) => continue,
        };
        if vendor != "0x1002" {
            continue;
        }

        let dev_dir = vendor_path.parent().unwrap().to_path_buf();
        let real_dev = fs::canonicalize(&dev_dir).unwrap_or_else(|_| dev_dir.clone());
        if seen_devices.contains(&real_dev) {
            continue;
        }
        seen_devices.insert(real_dev);

        // GPU name
        let mut name = "AMD GPU".to_string();
        for name_file in &["product_name", "product_description"] {
            if let Ok(n) = fs::read_to_string(dev_dir.join(name_file)) {
                let n = n.trim().to_string();
                if !n.is_empty() {
                    name = n;
                    break;
                }
            }
        }
        if name == "AMD GPU" {
            if let Ok(d) = fs::read_to_string(dev_dir.join("device")) {
                name = format!("AMD GPU [{}]", d.trim());
            }
        }

        let util_path = dev_dir.join("gpu_busy_percent");
        let has_util = util_path.is_file();

        let vram_total_path = dev_dir.join("mem_info_vram_total");
        let vram_used_path = dev_dir.join("mem_info_vram_used");
        let mut has_vram = vram_total_path.is_file() && vram_used_path.is_file();
        let mut vram_total_bytes = 0u64;
        if has_vram {
            match fs::read_to_string(&vram_total_path) {
                Ok(v) => match v.trim().parse::<u64>() {
                    Ok(val) => vram_total_bytes = val,
                    Err(_) => has_vram = false,
                },
                Err(_) => has_vram = false,
            }
        }

        // hwmon temp paths
        let mut temp_path = None;
        let mut temp_crit_path = None;
        let mut power_path = None;
        let mut power_scale = 1_000_000.0;
        let hwmon_dir = dev_dir.join("hwmon");
        if hwmon_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&hwmon_dir) {
                let mut hwmons: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                hwmons.sort_by_key(|e| e.file_name());
                if let Some(hw) = hwmons.first() {
                    let t1 = hw.path().join("temp1_input");
                    if t1.is_file() {
                        temp_path = Some(t1);
                    }
                    let tc = hw.path().join("temp1_crit");
                    if tc.is_file() {
                        temp_crit_path = Some(tc);
                    }
                    let p_avg = hw.path().join("power1_average");
                    let p_input = hw.path().join("power1_input");
                    if p_avg.is_file() {
                        power_path = Some(p_avg);
                    } else if p_input.is_file() {
                        power_path = Some(p_input);
                    }
                    let p_cap = hw.path().join("power1_cap");
                    if !p_cap.is_file() {
                        power_scale = 1_000_000.0;
                    }
                }
            }
        }

        cards.push(AmdCard {
            name,
            util_path,
            has_util,
            vram_used_path,
            vram_total_bytes,
            has_vram,
            temp_path,
            temp_crit_path,
            power_path,
            power_scale,
        });
    }
    cards
}

impl AmdCard {
    pub fn sample(&self, id: usize) -> GpuInfo {
        let util = if self.has_util {
            fs::read_to_string(&self.util_path)
                .ok()
                .and_then(|v| v.trim().parse::<f64>().ok())
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let mem_used = if self.has_vram {
            fs::read_to_string(&self.vram_used_path)
                .ok()
                .and_then(|v| v.trim().parse::<f64>().ok())
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let mem_total = self.vram_total_bytes as f64;
        let mem_pct = if mem_total > 0.0 {
            mem_used / mem_total * 100.0
        } else {
            0.0
        };

        GpuInfo {
            id,
            name: self.name.clone(),
            util,
            mem_used_gb: mem_used / (1024.0 * 1024.0 * 1024.0),
            mem_total_gb: mem_total / (1024.0 * 1024.0 * 1024.0),
            mem_pct,
        }
    }

    pub fn temp(&self) -> Option<GpuTemp> {
        let temp_path = self.temp_path.as_ref()?;
        let t = fs::read_to_string(temp_path)
            .ok()?
            .trim()
            .parse::<f64>()
            .ok()?
            / 1000.0;
        let max = self
            .temp_crit_path
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|v| v.trim().parse::<f64>().ok())
            .map(|v| v / 1000.0)
            .unwrap_or(95.0);
        Some(GpuTemp { temp: t, max })
    }

    pub fn power_watts(&self) -> Option<f64> {
        let path = self.power_path.as_ref()?;
        let raw = fs::read_to_string(path).ok()?;
        let value = raw.trim().parse::<f64>().ok()?;
        Some(value / self.power_scale)
    }
}

fn glob_paths(pattern: &str) -> Vec<PathBuf> {
    // Simple glob for /sys/class/drm/card*/device/vendor
    let prefix = "/sys/class/drm/";
    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(prefix) {
        let mut dirs: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.starts_with("card"))
                    .unwrap_or(false)
            })
            .collect();
        dirs.sort_by_key(|e| e.file_name());
        for d in dirs {
            let p = d.path().join("device/vendor");
            if p.is_file() {
                results.push(p);
            }
        }
    }
    let _ = pattern; // suppress unused
    results
}
