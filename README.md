# Rust Asteroids

A classic arcade shooter implemented in Rust using the [Macroquad](https://macroquad.rs/) game engine. Blast your way through asteroid fields, dodge alien UFOs, and survive as long as you can!

## Features

-   **Classic Gameplay**: Thrust, rotate, and shoot mechanics.
-   **Asteroid Splitting**: Large asteroids break into smaller debris when destroyed.
-   **Enemy UFOs**: Saucers appear to hunt you down with increasing frequency.
-   **Hyperspace**: Emergency teleport system (Shift) for tight spots.
-   **Level Progression**: 3 distinct levels with increasing difficulty, asteroid count, and speed.
-   **Score System**: Track your high score.
-   **Ship Customizer (Secret Tool)**: Live-edit ship color, size, and polygon sides.
-   **Ghost Respawn**: Temporary invulnerability with transparency after taking damage.
-   **Engine Flame + Tip Highlight**: Visual feedback for thrust and bullet origin.
-   **Background Scan Demo**: Multithreading example that awards bonus points.
-   **Title + Pause + Settings Menus**: Start screen, pause overlay, and configurable options.
-   **Particle Effects**: Explosions and muzzle flashes with async generation.

## Controls

| Action | Key |
| :--- | :--- |
| **Thrust** | `Up Arrow` |
| **Rotate** | `Left Arrow` / `Right Arrow` |
| **Shoot** | `Space` |
| **Hyperspace** | `Left Shift` |
| **Design Mode** | `D` |
| **Design: Change Color** | `C` |
| **Background Scan** | `S` |
| **Pause / Resume** | `P` |
| **Start Game** | `Enter` (Title screen) |
| **Settings** | `S` (Title / Pause) |
| **Quit** | `Esc` |
| **Restart** | `R` (on Game Over screen) |

### Settings Menu Controls

- **Volume**: Left / Right
- **Difficulty**: Up / Down
- **Show FPS**: F
- **Save / Return**: Enter (or Esc)

## Prerequisites

You need to have **Rust** and **Cargo** installed on your machine. If you haven't installed them yet, get them from [rustup.rs](https://rustup.rs/).

## Installation & Running

1.  Clone the repository (or download the source):
    ```bash
    git clone <repository-url>
    cd rust_asteroids
    ```

2.  Run the game:
    ```bash
    cargo run
    ```
    *Note: The first build might take a minute as it compiles dependencies.*

## Building a Standalone Executable

This project embeds its audio assets directly into the executable at compile time, so the release build is fully standalone.

```bash
cargo build --release
```

The executable will be located at:
`target/release/rust_asteroids.exe`

**Important:** The `assets/` folder is only needed at build time (for embedding). You do not need to ship it alongside the `.exe`.

## Settings File

The game loads settings from `settings.cfg` on startup (non-blocking), and writes changes back when you exit the Settings menu.
If the file is missing or invalid, defaults are used.

## Game Rules

-   **Objective**: Clear the screen of all asteroids and UFOs to advance to the next level.
-   **Lives**: Start with 3 lives. Gain an extra life every 3000 points.
-   **Scoring**:
    -   Asteroids: 100 points
    -   UFOs: 500 points
-   **Hyperspace**: Teleports you to a random location on the screen. **Warning**: You retain 0 velocity upon exit, but you might teleport directly into danger!
-   **Ghost Respawn**: After being hit, your ship respawns randomly and is temporarily invulnerable with a transparent blink.

## Testing & Instrumentation

The project includes unit tests for game logic, physics core, and configuration validation.

**Run All Tests:**
```bash
cargo test
```

**Logging:**
The game logs key events (Startup, Score Milestones, Respawns, Errors) to the console using `info!` and `warn!` macros.
-   **Startup**: Logs embedded asset loading and configuration validation.
-   **Runtime**: Logs background scan rewards, respawns, extra lives, and game-over reasons.
-   **Errors**: Logs invalid configurations and unknown sound requests.

## Project Structure

The code is modularized for readability and maintainability:

*   **`src/main.rs`**: Entry point and Main Game Loop.
*   **`src/components.rs`**: Game entities (`Player`, `Bullet`, `Asteroid`, `Ufo`) data structures.
*   **`src/config.rs`**: Level configuration and validation logic.
*   **`src/physics.rs`**: Pure functions for collision, movement wrapping, and scoring logic (contains Unit Tests).
*   **`src/level.rs`**: Logic for spawning levels.
*   **`src/resources.rs`**: Asset loading and management.

## Assets

Audio files are embedded directly into the executable:
-   `shoot.wav`
-   `bang.wav`
-   `warp.wav`
*The `assets/` folder is required only when compiling the project.*

## Built With

-   [Rust](https://www.rust-lang.org/)
-   [Macroquad](https://github.com/not-fl3/macroquad)
-   [Rand](https://crates.io/crates/rand)
