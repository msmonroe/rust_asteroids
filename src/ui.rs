use macroquad::prelude::*;
use crate::settings::{Settings, Difficulty};

/// Draw the title screen with basic controls.
pub fn draw_title_screen() {
    clear_background(BLACK);
    draw_text("RUST ASTEROIDS", screen_width() / 2. - 140., screen_height() / 2. - 80., 40., WHITE);
    draw_text("Press Enter to Start", screen_width() / 2. - 130., screen_height() / 2. - 20., 24., GRAY);
    draw_text("Press S for Settings", screen_width() / 2. - 120., screen_height() / 2. + 10., 20., GRAY);
    draw_text("Press Esc to Quit", screen_width() / 2. - 115., screen_height() / 2. + 40., 20., GRAY);
}

/// Draw the pause overlay.
pub fn draw_pause_screen() {
    draw_text("PAUSED", screen_width() / 2. - 60., screen_height() / 2. - 60., 36., WHITE);
    draw_text("Press P to Resume", screen_width() / 2. - 120., screen_height() / 2. - 20., 22., GRAY);
    draw_text("Press S for Settings", screen_width() / 2. - 120., screen_height() / 2. + 10., 20., GRAY);
    draw_text("Press Esc for Title", screen_width() / 2. - 115., screen_height() / 2. + 40., 20., GRAY);
}

/// Draw the settings screen.
pub fn draw_settings_screen(settings: &Settings) {
    clear_background(BLACK);
    draw_text("SETTINGS", screen_width() / 2. - 80., 60., 36., WHITE);

    let difficulty = match settings.difficulty {
        Difficulty::Easy => "Easy",
        Difficulty::Normal => "Normal",
        Difficulty::Hard => "Hard",
    };

    draw_text("Use Left/Right to change values", 40., 120., 20., GRAY);
    draw_text("Use Up/Down to change difficulty", 40., 145., 20., GRAY);
    draw_text("Press F to toggle FPS display", 40., 170., 20., GRAY);
    draw_text("Press Enter to Save/Return", 40., 195., 20., GRAY);

    draw_text(&format!("Volume: {:.2}", settings.volume), 60., 260., 24., WHITE);
    draw_text(&format!("Difficulty: {}", difficulty), 60., 290., 24., WHITE);
    draw_text(&format!("Show FPS: {}", if settings.show_fps { "On" } else { "Off" }), 60., 320., 24., WHITE);
}
