use crate::config;
use crate::gpu::{self, GpuInfo, GpuTemp, NvidiaState};
use crate::system::{self, NetTracker, OomTracker, ProcInfo, ProcScanner};
use crate::theme::{self, Theme};
use crate::ui;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::{HashMap, VecDeque};
use std::io;
use std::time::{Duration, Instant};

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};

const HISTORY_LEN: usize = 300;

pub struct AppState {
    // Theme
    pub theme_name: String,
    pub theme: Theme,
    pub picking_theme: bool,
    pub theme_cursor: usize,
    pub theme_scroll: usize,

    // CPU
    pub cpu_pct: f64,
    pub cpu_total_pct: f64,
    pub cpu_hist: VecDeque<f64>,
    pub cpu_cores: usize,
    pub cpu_freq: String,

    // Memory
    pub ram_pct: f64,
    pub ram_used: u64,
    pub ram_total: u64,
    pub ram_available: u64,
    pub ram_cached: u64,
    pub swap_pct: f64,
    pub swap_used: u64,
    pub swap_total: u64,

    // GPU
    pub gpu_infos: Vec<GpuInfo>,
    pub gpu_util_hist: HashMap<usize, VecDeque<f64>>,
    pub gpu_mem_hist: HashMap<usize, VecDeque<f64>>,
    pub gpu_power_watts: HashMap<usize, f64>,
    pub gpu_power_limits: HashMap<usize, f64>,

    // Network
    pub net_up: f64,
    pub net_down: f64,
    pub net_up_hist: VecDeque<f64>,
    pub net_down_hist: VecDeque<f64>,
    pub net_max_speed: f64,

    // Temps
    pub cpu_temp: Option<f64>,
    pub cpu_temp_max: f64,
    pub mem_temp: Option<f64>,
    pub mem_temp_max: f64,
    pub gpu_temps: Vec<Option<GpuTemp>>,

    // Processes
    pub procs_by_mem: Vec<ProcInfo>,
    pub procs_by_cpu: Vec<ProcInfo>,
    pub num_cpus: u64,

    // OOM
    pub oom_str: Option<String>,
    pub est_power_watts: Option<f64>,
}

