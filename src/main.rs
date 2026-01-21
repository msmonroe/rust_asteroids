mod components;
mod config;
mod resources;
mod physics;
mod level;

use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::path::Path;

use components::*;
use config::get_levels;
use resources::Resources;
use physics::*;
use level::load_level;

/// Main Entry Point
#[macroquad::main("Rust Asteroids")]
async fn main() {
    info!("Starting Rust Asteroids");
    
    // Debug: Check Working Directory and Assets
    if let Ok(cwd) = std::env::current_dir() {
        info!("Current Working Directory: {:?}", cwd);
        if Path::new("assets/shoot.wav").exists() {
             info!("Asset check: 'assets/shoot.wav' FOUND.");
        } else {
             error!("Asset check: 'assets/shoot.wav' NOT FOUND. Make sure you run from the project root!");
        }
    }

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
    let resources = Resources::load().await;

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
    };
    
    let mut design_mode = false;
    let available_colors = [WHITE, RED, ORANGE, YELLOW, GREEN, SKYBLUE, BLUE, PURPLE, PINK];
    let mut current_color_idx = 0;
    
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut asteroids: Vec<Asteroid> = Vec::new();
    let mut ufos: Vec<Ufo> = Vec::new();
    
    let mut game_over = false;
    let mut game_won = false;

    // Initial Level Load
    load_level(&levels[current_level_idx], &mut asteroids, &mut ufos);

    loop {
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
                player.pos = vec2(screen_width() / 2., screen_height() / 2.);
                player.vel = vec2(0., 0.);
                player.active = true;
                bullets.clear();
                load_level(&levels[current_level_idx], &mut asteroids, &mut ufos);
            }

            if is_key_pressed(KeyCode::Escape) {
                break;
            }

            next_frame().await;
            continue;
        }

        // --- DESIGN MODE (Secret Tool) ---
        if is_key_pressed(KeyCode::D) {
            design_mode = !design_mode;
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

        // --- SPAWN LOGIC (UFO) ---
        if ufos.len() < 1 && gen_range(0.0, 1.0) < level_cfg.ufo_spawn_chance {
            resources.play("warp"); // Sound cue for UFO entry
            ufos.push(Ufo {
                pos: vec2(0., gen_range(0., screen_height())), 
                vel: vec2(level_cfg.ufo_speed, 0.),
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
            } else {
                player.pos = vec2(screen_width() / 2., screen_height() / 2.);
                player.vel = vec2(0., 0.);
                bullets.clear();
                load_level(&levels[current_level_idx], &mut asteroids, &mut ufos);
            }
        }

        // --- INPUT ---
        if is_key_pressed(KeyCode::Escape) {
            break;
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
        }

        // HYPERSPACE (Shift Key)
        if is_key_pressed(KeyCode::LeftShift) {
            resources.play("warp");
            player.pos = vec2(gen_range(0., screen_width()), gen_range(0., screen_height()));
            player.vel = vec2(0., 0.); // Reset velocity for safety
        }

        // --- PHYSICS & UPDATES ---

        // Player
        player.pos += player.vel;
        player.vel *= 0.98; // Friction
        player.pos = wrap_screen(player.pos);

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
                ufo.vel.y = gen_range(-1.0, 1.0) * level_cfg.ufo_speed;
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
                    }
                }
            }

            // Bullet vs Player
            if bullet.active && bullet.owner == BulletOwner::Ufo {
                 if check_collision(bullet.pos, 0., player.pos, player.radius) {
                     resources.play("bang");
                     lives -= 1;
                     bullet.active = false;
                     if lives <= 0 {
                         game_over = true;
                         warn!("Game Over: Player hit by UFO bullet. Lives: 0");
                     } else {
                         warn!("Player hit by UFO bullet. Lives remaining: {}", lives);
                         player.pos = vec2(screen_width() / 2., screen_height() / 2.);
                         player.vel = vec2(0., 0.);
                     }
                 }
            }
        }

        // Physical Collisions (Ship vs Asteroid / UFO vs Asteroid / Ship vs UFO)
        for asteroid in asteroids.iter_mut() {
            if !asteroid.active { continue; }
            
            // Player vs Asteroid
            if check_collision(player.pos, player.radius, asteroid.pos, asteroid.radius) {
                resources.play("bang");
                lives -= 1;
                asteroid.active = false; // Destroy asteroid on impact

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
                     player.pos = vec2(screen_width() / 2., screen_height() / 2.);
                     player.vel = vec2(0., 0.);
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
                }
            }
        }
        
        // Player vs UFO
        for ufo in ufos.iter_mut() {
            if ufo.active && check_collision(player.pos, player.radius, ufo.pos, ufo.radius) {
                ufo.active = false; // Destroy UFO
                resources.play("bang");
                lives -= 1;
                if lives <= 0 {
                    game_over = true;
                    warn!("Game Over: Player hit by UFO. Lives: 0");
                } else {
                    warn!("Player hit by UFO. Lives remaining: {}", lives);
                    player.pos = vec2(screen_width() / 2., screen_height() / 2.);
                    player.vel = vec2(0., 0.);
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

        draw_poly_lines(player.pos.x, player.pos.y, player.sides, player.radius, player.rotation * 180. / std::f32::consts::PI, 2., player.color);

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

        next_frame().await
    }
}
