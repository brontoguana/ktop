use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    pub gpu: Color,
    pub cpu: Color,
    pub mem: Color,
    pub proc_mem: Color,
    pub proc_cpu: Color,
    pub bar_low: Color,
    pub bar_mid: Color,
    pub bar_high: Color,
    pub net: Color,
    pub net_up: Color,
    pub net_down: Color,
}

fn hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
    Color::Rgb(r, g, b)
}

fn named(s: &str) -> Color {
    match s {
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "red" => Color::Red,
        "white" => Color::White,
        "bright_magenta" => Color::LightMagenta,
        "bright_cyan" => Color::LightCyan,
        "bright_green" => Color::LightGreen,
        "bright_yellow" => Color::LightYellow,
        "bright_red" => Color::LightRed,
        "bright_white" => Color::White,
        "dark_green" => Color::DarkGray,
        "dim" => Color::DarkGray,
        _ if s.starts_with('#') => hex(s),
        _ => Color::White,
    }
}

fn t(gpu: &str, cpu: &str, mem: &str, pm: &str, pc: &str, lo: &str, mid: &str, hi: &str,
     net: Option<&str>, net_up: Option<&str>, net_down: Option<&str>) -> Theme {
    Theme {
        gpu: named(gpu),
        cpu: named(cpu),
        mem: named(mem),
        proc_mem: named(pm),
        proc_cpu: named(pc),
        bar_low: named(lo),
        bar_mid: named(mid),
        bar_high: named(hi),
        net: named(net.unwrap_or(cpu)),
        net_up: named(net_up.unwrap_or(gpu)),
        net_down: named(net_down.unwrap_or(net.unwrap_or(cpu))),
    }
}

pub fn theme_names() -> Vec<&'static str> {
    vec![
        "Default", "Monokai", "Dracula", "Nord", "Solarized", "Gruvbox", "One Dark",
        "Tokyo Night", "Catppuccin Mocha", "Catppuccin Latte", "Rosé Pine", "Everforest",
        "Kanagawa",
        "Monochrome", "Green Screen", "Amber", "Phosphor",
        "Ocean", "Sunset", "Forest", "Lava", "Arctic", "Sakura", "Mint", "Lavender",
        "Coral", "Cyberpunk", "Neon", "Synthwave", "Vaporwave", "Matrix",
        "Pastel", "Soft", "Cotton Candy", "Ice Cream",
        "Electric", "Inferno", "Glacier", "Twilight", "Autumn", "Spring", "Summer", "Winter",
        "High Contrast", "Blueprint", "Redshift", "Emerald", "Royal", "Bubblegum", "Horizon",
    ]
}

