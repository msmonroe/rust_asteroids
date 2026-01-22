mod components;
mod config;
mod resources;
mod physics;
mod level;
mod settings;
mod ui;
mod particles;

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use components::*;
use config::get_levels;
use resources::Resources;
use physics::*;
use level::load_level;
use settings::Settings;
use ui::{draw_pause_screen, draw_settings_screen, draw_title_screen};
use particles::{ParticleSpawnBridge, ParticleSystem, SpawnRequest};

#[derive(Clone, Copy, PartialEq)]
enum GameState {
    Title,
    Playing,
    Paused,
    Settings,
}

#[derive(Clone, Copy, PartialEq)]
enum SettingsOrigin {
    Title,
    Pause,
}

/// Main Entry Point
#[macroquad::main("Rust Asteroids")]
async fn main() {
    info!("Starting Rust Asteroids");
    info!("Running in Standalone Mode (Assets Embedded)");

    let levels = get_levels();
    
    // Validate Level Configurations
    for (i, config) in levels.iter().enumerate() {
        if let Err(e) = config.validate(i) {
            error!("Configuration Error: {}", e);
            // In a real app we might panic or show an error screen, 
            // but here we just log it and likely crash later or have weird behavior.
            // For safety, let's panic to prevent running with bad config
            panic!("Invalid Level Configuration: {}", e);
        }
    }

    let mut current_level_idx = 0;
    
    // Load Audio Resources safely
    let mut resources = Resources::load().await;

    // Settings (loaded on background thread)
    let settings_path = "settings.cfg".to_string();
    let (settings_tx, settings_rx) = mpsc::channel();
    let settings_path_clone = settings_path.clone();
    thread::spawn(move || {
        let result = Settings::load_from_file(&settings_path_clone);
        let _ = settings_tx.send(result);
    });
    let mut settings = Settings::default();
    let mut settings_rx = Some(settings_rx);

    // Game State initialization
    let mut score = 0;
    let mut lives = 3;
    let mut last_extra_life_score = 0;
    let mut player = Player {
        pos: vec2(screen_width() / 2., screen_height() / 2.),
        vel: vec2(0., 0.),
        rotation: 0.,
        radius: 15.,
        active: true,
        sides: 3,
        color: WHITE,
        invulnerable: false,
        invulnerable_timer: 0.,
    };
    
    let mut design_mode = false;
    let available_colors = [WHITE, RED, ORANGE, YELLOW, GREEN, SKYBLUE, BLUE, PURPLE, PINK];
    let mut current_color_idx = 0;
    
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut asteroids: Vec<Asteroid> = Vec::new();
    let mut ufos: Vec<Ufo> = Vec::new();
    
    let mut game_over = false;
    let mut game_won = false;

    let mut game_state = GameState::Title;
    let mut settings_origin = SettingsOrigin::Title;

    // Multitasking Demo: Background Scanner
    let mut is_scanning = false;
    let mut scan_receiver: Option<mpsc::Receiver<String>> = None;
    let mut scan_message = String::new();
    let mut scan_message_timer = 0.0;

    // Particle system + spawn bridge (background generation)
    let mut particle_system = ParticleSystem::new();
    let particle_spawner = ParticleSpawnBridge::new();

    // Initial Level Load (uses default settings until loaded)
    let initial_cfg = levels[current_level_idx].scaled(
        settings.difficulty.speed_multiplier(),
        settings.difficulty.spawn_multiplier(),
    );
    load_level(&initial_cfg, &mut asteroids, &mut ufos);

    loop {
        // Check for async settings load
        if let Some(rx) = &settings_rx {
            if let Ok(result) = rx.try_recv() {
                settings_rx = None;
                match result {
                    Ok(loaded) => {
                        settings = loaded;
                        resources.set_volume(settings.volume);
                        info!("Settings loaded: volume={:.2}, difficulty={:?}, show_fps={}", settings.volume, settings.difficulty, settings.show_fps);
                    }
                    Err(err) => {
                        warn!("Using default settings. Reason: {}", err);
                    }
                }
            }
        }

        // Collect particle batches from background worker
        if let Some(batch) = particle_spawner.try_receive() {
            particle_system.spawn_batch(batch);
        }

        // --- TITLE / PAUSE / SETTINGS SCREENS ---
        if game_state == GameState::Title {
            draw_title_screen();
            if is_key_pressed(KeyCode::Enter) {
                info!("Title -> Playing");
                game_state = GameState::Playing;
            }
            if is_key_pressed(KeyCode::S) {
                info!("Title -> Settings");
                settings_origin = SettingsOrigin::Title;
                game_state = GameState::Settings;
            }
            if is_key_pressed(KeyCode::Escape) {
                break;
            }
            next_frame().await;
            continue;
        }

        if game_state == GameState::Settings {
            draw_settings_screen(&settings);

            if is_key_pressed(KeyCode::Left) {
                settings.volume = (settings.volume - 0.05).clamp(0.0, 1.0);
                resources.set_volume(settings.volume);
                info!("Settings volume changed: {:.2}", settings.volume);
            }
            if is_key_pressed(KeyCode::Right) {
                settings.volume = (settings.volume + 0.05).clamp(0.0, 1.0);
                resources.set_volume(settings.volume);
                info!("Settings volume changed: {:.2}", settings.volume);
            }
            if is_key_pressed(KeyCode::Up) {
                settings.difficulty = settings.difficulty.next();
                info!("Settings difficulty changed: {:?}", settings.difficulty);
            }
            if is_key_pressed(KeyCode::Down) {
                settings.difficulty = settings.difficulty.prev();
                info!("Settings difficulty changed: {:?}", settings.difficulty);
            }
            if is_key_pressed(KeyCode::F) {
                settings.show_fps = !settings.show_fps;
                info!("Settings show_fps toggled: {}", settings.show_fps);
            }

            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                let save_settings = settings.clone();
                let save_path = settings_path.clone();
                thread::spawn(move || {
                    let _ = save_settings.save_to_file(&save_path);
                });
                info!("Settings saved. Returning from settings menu.");
                game_state = match settings_origin {
                    SettingsOrigin::Title => GameState::Title,
                    SettingsOrigin::Pause => GameState::Paused,
                };
            }

            next_frame().await;
            continue;
        }

        if game_state == GameState::Paused {
            draw_pause_screen();
            if is_key_pressed(KeyCode::P) {
                info!("Paused -> Playing");
                game_state = GameState::Playing;
            }
            if is_key_pressed(KeyCode::S) {
                info!("Paused -> Settings");
                settings_origin = SettingsOrigin::Pause;
                game_state = GameState::Settings;
            }
            if is_key_pressed(KeyCode::Escape) {
                info!("Paused -> Title");
                game_state = GameState::Title;
            }
            next_frame().await;
            continue;
        }

        // --- GAME OVER / WIN SCREEN ---
        if game_over || game_won {
            clear_background(BLACK);
            let title = if game_won { "YOU WIN!" } else { "GAME OVER" };
            draw_text(title, screen_width()/2. - 100., screen_height()/2. - 50., 40., WHITE);
            draw_text(&format!("Final Score: {}", score), screen_width()/2. - 100., screen_height()/2., 30., GREEN);
            draw_text("Press R to Restart", screen_width()/2. - 120., screen_height()/2. + 50., 20., GRAY);
            draw_text("Press Esc to Quit", screen_width()/2. - 110., screen_height()/2. + 80., 20., GRAY);
            
            if is_key_pressed(KeyCode::R) {
                current_level_idx = 0;
                score = 0;
                lives = 3;
                last_extra_life_score = 0;
                game_over = false;
                game_won = false;
                game_state = GameState::Playing;
                player.pos = vec2(screen_width() / 2., screen_height() / 2.);
                player.vel = vec2(0., 0.);
                player.active = true;
                player.invulnerable = true;
                player.invulnerable_timer = 3.0;
                info!("Restart: player respawned at center with 3s invulnerability.");
                bullets.clear();
                let restart_cfg = levels[current_level_idx].scaled(
                    settings.difficulty.speed_multiplier(),
                    settings.difficulty.spawn_multiplier(),
                );
                load_level(&restart_cfg, &mut asteroids, &mut ufos);
            }

            if is_key_pressed(KeyCode::Escape) {
                break;
            }

            next_frame().await;
            continue;
        }

        // --- MULTITASKING: BACKGROUND SCANNER ---
        // Press 'S' to offload a task to another thread
        if !is_scanning && is_key_pressed(KeyCode::S) {
            info!("Starting background scan...");
            is_scanning = true;
            scan_message = "Scanning Deep Space...".to_string();
            
            let (tx, rx) = mpsc::channel();
            scan_receiver = Some(rx);

            // Spawn a separate OS thread to do "heavy work" without freezing the game
            thread::spawn(move || {
                // Simulate heavy calculation (blocking this thread)
                thread::sleep(Duration::from_secs(2)); 
                // Send result back to main thread
                let _ = tx.send("Sector Analysis Complete: Data Cache Found! +500 pts".to_string());
            });
        }

        // Check for results from the thread every frame
        if is_scanning {
            if let Some(rx) = &scan_receiver {
                if let Ok(msg) = rx.try_recv() {
                    // Message received! Thread has finished.
                    is_scanning = false;
                    scan_message = msg;
                    scan_message_timer = 3.0; // Show result for 3 seconds
                    score += 500;
                    info!("Background scan complete. Bonus awarded. Score: {}", score);
                    // resources.play("warp"); // Optional: Audio feedback
                }
            }
        }

        // --- DESIGN MODE (Secret Tool) ---
        if is_key_pressed(KeyCode::D) {
            design_mode = !design_mode;
            info!("Design mode toggled: {}", if design_mode { "ON" } else { "OFF" });
            // Reset velocity if entering design mode to prevent drifting while editing
            if design_mode {
                player.vel = vec2(0., 0.);
            }
        }

        if design_mode {
            clear_background(DARKGRAY);
            
            // Design Inputs
            if is_key_pressed(KeyCode::Right) && player.sides < 8 { player.sides += 1; }
            if is_key_pressed(KeyCode::Left) && player.sides > 3 { player.sides -= 1; }
            
            if is_key_pressed(KeyCode::Up) && player.radius < 50. { player.radius += 1.; }
            if is_key_pressed(KeyCode::Down) && player.radius > 5. { player.radius -= 1.; }

            if is_key_pressed(KeyCode::C) {
                current_color_idx = (current_color_idx + 1) % available_colors.len();
                player.color = available_colors[current_color_idx];
            }

            // Draw Preview
            draw_text("DESIGN MODE", screen_width()/2. - 80., 50., 30., WHITE);
            draw_text("Use Arrows to change Shape/Size", screen_width()/2. - 120., 80., 20., LIGHTGRAY);
            draw_text("Press 'C' to change Color", screen_width()/2. - 90., 100., 20., LIGHTGRAY);
            draw_text("Press 'D' to Return", screen_width()/2. - 70., 120., 20., LIGHTGRAY);

            // Display current stats
            draw_text(&format!("Sides: {}", player.sides), 50., screen_height() - 100., 20., WHITE);
            draw_text(&format!("Radius: {:.1}", player.radius), 50., screen_height() - 80., 20., WHITE);

            // Draw Ship (Static centered)
            draw_poly_lines(screen_width()/2., screen_height()/2., player.sides, player.radius, -90., 3., player.color);

            next_frame().await;
            continue;
        }

        // --- CHECK EXTRA LIFE ---
        if check_extra_life(score, &mut last_extra_life_score) {
            lives += 1;
            resources.play("warp"); // Joyful sound
            info!("Extra life gained! Lives: {}", lives);
        }

        let level_cfg = &levels[current_level_idx];
        let effective_cfg = level_cfg.scaled(
            settings.difficulty.speed_multiplier(),
            settings.difficulty.spawn_multiplier(),
        );

        // --- SPAWN LOGIC (UFO) ---
        if ufos.len() < 1 && gen_range(0.0, 1.0) < effective_cfg.ufo_spawn_chance {
            resources.play("warp"); // Sound cue for UFO entry
            ufos.push(Ufo {
                pos: vec2(0., gen_range(0., screen_height())), 
                vel: vec2(effective_cfg.ufo_speed, 0.),
                radius: 20.,
                active: true,
                shoot_timer: 0.,
                change_dir_timer: 0.,
            });
        }

        // --- LEVEL PROGRESSION ---
        if asteroids.is_empty() && ufos.is_empty() {
            current_level_idx += 1;
            if current_level_idx >= levels.len() {
                game_won = true;
                info!("All levels cleared. Game won.");
            } else {
                info!("Level complete. Advancing to level {}", current_level_idx + 1);
                player.pos = vec2(screen_width() / 2., screen_height() / 2.);
                player.vel = vec2(0., 0.);
                bullets.clear();
                let next_cfg = levels[current_level_idx].scaled(
                    settings.difficulty.speed_multiplier(),
                    settings.difficulty.spawn_multiplier(),
                );
                load_level(&next_cfg, &mut asteroids, &mut ufos);
            }
        }

        // --- PRE-INPUT UPDATES ---
        // Invulnerability countdown (used after respawn to avoid instant re-hit)
        if player.invulnerable {
            player.invulnerable_timer -= get_frame_time();
            if player.invulnerable_timer <= 0.0 {
                player.invulnerable = false;
                info!("Invulnerability ended.");
            }
        }

        // --- INPUT ---
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if is_key_pressed(KeyCode::P) {
            info!("Playing -> Paused");
            game_state = GameState::Paused;
            next_frame().await;
            continue;
        }

        let rotation_speed = 4.0;
        let thrust_power = 0.15;
        
        if is_key_down(KeyCode::Left) { player.rotation -= rotation_speed * get_frame_time(); }
        if is_key_down(KeyCode::Right) { player.rotation += rotation_speed * get_frame_time(); }
        
        if is_key_down(KeyCode::Up) {
            let direction = vec2(player.rotation.cos(), player.rotation.sin());
            player.vel += direction * thrust_power;
        }

        // Shoot
        if is_key_pressed(KeyCode::Space) {
            resources.play("shoot");
            let direction = vec2(player.rotation.cos(), player.rotation.sin());
            bullets.push(Bullet {
                pos: player.pos + direction * player.radius,
                vel: direction * 8.0,
                lifetime: 1.5,
                active: true,
                owner: BulletOwner::Player,
            });
            info!("Player fired. Bullets active: {}", bullets.len());

            particle_spawner.request(SpawnRequest {
                pos: player.pos + direction * player.radius,
                color: YELLOW,
                count: 8,
                speed: 80.0,
                life: 0.25,
                size: 2.0,
            });
        }

        // HYPERSPACE (Shift Key)
        if is_key_pressed(KeyCode::LeftShift) {
            resources.play("warp");
            player.pos = vec2(gen_range(0., screen_width()), gen_range(0., screen_height()));
            player.vel = vec2(0., 0.); // Reset velocity for safety
            info!("Hyperspace jump to ({:.1}, {:.1})", player.pos.x, player.pos.y);
        }

        // --- PHYSICS & UPDATES ---

        // Player
        player.pos += player.vel;
        player.vel *= 0.98; // Friction
        player.pos = wrap_screen(player.pos);

        // Particles
        particle_system.update(get_frame_time());

        // Bullets
        for bullet in bullets.iter_mut() {
            bullet.pos += bullet.vel;
            bullet.lifetime -= get_frame_time();
            if bullet.lifetime <= 0. { bullet.active = false; }
            bullet.pos = wrap_screen(bullet.pos);
        }

        // Asteroids
        for asteroid in asteroids.iter_mut() {
            asteroid.pos += asteroid.vel;
            asteroid.pos = wrap_screen(asteroid.pos);
        }

        // UFO Logic
        for ufo in ufos.iter_mut() {
            ufo.pos += ufo.vel;
            ufo.pos = wrap_screen(ufo.pos);
            ufo.change_dir_timer += get_frame_time();
            ufo.shoot_timer += get_frame_time();

            if ufo.change_dir_timer > 2.0 {
                ufo.vel.y = gen_range(-1.0, 1.0) * effective_cfg.ufo_speed;
                ufo.change_dir_timer = 0.;
            }

            if ufo.shoot_timer > 2.0 {
                ufo.shoot_timer = 0.;
                resources.play("shoot"); 
                // AI Aiming
                let target = if asteroids.len() > 0 && gen_range(0, 3) == 0 {
                    asteroids[0].pos 
                } else {
                    player.pos 
                };

                let aim_dir = (target - ufo.pos).normalize();
                bullets.push(Bullet {
                    pos: ufo.pos + aim_dir * ufo.radius,
                    vel: aim_dir * 6.0,
                    lifetime: 2.0,
                    active: true,
                    owner: BulletOwner::Ufo,
                });
            }
        }

        // --- COLLISIONS ---
        
        let mut new_asteroids = Vec::new();
        
        for bullet in bullets.iter_mut() {
            if !bullet.active { continue; }

            // Bullet vs Asteroid
            for asteroid in asteroids.iter_mut() {
                if !asteroid.active { continue; }
                if check_collision(bullet.pos, 0., asteroid.pos, asteroid.radius) {
                    bullet.active = false;
                    asteroid.active = false;
                    resources.play("bang");
                    
                    if bullet.owner == BulletOwner::Player { 
                        score += 100; 
                        info!("Asteroid destroyed. Score: {}", score);
                        particle_spawner.request(SpawnRequest {
                            pos: asteroid.pos,
                            color: GRAY,
                            count: 28,
                            speed: 120.0,
                            life: 0.9,
                            size: 2.5,
                        });
                    }

                    // Split asteroid
                    if asteroid.radius > 15.0 {
                        let new_radius = asteroid.radius / 2.0;
                        for _ in 0..2 {
                            new_asteroids.push(Asteroid {
                                pos: asteroid.pos,
                                vel: vec2(gen_range(-2., 2.), gen_range(-2., 2.)),
                                radius: new_radius,
                                active: true,
                                sides: gen_range(5, 9),
                            });
                        }
                    }
                    break;
                }
            }

            // Bullet vs UFO
            if bullet.active && bullet.owner == BulletOwner::Player {
                for ufo in ufos.iter_mut() {
                    if !ufo.active { continue; }
                    if check_collision(bullet.pos, 0., ufo.pos, ufo.radius) {
                        ufo.active = false;
                        bullet.active = false;
                        resources.play("bang");
                        score += 500; 
                        info!("UFO destroyed. Score: {}", score);
                        particle_spawner.request(SpawnRequest {
                            pos: ufo.pos,
                            color: RED,
                            count: 32,
                            speed: 140.0,
                            life: 0.8,
                            size: 3.0,
                        });
                    }
                }
            }

            // Bullet vs Player
            if bullet.active && bullet.owner == BulletOwner::Ufo {
                 if !player.invulnerable && check_collision(bullet.pos, 0., player.pos, player.radius) {
                     resources.play("bang");
                     lives -= 1;
                     bullet.active = false;
                     particle_spawner.request(SpawnRequest {
                         pos: player.pos,
                         color: ORANGE,
                         count: 30,
                         speed: 160.0,
                         life: 0.9,
                         size: 3.0,
                     });
                     if lives <= 0 {
                         game_over = true;
                         warn!("Game Over: Player hit by UFO bullet. Lives: 0");
                     } else {
                         warn!("Player hit by UFO bullet. Lives remaining: {}", lives);
                         player.pos = vec2(gen_range(0., screen_width()), gen_range(0., screen_height()));
                         player.vel = vec2(0., 0.);
                         player.invulnerable = true;
                         player.invulnerable_timer = 3.0;
                         info!(
                             "Respawned after UFO bullet at ({:.1}, {:.1}) with 3s invulnerability.",
                             player.pos.x, player.pos.y
                         );
                     }
                 }
            }
        }

        // Physical Collisions (Ship vs Asteroid / UFO vs Asteroid / Ship vs UFO)
        for asteroid in asteroids.iter_mut() {
            if !asteroid.active { continue; }
            
            // Player vs Asteroid
            if !player.invulnerable && check_collision(player.pos, player.radius, asteroid.pos, asteroid.radius) {
                resources.play("bang");
                lives -= 1;
                asteroid.active = false; // Destroy asteroid on impact
                particle_spawner.request(SpawnRequest {
                    pos: player.pos,
                    color: ORANGE,
                    count: 30,
                    speed: 160.0,
                    life: 0.9,
                    size: 3.0,
                });

                // Split implicitly if large
                if asteroid.radius > 15.0 {
                    let new_radius = asteroid.radius / 2.0;
                    for _ in 0..2 {
                        new_asteroids.push(Asteroid {
                            pos: asteroid.pos,
                            vel: vec2(gen_range(-2., 2.), gen_range(-2., 2.)),
                            radius: new_radius,
                            active: true,
                            sides: gen_range(5, 9),
                        });
                    }
                }
                
                if lives <= 0 {
                    game_over = true;
                     warn!("Game Over: Player hit by asteroid. Lives: 0");
                } else {
                     warn!("Player hit by asteroid. Lives remaining: {}", lives);
                     player.pos = vec2(gen_range(0., screen_width()), gen_range(0., screen_height()));
                     player.vel = vec2(0., 0.);
                     player.invulnerable = true;
                     player.invulnerable_timer = 3.0;
                     info!(
                        "Respawned after asteroid at ({:.1}, {:.1}) with 3s invulnerability.",
                        player.pos.x, player.pos.y
                     );
                }
                // We break here to avoid processing this asteroid more in this frame (though it is marked inactive now)
                // Need to be careful with double mutations if we didn't use continue
            }
        }

        // UFO vs Asteroid
        // Note: We need a second pass or careful index handling if we want to mutate both in same loop.
        // Simplified approach: Iterate UFOs and check against asteroids.
        for ufo in ufos.iter_mut() {
            if !ufo.active { continue; }
            for asteroid in asteroids.iter_mut() {
                if !asteroid.active { continue; }
                if check_collision(ufo.pos, ufo.radius, asteroid.pos, asteroid.radius) {
                    ufo.active = false;
                    asteroid.active = false; 

                    if asteroid.radius > 15.0 {
                         let new_radius = asteroid.radius / 2.0;
                        for _ in 0..2 {
                            new_asteroids.push(Asteroid {
                                pos: asteroid.pos,
                                vel: vec2(gen_range(-2., 2.), gen_range(-2., 2.)),
                                radius: new_radius,
                                active: true,
                                sides: gen_range(5, 9),
                            });
                        }
                    }

                    resources.play("bang");
                    info!("UFO collided with asteroid.");
                    particle_spawner.request(SpawnRequest {
                        pos: asteroid.pos,
                        color: GRAY,
                        count: 24,
                        speed: 120.0,
                        life: 0.8,
                        size: 2.5,
                    });
                }
            }
        }
        
        // Player vs UFO
        for ufo in ufos.iter_mut() {
            if ufo.active && !player.invulnerable && check_collision(player.pos, player.radius, ufo.pos, ufo.radius) {
                ufo.active = false; // Destroy UFO
                resources.play("bang");
                lives -= 1;
                particle_spawner.request(SpawnRequest {
                    pos: player.pos,
                    color: ORANGE,
                    count: 30,
                    speed: 160.0,
                    life: 0.9,
                    size: 3.0,
                });
                if lives <= 0 {
                    game_over = true;
                    warn!("Game Over: Player hit by UFO. Lives: 0");
                } else {
                    warn!("Player hit by UFO. Lives remaining: {}", lives);
                    player.pos = vec2(gen_range(0., screen_width()), gen_range(0., screen_height()));
                    player.vel = vec2(0., 0.);
                    player.invulnerable = true;
                    player.invulnerable_timer = 3.0;
                    info!(
                        "Respawned after UFO collision at ({:.1}, {:.1}) with 3s invulnerability.",
                        player.pos.x, player.pos.y
                    );
                }
            }
        }

        // Clean up
        bullets.retain(|b| b.active);
        asteroids.retain(|a| a.active);
        ufos.retain(|u| u.active);
        asteroids.append(&mut new_asteroids);

        // --- DRAW ---
        clear_background(BLACK);

        particle_system.draw();

        let mut player_draw_color = player.color;
        if player.invulnerable {
             // Blink transparency effect
             player_draw_color.a = if (get_time() * 10.0) as i32 % 2 == 0 { 0.5 } else { 0.2 };
        }
        draw_poly_lines(player.pos.x, player.pos.y, player.sides, player.radius, player.rotation * 180. / std::f32::consts::PI, 2., player_draw_color);

        // Calculate direction vectors for visual effects
        let rotation_rad = player.rotation;
        let forward = vec2(rotation_rad.cos(), rotation_rad.sin());
        
        // --- 1. SHIP TIP HIGHLIGHT ---
        // The "nose" is at the perimeter in the direction of rotation
        let nose_pos = player.pos + forward * player.radius;
        draw_circle(nose_pos.x, nose_pos.y, 3.0, RED);

        // --- 2. ENGINE FLAME ---
        // Only draw if thrusting and player is active
        if is_key_down(KeyCode::Up) && player.active {
            // The flame comes out of the back
            // We use a slight offset so it looks like it's coming from the engine, not the center
            let flame_base = player.pos - forward * (player.radius * 0.8);
            
            // Randomize flame length for flickering effect
            let flickr_len = gen_range(10.0, 20.0);
            let flame_tip = flame_base - forward * flickr_len;
            
            // Draw a simple triangle for the flame
            // We need two side points at the base to make it a triangle
            // Right vector is forward rotated 90 degrees
            let right = vec2(forward.y, -forward.x);
            let side_width = 5.0;
            let p1 = flame_base + right * side_width;
            let p2 = flame_base - right * side_width;
            
            draw_triangle(p1, p2, flame_tip, ORANGE);
            draw_triangle(p1 + forward*2., p2 + forward*2., flame_tip + forward*5., YELLOW); // Inner flame
        }

        for a in asteroids.iter() {
             draw_poly_lines(a.pos.x, a.pos.y, a.sides, a.radius, 0., 2., GRAY);
        }

        for u in ufos.iter() {
            draw_poly_lines(u.pos.x, u.pos.y, 8, u.radius, 0., 2., RED);
            draw_line(u.pos.x - u.radius, u.pos.y, u.pos.x + u.radius, u.pos.y, 2., RED);
        }

        for b in bullets.iter() {
            let color = if b.owner == BulletOwner::Player { YELLOW } else { RED };
            draw_circle(b.pos.x, b.pos.y, 2., color);
        }

        draw_text(&format!("Level: {}", current_level_idx + 1), 20., 30., 20., WHITE);
        draw_text(&format!("Score: {}", score), 20., 55., 20., GREEN);
        draw_text(&format!("Lives: {}", lives), 20., 80., 20., RED);
        draw_text("Hyperspace: Shift | Quit: Esc", screen_width() - 300., 30., 20., BLUE);
        if settings.show_fps {
            draw_text(&format!("FPS: {}", get_fps()), screen_width() - 120., 55., 20., YELLOW);
        }
        
        // --- SCANNER UI ---
        if is_scanning {
             draw_text("SCANNING SECTOR...", 20., 110., 20., SKYBLUE);
             // Visualize the "work"
             let dots = (get_time() * 5.0) as i32 % 4;
             let bar = ".".repeat(dots as usize);
             draw_text(&bar, 220., 110., 20., SKYBLUE);
        } else if scan_message_timer > 0.0 {
             draw_text(&scan_message, 20., 110., 20., YELLOW);
             scan_message_timer -= get_frame_time();
        } else {
             draw_text("Press 'S' to Scan Sector", 20., 110., 15., GRAY);
        }

        next_frame().await
    }
}
