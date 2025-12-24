use std::fs;

#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    pub fn from_hex(hex: String) -> Self {
        let r = u8::from_str_radix(&hex[0..2], 16).expect("Invalid red channel value");
        let g = u8::from_str_radix(&hex[2..4], 16).expect("Invalid green channel value");
        let b = u8::from_str_radix(&hex[4..6], 16).expect("Invalid blue channel value");

        Self { r, g, b }
    }

    /// (0xFF >> 24) + (r >> 16) + (g >> 8) + b
    pub fn as_hex(&self) -> i32 {
        let r = self.r as u64;
        let g = self.g as u64;
        let b = self.b as u64;

        let r = r << 16;
        let g = g << 8;

        (r + g + b) as i32
    }
}

pub struct ConfigState {
    pub bar_color:  Color,
    pub text_color: Color
}

impl ConfigState {
    pub fn new() -> Self {
        let mut config = ConfigState::default();

        let path = {
            let home = std::env::var("HOME").expect("lmao $HOMEless");
            format!("{}/.config/svbar/config", home)
        };

        if let Ok(exists) = fs::exists(&path) {
            if !exists { return config; }
        }
        else { return config; }

        if let Some(contents) = fs::read_to_string(path).ok() {
            for line in contents.lines() {
                if let Some(hex) = line.strip_prefix("background=") {
                    config.bar_color = Color::from_hex(hex.to_string());
                }
                else if let Some(hex) = line.strip_prefix("foreground=") {
                    config.text_color = Color::from_hex(hex.to_string());
                }
            }
        }

        config
    }
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            bar_color: Color { r: 0, g: 0, b: 0 },
            text_color: Color { r: 255, g: 255, b: 255 },
        }
    }
}