pub fn run(
    refresh: f64,
    sim: bool,
    theme_override: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Init system
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::everything()),
    );
    let mut networks = Networks::new_with_refreshed_list();

    // First CPU sample (baseline)
    sys.refresh_cpu_usage();
    std::thread::sleep(Duration::from_millis(100));

    // Init GPU
    let nvidia = NvidiaState::init();
    let nvidia_count = nvidia.as_ref().map(|n| n.count).unwrap_or(0);
    let amd_cards = gpu::detect_amd_gpus();
    let gpu_count = nvidia_count + amd_cards.len();

    // Init histories
    let mut gpu_util_hist: HashMap<usize, VecDeque<f64>> = HashMap::new();
    let mut gpu_mem_hist: HashMap<usize, VecDeque<f64>> = HashMap::new();
    for i in 0..gpu_count {
        gpu_util_hist.insert(i, VecDeque::with_capacity(HISTORY_LEN));
        gpu_mem_hist.insert(i, VecDeque::with_capacity(HISTORY_LEN));
    }

    // Load theme
    let cfg = config::load_config();
    let theme_name = theme_override
        .or(cfg.theme)
        .unwrap_or_else(|| "Vaporwave".to_string());
    let theme_name = if theme::theme_names().contains(&theme_name.as_str()) {
        theme_name
    } else {
        "Default".to_string()
    };

    let names = theme::theme_names();
    let theme_cursor = names
        .iter()
        .position(|&n| n == theme_name)
        .unwrap_or(0);

    // Init other subsystems
    let mut net_tracker = NetTracker::new(&sys);
    let mut proc_scanner = ProcScanner::new();
    let mut oom_tracker = OomTracker::new(sim);
    let mut power_estimator = system::PowerEstimator::new();
    let mut last_freq_check = Instant::now() - Duration::from_secs(10);

    // Terminal setup
    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut state = AppState {
        theme_name: theme_name.clone(),
        theme: theme::get_theme(&theme_name),
        picking_theme: false,
        theme_cursor,
        theme_scroll: 0,
        cpu_pct: 0.0,
        cpu_total_pct: 0.0,
        cpu_hist: VecDeque::with_capacity(HISTORY_LEN),
        cpu_cores: sys.cpus().len(),
        cpu_freq: system::read_cpu_freq().unwrap_or_else(|| "N/A".to_string()),
        ram_pct: 0.0,
        ram_used: 0,
        ram_total: 0,
        ram_available: 0,
        ram_cached: 0,
        swap_pct: 0.0,
        swap_used: 0,
        swap_total: 0,
        gpu_infos: Vec::new(),
        gpu_util_hist,
        gpu_mem_hist,
        gpu_power_watts: HashMap::new(),
        gpu_power_limits: HashMap::new(),
        net_up: 0.0,
        net_down: 0.0,
        net_up_hist: VecDeque::with_capacity(HISTORY_LEN),
        net_down_hist: VecDeque::with_capacity(HISTORY_LEN),
        net_max_speed: 1.0,
        cpu_temp: None,
        cpu_temp_max: 100.0,
        mem_temp: None,
        mem_temp_max: 85.0,
        gpu_temps: Vec::new(),
        procs_by_mem: Vec::new(),
        procs_by_cpu: Vec::new(),
        num_cpus: proc_scanner.num_cpus(),
        oom_str: None,
        est_power_watts: None,
    };

    let refresh_dur = Duration::from_secs_f64(refresh);
    let mut last_refresh = Instant::now() - refresh_dur;

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Handle events with ~50ms poll timeout
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if handle_key(key, &mut state) {
                        break;
                    }
                    // Immediate redraw on keypress
                    terminal.draw(|f| ui::render(f, &state))?;
                    continue;
                }
            }

            let now = Instant::now();
            if now.duration_since(last_refresh) >= refresh_dur {
                last_refresh = now;

                // Sample CPU
                sys.refresh_cpu_usage();
                state.cpu_pct = sys.global_cpu_usage() as f64;
                state.cpu_total_pct = sys.cpus().iter().map(|c| c.cpu_usage() as f64).sum();
                state.cpu_hist.push_back(state.cpu_pct);
                if state.cpu_hist.len() > HISTORY_LEN {
                    state.cpu_hist.pop_front();
                }

                // CPU frequency (every 5s)
                if now.duration_since(last_freq_check) >= Duration::from_secs(5) {
                    last_freq_check = now;
                    state.cpu_freq =
                        system::read_cpu_freq().unwrap_or_else(|| state.cpu_freq.clone());
                }

                // Memory
                sys.refresh_memory();
                state.ram_total = sys.total_memory();
                state.ram_used = sys.used_memory();
                state.ram_pct = if state.ram_total > 0 {
                    state.ram_used as f64 / state.ram_total as f64 * 100.0
                } else {
                    0.0
                };
                let (avail, cached) = system::read_available_cached();
                state.ram_available = avail;
                state.ram_cached = cached;
                state.swap_total = sys.total_swap();
                state.swap_used = sys.used_swap();
                state.swap_pct = if state.swap_total > 0 {
                    state.swap_used as f64 / state.swap_total as f64 * 100.0
                } else {
                    0.0
                };

                // GPUs
                let mut gpu_infos = Vec::new();
                if let Some(ref nv) = nvidia {
                    gpu_infos.extend(nv.sample(0));
                }
                for (j, card) in amd_cards.iter().enumerate() {
                    gpu_infos.push(card.sample(nvidia_count + j));
                }
                for g in &gpu_infos {
                    if let Some(hist) = state.gpu_util_hist.get_mut(&g.id) {
                        hist.push_back(g.util);
                        if hist.len() > HISTORY_LEN {
                            hist.pop_front();
                        }
                    }
                    if let Some(hist) = state.gpu_mem_hist.get_mut(&g.id) {
                        hist.push_back(g.mem_pct);
                        if hist.len() > HISTORY_LEN {
                            hist.pop_front();
                        }
                    }
                }
                state.gpu_infos = gpu_infos;

                // GPU temps
                let mut gpu_temps = Vec::new();
                if let Some(ref nv) = nvidia {
                    gpu_temps.extend(nv.temps());
                }
                for card in &amd_cards {
                    gpu_temps.push(card.temp());
                }
                state.gpu_temps = gpu_temps;

                // GPU power (per-GPU)
                let mut gpu_power_watts = HashMap::new();
                let mut gpu_power_limits = HashMap::new();
                if let Some(ref nv) = nvidia {
                    for (i, (power, limit)) in nv.power_with_limits().into_iter().enumerate() {
                        if let Some(p) = power {
                            gpu_power_watts.insert(i, p);
                        }
                        gpu_power_limits.insert(i, limit);
                    }
                }
                for (j, card) in amd_cards.iter().enumerate() {
                    let idx = nvidia_count + j;
                    if let Some((power, limit)) = card.power_with_limit() {
                        gpu_power_watts.insert(idx, power);
                        gpu_power_limits.insert(idx, limit);
                    }
                }
                state.gpu_power_watts = gpu_power_watts;
                state.gpu_power_limits = gpu_power_limits;

                let cpu_power = power_estimator.sample_cpu_watts();
                let nvidia_power = state.gpu_power_watts.values().filter(|&&v| v > 0.0).sum::<f64>();
                let total_power = cpu_power.unwrap_or(0.0) + nvidia_power;
                state.est_power_watts = if cpu_power.is_some() || nvidia_power > 0.0 {
                    Some(total_power)
                } else {
                    None
                };

                // CPU temps via sysinfo
                sample_temps(&sys, &mut state);

                // Network
                networks.refresh(true);
                let (up, down) = net_tracker.sample(&networks);
                state.net_up = up;
                state.net_down = down;
                state.net_max_speed = net_tracker.max_speed;
                state.net_up_hist.push_back(up);
                state.net_down_hist.push_back(down);
                if state.net_up_hist.len() > HISTORY_LEN {
                    state.net_up_hist.pop_front();
                }
                if state.net_down_hist.len() > HISTORY_LEN {
                    state.net_down_hist.pop_front();
                }

                // Processes
                proc_scanner.scan(state.ram_total);
                state.procs_by_mem = proc_scanner.by_mem.clone();
                state.procs_by_cpu = proc_scanner.by_cpu.clone();

                // OOM
                oom_tracker.check();
                state.oom_str = oom_tracker.last_oom.clone();

                terminal.draw(|f| ui::render(f, &state))?;
            }
        }
        Ok(())
    })();

    // Cleanup
    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}

