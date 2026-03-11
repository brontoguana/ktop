mod app;
mod config;
mod gpu;
mod system;
mod theme;
mod ui;

use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let mut refresh = 1.0f64;
    let mut sim = false;
    let mut theme_override: Option<String> = None;

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-v" | "--version" => {
                println!("ktop {}", VERSION);
                return;
            }
            "-h" | "--help" => {
                println!("ktop {} — system monitor for hybrid LLM workloads", VERSION);
                println!();
                println!("Usage: ktop [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -v, --version          Print version");
                println!("  -r, --refresh <SECS>   Refresh interval (default: 1.0)");
                println!("  --theme <NAME>         Color theme");
                println!("  --sim                  Simulation mode");
                println!("  -h, --help             Print help");
                return;
            }
            "-r" | "--refresh" => {
                i += 1;
                if i < args.len() {
                    refresh = args[i].parse().unwrap_or(1.0);
                }
            }
            "--theme" => {
                i += 1;
                if i < args.len() {
                    theme_override = Some(args[i].clone());
                }
            }
            "--sim" => {
                sim = true;
            }
            _ => {}
        }
        i += 1;
    }

    if let Err(e) = app::run(refresh, sim, theme_override) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
