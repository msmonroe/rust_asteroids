# Rust Asteroids

A classic arcade shooter implemented in Rust using the [Macroquad](https://macroquad.rs/) game engine. Blast your way through asteroid fields, dodge alien UFOs, and survive as long as you can!

## Features

-   **Classic Gameplay**: Thrust, rotate, and shoot mechanics.
-   **Asteroid Splitting**: Large asteroids break into smaller debris when destroyed.
-   **Enemy UFOs**: Saucers appear to hunt you down with increasing frequency.
-   **Hyperspace**: Emergency teleport system (Shift) for tight spots.
-   **Level Progression**: 3 distinct levels with increasing difficulty, asteroid count, and speed.
-   **Score System**: Track your high score.

## Controls

| Action | Key |
| :--- | :--- |
| **Thrust** | `Up Arrow` |
| **Rotate** | `Left Arrow` / `Right Arrow` |
| **Shoot** | `Space` |
| **Hyperspace** | `Left Shift` |
| **Quit** | `Esc` |
| **Restart** | `R` (on Game Over screen) |

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

## Game Rules

-   **Objective**: Clear the screen of all asteroids and UFOs to advance to the next level.
-   **Lives**: Start with 3 lives. Gain an extra life every 3000 points.
-   **Scoring**:
    -   Asteroids: 100 points
    -   UFOs: 500 points
-   **Hyperspace**: Teleports you to a random location on the screen. **Warning**: You retain 0 velocity upon exit, but you might teleport directly into danger!

## Testing & Instrumentation

The project includes unit tests for game logic, physics core, and configuration validation.

**Run All Tests:**
```bash
cargo test
```

**Logging:**
The game logs key events (Startup, Assets, Score Milestones, Errors) to the console using `info!` and `warn!` macros.
-   **Startup**: Checks current working directory and asset availability.
-   **Runtime**: Logs "Game Over" reasons and Extra Lives.
-   **Errors**: Logs missing audio files or invalid configurations.

## Project Structure

The code is modularized for readability and maintainability:

*   **`src/main.rs`**: Entry point and Main Game Loop.
*   **`src/components.rs`**: Game entities (`Player`, `Bullet`, `Asteroid`, `Ufo`) data structures.
*   **`src/config.rs`**: Level configuration and validation logic.
*   **`src/physics.rs`**: Pure functions for collision, movement wrapping, and scoring logic (contains Unit Tests).
*   **`src/level.rs`**: Logic for spawning levels.
*   **`src/resources.rs`**: Asset loading and management.

## Assets

The game looks for audio files in an `assets/` folder:
-   `shoot.wav`
-   `bang.wav`
-   `warp.wav`

*If these files are missing, the game will still run silently.*

## Built With

-   [Rust](https://www.rust-lang.org/)
-   [Macroquad](https://github.com/not-fl3/macroquad)
-   [Rand](https://crates.io/crates/rand)