fn handle_key(key: KeyEvent, state: &mut AppState) -> bool {
    let names = theme::theme_names();
    let total = names.len();
    let cols = 3;

    if state.picking_theme {
        match key.code {
            KeyCode::Esc => state.picking_theme = false,
            KeyCode::Enter => {
                if let Some(&name) = names.get(state.theme_cursor) {
                    state.theme_name = name.to_string();
                    state.theme = theme::get_theme(name);
                    state.picking_theme = false;
                    config::save_config(&config::Config {
                        theme: Some(name.to_string()),
                    });
                }
            }
            KeyCode::Up => {
                state.theme_cursor = state.theme_cursor.saturating_sub(cols);
            }
            KeyCode::Down => {
                state.theme_cursor = (state.theme_cursor + cols).min(total - 1);
            }
            KeyCode::Left => {
                state.theme_cursor = state.theme_cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                state.theme_cursor = (state.theme_cursor + 1).min(total - 1);
            }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,
        KeyCode::Esc => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Char('t') | KeyCode::Char('T') => {
            state.picking_theme = true;
            state.theme_cursor = names
                .iter()
                .position(|&n| n == state.theme_name)
                .unwrap_or(0);
        }
        _ => {}
    }
    false
}

fn sample_temps(_sys: &System, state: &mut AppState) {
    // Read from sysfs directly for CPU and memory temps
    state.cpu_temp = None;
    state.cpu_temp_max = 100.0;
    state.mem_temp = None;
    state.mem_temp_max = 85.0;

    if let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = std::fs::read_to_string(path.join("name"))
                .unwrap_or_default()
                .trim()
                .to_string();

            // CPU temps
            if matches!(
                name.as_str(),
                "coretemp" | "k10temp" | "cpu_thermal" | "zenpower" | "acpitz"
            ) {
                for i in 1..=16 {
                    let temp_path = path.join(format!("temp{}_input", i));
                    if let Ok(val) = std::fs::read_to_string(&temp_path) {
                        if let Ok(millideg) = val.trim().parse::<f64>() {
                            let temp = millideg / 1000.0;
                            state.cpu_temp = Some(
                                state.cpu_temp.map(|t: f64| t.max(temp)).unwrap_or(temp),
                            );
                        }
                    }
                    // Check critical then high temp
                    let crit_path = path.join(format!("temp{}_crit", i));
                    if let Ok(val) = std::fs::read_to_string(&crit_path) {
                        if let Ok(millideg) = val.trim().parse::<f64>() {
                            let crit = millideg / 1000.0;
                            if crit > state.cpu_temp_max {
                                state.cpu_temp_max = crit;
                            }
                        }
                    } else {
                        let high_path = path.join(format!("temp{}_max", i));
                        if let Ok(val) = std::fs::read_to_string(&high_path) {
                            if let Ok(millideg) = val.trim().parse::<f64>() {
                                let high = millideg / 1000.0;
                                if high > state.cpu_temp_max {
                                    state.cpu_temp_max = high;
                                }
                            }
                        }
                    }
                }
            }

            // Memory temps (SODIMM, dimm, memory)
            if name == "SODIMM" || name == "dimm" || name == "memory" {
                for i in 1..=4 {
                    let temp_path = path.join(format!("temp{}_input", i));
                    if let Ok(val) = std::fs::read_to_string(&temp_path) {
                        if let Ok(millideg) = val.trim().parse::<f64>() {
                            let temp = millideg / 1000.0;
                            state.mem_temp = Some(
                                state.mem_temp.map(|t: f64| t.max(temp)).unwrap_or(temp),
                            );
                        }
                    }
                    let crit_path = path.join(format!("temp{}_crit", i));
                    if let Ok(val) = std::fs::read_to_string(&crit_path) {
                        if let Ok(millideg) = val.trim().parse::<f64>() {
                            state.mem_temp_max = millideg / 1000.0;
                        }
                    }
                }
            }
        }
    }
}