pub fn get_theme(name: &str) -> Theme {
    match name {
        "Default" => t("magenta", "cyan", "green", "green", "cyan", "green", "yellow", "red", None, None, None),
        "Monokai" => t("bright_magenta", "bright_cyan", "bright_green", "bright_green", "bright_cyan", "green", "yellow", "red", None, None, None),
        "Dracula" => t("#bd93f9", "#8be9fd", "#50fa7b", "#50fa7b", "#8be9fd", "#50fa7b", "#f1fa8c", "#ff5555", None, None, None),
        "Nord" => t("#b48ead", "#88c0d0", "#a3be8c", "#a3be8c", "#88c0d0", "#a3be8c", "#ebcb8b", "#bf616a", None, None, None),
        "Solarized" => t("#d33682", "#2aa198", "#859900", "#859900", "#2aa198", "#859900", "#b58900", "#dc322f", None, None, None),
        "Gruvbox" => t("#d3869b", "#83a598", "#b8bb26", "#b8bb26", "#83a598", "#b8bb26", "#fabd2f", "#fb4934", None, None, None),
        "One Dark" => t("#c678dd", "#56b6c2", "#98c379", "#98c379", "#56b6c2", "#98c379", "#e5c07b", "#e06c75", None, None, None),
        "Tokyo Night" => t("#bb9af7", "#7dcfff", "#9ece6a", "#9ece6a", "#7dcfff", "#9ece6a", "#e0af68", "#f7768e", None, None, None),
        "Catppuccin Mocha" => t("#cba6f7", "#89dceb", "#a6e3a1", "#a6e3a1", "#89dceb", "#a6e3a1", "#f9e2af", "#f38ba8", None, None, None),
        "Catppuccin Latte" => t("#8839ef", "#04a5e5", "#40a02b", "#40a02b", "#04a5e5", "#40a02b", "#df8e1d", "#d20f39", None, None, None),
        "Rosé Pine" => t("#c4a7e7", "#9ccfd8", "#31748f", "#31748f", "#9ccfd8", "#31748f", "#f6c177", "#eb6f92", None, None, None),
        "Everforest" => t("#d699b6", "#7fbbb3", "#a7c080", "#a7c080", "#7fbbb3", "#a7c080", "#dbbc7f", "#e67e80", None, None, None),
        "Kanagawa" => t("#957fb8", "#7e9cd8", "#98bb6c", "#98bb6c", "#7e9cd8", "#98bb6c", "#e6c384", "#c34043", None, None, None),
        "Monochrome" => t("white", "white", "white", "white", "white", "bright_white", "white", "#888888", None, None, None),
        "Green Screen" => t("green", "green", "green", "green", "green", "bright_green", "green", "dark_green", None, None, None),
        "Amber" => t("#ffbf00", "#ffbf00", "#ffbf00", "#ffbf00", "#ffbf00", "#ffd700", "#ffbf00", "#ff8c00", None, None, None),
        "Phosphor" => t("#33ff00", "#33ff00", "#33ff00", "#33ff00", "#33ff00", "#66ff33", "#33ff00", "#009900", None, None, None),
        "Ocean" => t("#6c5ce7", "#0984e3", "#00b894", "#00b894", "#0984e3", "#00b894", "#fdcb6e", "#d63031", None, None, None),
        "Sunset" => t("#e17055", "#fdcb6e", "#fab1a0", "#fab1a0", "#fdcb6e", "#ffeaa7", "#e17055", "#d63031", None, None, None),
        "Forest" => t("#00b894", "#55efc4", "#00cec9", "#00cec9", "#55efc4", "#55efc4", "#ffeaa7", "#e17055", None, None, None),
        "Lava" => t("#ff6348", "#ff4757", "#ff6b81", "#ff6b81", "#ff4757", "#ffa502", "#ff6348", "#ff3838", None, None, None),
        "Arctic" => t("#dfe6e9", "#74b9ff", "#81ecec", "#81ecec", "#74b9ff", "#81ecec", "#74b9ff", "#a29bfe", None, None, None),
        "Sakura" => t("#fd79a8", "#e84393", "#fab1a0", "#fab1a0", "#e84393", "#fab1a0", "#fd79a8", "#e84393", None, None, None),
        "Mint" => t("#00b894", "#00cec9", "#55efc4", "#55efc4", "#00cec9", "#55efc4", "#81ecec", "#ff7675", None, None, None),
        "Lavender" => t("#a29bfe", "#6c5ce7", "#dfe6e9", "#dfe6e9", "#6c5ce7", "#a29bfe", "#6c5ce7", "#fd79a8", None, None, None),
        "Coral" => t("#ff7675", "#fab1a0", "#ffeaa7", "#ffeaa7", "#fab1a0", "#ffeaa7", "#ff7675", "#d63031", None, None, None),
        "Cyberpunk" => t("#ff00ff", "#00ffff", "#ff00aa", "#ff00aa", "#00ffff", "#00ff00", "#ffff00", "#ff0000", None, None, None),
        "Neon" => t("#ff6ec7", "#00ffff", "#39ff14", "#39ff14", "#00ffff", "#39ff14", "#ffff00", "#ff073a", None, None, None),
        "Synthwave" => t("#f72585", "#4cc9f0", "#7209b7", "#7209b7", "#4cc9f0", "#4cc9f0", "#f72585", "#ff0a54", None, None, None),
        "Vaporwave" => t("#ff71ce", "#01cdfe", "#05ffa1", "#05ffa1", "#01cdfe", "#05ffa1", "#b967ff", "#ff71ce", None, None, None),
        "Matrix" => t("#00ff41", "#008f11", "#003b00", "#003b00", "#008f11", "#00ff41", "#008f11", "#003b00", None, None, None),
        "Pastel" => t("#c39bd3", "#85c1e9", "#82e0aa", "#82e0aa", "#85c1e9", "#82e0aa", "#f9e79f", "#f1948a", None, None, None),
        "Soft" => t("#bb8fce", "#76d7c4", "#7dcea0", "#7dcea0", "#76d7c4", "#7dcea0", "#f0b27a", "#ec7063", None, None, None),
        "Cotton Candy" => t("#ffb3ba", "#bae1ff", "#baffc9", "#baffc9", "#bae1ff", "#baffc9", "#ffffba", "#ffb3ba", None, None, None),
        "Ice Cream" => t("#ff9a9e", "#a1c4fd", "#c2e9fb", "#c2e9fb", "#a1c4fd", "#c2e9fb", "#ffecd2", "#ff9a9e", None, None, None),
        "Electric" => t("#7b2ff7", "#00d4ff", "#00ff87", "#00ff87", "#00d4ff", "#00ff87", "#ffd000", "#ff0055", None, None, None),
        "Inferno" => t("#ff4500", "#ff6a00", "#ff8c00", "#ff8c00", "#ff6a00", "#ffd700", "#ff8c00", "#ff0000", None, None, None),
        "Glacier" => t("#e0f7fa", "#80deea", "#4dd0e1", "#4dd0e1", "#80deea", "#80deea", "#4dd0e1", "#00838f", None, None, None),
        "Twilight" => t("#7c4dff", "#448aff", "#18ffff", "#18ffff", "#448aff", "#18ffff", "#7c4dff", "#ff1744", None, None, None),
        "Autumn" => t("#d35400", "#e67e22", "#f39c12", "#f39c12", "#e67e22", "#f1c40f", "#e67e22", "#c0392b", None, None, None),
        "Spring" => t("#e91e63", "#00bcd4", "#8bc34a", "#8bc34a", "#00bcd4", "#8bc34a", "#ffeb3b", "#f44336", None, None, None),
        "Summer" => t("#ff9800", "#03a9f4", "#4caf50", "#4caf50", "#03a9f4", "#4caf50", "#ffeb3b", "#f44336", None, None, None),
        "Winter" => t("#9c27b0", "#3f51b5", "#607d8b", "#607d8b", "#3f51b5", "#607d8b", "#9c27b0", "#e91e63", None, None, None),
        "High Contrast" => t("bright_magenta", "bright_cyan", "bright_green", "bright_green", "bright_cyan", "bright_green", "bright_yellow", "bright_red", None, None, None),
        "Blueprint" => t("#4fc3f7", "#29b6f6", "#03a9f4", "#03a9f4", "#29b6f6", "#4fc3f7", "#0288d1", "#01579b", None, None, None),
        "Redshift" => t("#ef5350", "#e53935", "#c62828", "#c62828", "#e53935", "#ef9a9a", "#ef5350", "#b71c1c", None, None, None),
        "Emerald" => t("#66bb6a", "#43a047", "#2e7d32", "#2e7d32", "#43a047", "#a5d6a7", "#66bb6a", "#1b5e20", None, None, None),
        "Royal" => t("#7e57c2", "#5c6bc0", "#42a5f5", "#42a5f5", "#5c6bc0", "#42a5f5", "#7e57c2", "#d32f2f", None, None, None),
        "Bubblegum" => t("#ff77a9", "#ff99cc", "#ffb3d9", "#ffb3d9", "#ff99cc", "#ffb3d9", "#ff77a9", "#ff3385", None, None, None),
        "Horizon" => t("#e95678", "#fab795", "#25b0bc", "#25b0bc", "#fab795", "#25b0bc", "#fab795", "#e95678", None, None, None),
        _ => t("magenta", "cyan", "green", "green", "cyan", "green", "yellow", "red", None, None, None),
    }
}
