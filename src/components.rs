use macroquad::prelude::*;

/// Enum indicating who owns a bullet (player or enemy).
#[derive(PartialEq, Debug)]
pub enum BulletOwner {
    Player,
    Ufo,
}

/// Represents the player's ship.
pub struct Player {
    /// Position vector (x, y)
    pub pos: Vec2,
    /// Velocity vector (dx, dy)
    pub vel: Vec2,
    /// Rotation angle in radians
    pub rotation: f32,
    /// Collision radius
    pub radius: f32,
    /// Whether the player is active (alive)
    pub active: bool,
    /// Number of sides for the ship polygon (3 = Triangle)
    pub sides: u8,
    /// Color of the ship
    pub color: Color,
}

/// Represents a projectile fired by an entity.
pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    /// Time in seconds before the bullet disappears
    pub lifetime: f32,
    pub active: bool,
    pub owner: BulletOwner,
}

/// Represents an asteroid obstacle.
#[derive(Clone)]
pub struct Asteroid {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub active: bool,
    /// Number of sides for drawing the polygon (visual variance)
    pub sides: u8,
}

/// Represents an enemy UFO.
pub struct Ufo {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub active: bool,
    /// Time accumulator for shooting cooldown
    pub shoot_timer: f32,
    /// Time accumulator for changing movement direction
    pub change_dir_timer: f32,
}
