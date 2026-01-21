/// Configuration settings for a specific game level.
pub struct LevelConfig {
    /// Number of asteroids to spawn initially.
    pub asteroid_count: usize,
    /// Min and Max speed for asteroids.
    pub asteroid_speed_range: (f32, f32),
    /// Multiplier for asteroid size (1.0 = normal).
    pub asteroid_size_mult: f32,
    /// Probability per frame of a UFO spawning.
    pub ufo_spawn_chance: f32,
    /// Movement speed of the UFO.
    pub ufo_speed: f32,
}

/// Returns the list of level configurations for the game.
/// 
/// Adjust these values to tune game difficulty.
pub fn get_levels() -> Vec<LevelConfig> {
    vec![
        // Level 1: Introduction
        LevelConfig {
            asteroid_count: 4,
            asteroid_speed_range: (1.0, 2.0),
            asteroid_size_mult: 1.0,
            ufo_spawn_chance: 0.0, 
            ufo_speed: 0.0,
        },
        // Level 2: Faster, occasional UFOs
        LevelConfig {
            asteroid_count: 6,
            asteroid_speed_range: (2.0, 3.5),
            asteroid_size_mult: 1.0,
            ufo_spawn_chance: 0.002, 
            ufo_speed: 2.0,
        },
        // Level 3: Chaos
        LevelConfig {
            asteroid_count: 8,
            asteroid_speed_range: (3.0, 5.0),
            asteroid_size_mult: 1.2,
            ufo_spawn_chance: 0.008, 
            ufo_speed: 3.5,
        },
    ]
}

impl LevelConfig {
    /// Validates the configuration parameters to prevent runtime weirdness.
    pub fn validate(&self, level_idx: usize) -> Result<(), String> {
        if self.asteroid_count == 0 {
            return Err(format!("Level {}: Asteroid count must be greater than 0", level_idx + 1));
        }
        if self.asteroid_speed_range.0 > self.asteroid_speed_range.1 {
            return Err(format!("Level {}: Min asteroid speed cannot be greater than max speed", level_idx + 1));
        }
        if self.ufo_spawn_chance < 0.0 || self.ufo_spawn_chance > 1.0 {
            return Err(format!("Level {}: UFO spawn chance must be between 0.0 and 1.0", level_idx + 1));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_configs() {
        let levels = get_levels();
        for (i, config) in levels.iter().enumerate() {
            assert!(config.validate(i).is_ok());
        }
    }

    #[test]
    fn test_invalid_config() {
        let bad_config = LevelConfig {
            asteroid_count: 0,
            asteroid_speed_range: (1.0, 2.0),
            asteroid_size_mult: 1.0,
            ufo_spawn_chance: 0.5,
            ufo_speed: 1.0,
        };
        assert!(bad_config.validate(0).is_err());
    }
}
