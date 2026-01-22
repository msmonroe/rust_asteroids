use macroquad::logging::{error, info, warn};
use std::fs;

#[derive(Clone, Debug, PartialEq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Difficulty::Easy,
            2 => Difficulty::Normal,
            3 => Difficulty::Hard,
            _ => Difficulty::Normal,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Difficulty::Easy => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Difficulty::Easy => Difficulty::Hard,
            Difficulty::Normal => Difficulty::Easy,
            Difficulty::Hard => Difficulty::Normal,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Difficulty::Easy => 1,
            Difficulty::Normal => 2,
            Difficulty::Hard => 3,
        }
    }

    pub fn speed_multiplier(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.85,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.2,
        }
    }

    pub fn spawn_multiplier(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.7,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.3,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub volume: f32,
    pub difficulty: Difficulty,
    pub show_fps: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            volume: 0.8,
            difficulty: Difficulty::Normal,
            show_fps: false,
        }
    }
}

impl Settings {
    pub fn clamp(&mut self) {
        if !(0.0..=1.0).contains(&self.volume) {
            warn!("Volume out of range; clamping to [0.0, 1.0]");
            self.volume = self.volume.clamp(0.0, 1.0);
        }
    }

    pub fn from_str(input: &str) -> Self {
        let mut settings = Settings::default();

        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() != 2 {
                continue;
            }
            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "volume" => {
                    if let Ok(v) = value.parse::<f32>() {
                        settings.volume = v;
                    }
                }
                "difficulty" => {
                    if let Ok(v) = value.parse::<u8>() {
                        settings.difficulty = Difficulty::from_u8(v);
                    }
                }
                "show_fps" => {
                    settings.show_fps = matches!(value, "1" | "true" | "True" | "TRUE");
                }
                _ => {}
            }
        }

        settings.clamp();
        settings
    }

    pub fn to_string(&self) -> String {
        format!(
            "volume={}\ndifficulty={}\nshow_fps={}\n",
            self.volume,
            self.difficulty.to_u8(),
            if self.show_fps { 1 } else { 0 }
        )
    }

    pub fn load_from_file(path: &str) -> Result<Self, String> {
        match fs::read_to_string(path) {
            Ok(contents) => {
                info!("Settings loaded from {}", path);
                Ok(Settings::from_str(&contents))
            }
            Err(e) => {
                warn!("Settings file not found or unreadable: {}. Using defaults.", e);
                Err(format!("Failed to read settings: {}", e))
            }
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        if let Err(e) = fs::write(path, self.to_string()) {
            error!("Failed to save settings to {}: {}", path, e);
            return Err(format!("Failed to write settings: {}", e));
        }
        info!("Settings saved to {}", path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_parse_defaults() {
        let s = Settings::from_str("");
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn test_settings_parse_values() {
        let s = Settings::from_str("volume=0.5\ndifficulty=3\nshow_fps=true\n");
        assert_eq!(s.volume, 0.5);
        assert_eq!(s.difficulty, Difficulty::Hard);
        assert!(s.show_fps);
    }

    #[test]
    fn test_settings_clamp() {
        let s = Settings::from_str("volume=2.5\n");
        assert_eq!(s.volume, 1.0);
    }

    #[test]
    fn test_difficulty_cycle() {
        assert_eq!(Difficulty::Easy.next(), Difficulty::Normal);
        assert_eq!(Difficulty::Normal.next(), Difficulty::Hard);
        assert_eq!(Difficulty::Hard.next(), Difficulty::Easy);

        assert_eq!(Difficulty::Easy.prev(), Difficulty::Hard);
        assert_eq!(Difficulty::Normal.prev(), Difficulty::Easy);
        assert_eq!(Difficulty::Hard.prev(), Difficulty::Normal);
    }
}
