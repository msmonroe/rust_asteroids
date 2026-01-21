use macroquad::prelude::*;
use macroquad::rand::gen_range;
use crate::components::*;
use crate::config::LevelConfig;

/// Resets and populates the level with asteroids based on the provided configuration.
/// 
/// This clears existing asteroids/UFOs and spawns new asteroids at safe distances from the center.
pub fn load_level(config: &LevelConfig, asteroids: &mut Vec<Asteroid>, ufos: &mut Vec<Ufo>) {
    asteroids.clear();
    ufos.clear(); 
    
    for _ in 0..config.asteroid_count {
        let mut pos;
        // Ensure asteroids don't spawn on top of the player (center screen)
        loop {
            pos = vec2(gen_range(0., screen_width()), gen_range(0., screen_height()));
            if pos.distance(vec2(screen_width()/2., screen_height()/2.)) > 150. {
                break;
            }
        }
        
        asteroids.push(Asteroid {
            pos,
            vel: vec2(
                gen_range(config.asteroid_speed_range.0, config.asteroid_speed_range.1) * if gen_range(0, 2) == 0 { 1. } else { -1. },
                gen_range(config.asteroid_speed_range.0, config.asteroid_speed_range.1) * if gen_range(0, 2) == 0 { 1. } else { -1. }
            ),
            radius: gen_range(30., 50.) * config.asteroid_size_mult,
            active: true,
            sides: gen_range(5, 9),
        });
    }
}
