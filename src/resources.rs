use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::logging::{error, info, warn};

/// Manages game audio resources.
pub struct Resources {
    shoot: Option<Sound>,
    bang: Option<Sound>,
    warp: Option<Sound>,
    volume: f32,
}

impl Resources {
    /// Asynchronously loads all sound assets from embedded bytes.
    ///
    /// Returns a `Resources` struct containing loaded sounds. 
    /// If a sound fails to load, it is logged, and that sound will simply not play.
    pub async fn load() -> Self {
        async fn load_snd(data: &[u8], name: &str) -> Option<Sound> {
            match load_sound_from_bytes(data).await {
                Ok(snd) => {
                    info!("Embedded audio resource loaded successfully: {}", name);
                    Some(snd)
                },
                Err(e) => {
                    error!("Failed to load embedded audio resource '{}'. Error: {}", name, e);
                    None
                }
            }
        }

        // Embed the assets into the binary at compile time
        let shoot_bytes = include_bytes!("../assets/shoot.wav");
        let bang_bytes = include_bytes!("../assets/bang.wav");
        let warp_bytes = include_bytes!("../assets/warp.wav");

        Resources {
            shoot: load_snd(shoot_bytes, "shoot").await,
            bang: load_snd(bang_bytes, "bang").await,
            warp: load_snd(warp_bytes, "warp").await,
            volume: 1.0,
        }
    }

    /// Updates the master sound volume (clamped to 0.0..=1.0).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Plays a sound by name ("shoot", "bang", "warp").
    pub fn play(&self, sound_type: &str) {
        let sound = match sound_type {
            "shoot" => &self.shoot,
            "bang" => &self.bang,
            "warp" => &self.warp,
            _ => {
                warn!("Unknown sound type requested: {}", sound_type);
                &None
            },
        };
        
        if let Some(s) = sound {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume: self.volume,
                },
            );
        }
    }
}
