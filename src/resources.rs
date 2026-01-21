use macroquad::audio::{load_sound, play_sound_once, Sound};
use macroquad::logging::{info, error};

/// Manages game audio resources.
pub struct Resources {
    shoot: Option<Sound>,
    bang: Option<Sound>,
    warp: Option<Sound>,
}

impl Resources {
    /// Asynchronously loads all sound assets from the assets/ directory.
    ///
    /// Returns a `Resources` struct containing loaded sounds. 
    /// If a sound fails to load, it is logged, and that sound will simply not play.
    pub async fn load() -> Self {
        async fn load_snd(path: &str) -> Option<Sound> {
            match load_sound(path).await {
                Ok(snd) => {
                    info!("Audio resource loaded successfully: {}", path);
                    Some(snd)
                },
                Err(e) => {
                    error!("Failed to load audio resource '{}'. Error: {}", path, e);
                    None
                }
            }
        }

        Resources {
            shoot: load_snd("assets/shoot.wav").await,
            bang: load_snd("assets/bang.wav").await,
            warp: load_snd("assets/warp.wav").await,
        }
    }

    /// Plays a sound by name ("shoot", "bang", "warp").
    pub fn play(&self, sound_type: &str) {
        let sound = match sound_type {
            "shoot" => &self.shoot,
            "bang" => &self.bang,
            "warp" => &self.warp,
            _ => &None,
        };
        
        if let Some(s) = sound {
            play_sound_once(s);
        }
    }
}
