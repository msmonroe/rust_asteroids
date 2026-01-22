use macroquad::prelude::*;

/// Wraps a position vector around a boundary defined by width and height.
/// 
/// If an object goes off the left side, it appears on the right, and vice versa.
pub fn wrap_pos(pos: Vec2, width: f32, height: f32) -> Vec2 {
    let mut new_pos = pos;
    if new_pos.x > width { new_pos.x = 0.; }
    if new_pos.x < 0. { new_pos.x = width; }
    if new_pos.y > height { new_pos.y = 0.; }
    if new_pos.y < 0. { new_pos.y = height; }
    new_pos
}

/// Convenience function to wrap position around the global screen dimensions.
pub fn wrap_screen(pos: Vec2) -> Vec2 {
    wrap_pos(pos, screen_width(), screen_height())
}

/// Simple circle-based collision detection.
/// 
/// Returns true if the distance between `pos1` and `pos2` is less than the sum of their radii (`r1` + `r2`).
pub fn check_collision(pos1: Vec2, r1: f32, pos2: Vec2, r2: f32) -> bool {
    pos1.distance(pos2) < (r1 + r2)
}

/// Determines if the player has earned an extra life based on score milestones.
/// 
/// Returns `true` if score has crossed a 3000-point threshold since the last check.
/// Updates `last_score` to the new milestone.
pub fn check_extra_life(score: u32, last_score: &mut u32) -> bool {
    if score >= *last_score + 3000 {
        *last_score += 3000;
        true
    } else {
        false
    }
}

// --- UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_pos_inside() {
        let pos = vec2(100., 100.);
        let w = 800.;
        let h = 600.;
        let wrapped = wrap_pos(pos, w, h);
        assert_eq!(wrapped, pos);
    }

    #[test]
    fn test_wrap_pos_on_edges() {
        let w = 800.;
        let h = 600.;

        let pos_right_edge = vec2(w, 300.);
        let wrapped_right_edge = wrap_pos(pos_right_edge, w, h);
        assert_eq!(wrapped_right_edge.x, w);

        let pos_left_edge = vec2(0., 300.);
        let wrapped_left_edge = wrap_pos(pos_left_edge, w, h);
        assert_eq!(wrapped_left_edge.x, 0.);

        let pos_top_edge = vec2(300., 0.);
        let wrapped_top_edge = wrap_pos(pos_top_edge, w, h);
        assert_eq!(wrapped_top_edge.y, 0.);

        let pos_bottom_edge = vec2(300., h);
        let wrapped_bottom_edge = wrap_pos(pos_bottom_edge, w, h);
        assert_eq!(wrapped_bottom_edge.y, h);
    }

    #[test]
    fn test_wrap_pos_outside() {
        let w = 800.;
        let h = 600.;
        
        let pos_right = vec2(850., 100.);
        let wrapped_right = wrap_pos(pos_right, w, h);
        assert_eq!(wrapped_right.x, 0.);

        let pos_left = vec2(-50., 100.);
        let wrapped_left = wrap_pos(pos_left, w, h);
        assert_eq!(wrapped_left.x, w);

        let pos_bottom = vec2(100., 650.);
        let wrapped_bottom = wrap_pos(pos_bottom, w, h);
        assert_eq!(wrapped_bottom.y, 0.);

        let pos_top = vec2(100., -10.);
        let wrapped_top = wrap_pos(pos_top, w, h);
        assert_eq!(wrapped_top.y, h);
    }

    #[test]
    fn test_collision_detect() {
        let p1 = vec2(0., 0.);
        let r1 = 10.;
        let p2 = vec2(15., 0.); 
        let r2 = 10.;
        
        // dist is 15. radii sum is 20. 15 < 20 -> collision
        assert!(check_collision(p1, r1, p2, r2));

        let p3 = vec2(25., 0.);
        // dist is 25. radii sum is 20. 25 < 20 -> false
        assert!(!check_collision(p1, r1, p3, r2));
    }

    #[test]
    fn test_extra_life_logic() {
        let mut last_score = 0;
        assert!(!check_extra_life(2900, &mut last_score));
        assert_eq!(last_score, 0);

        assert!(check_extra_life(3000, &mut last_score));
        assert_eq!(last_score, 3000);

        assert!(!check_extra_life(5000, &mut last_score));
        assert_eq!(last_score, 3000);

        assert!(check_extra_life(6500, &mut last_score));
        assert_eq!(last_score, 6000);
    }

    #[test]
    fn test_extra_life_multiple_increments_over_time() {
        let mut last_score = 0;
        let mut lives = 3;

        let scores = [1000, 2999, 3000, 3100, 6000, 9000];
        for score in scores {
            if check_extra_life(score, &mut last_score) {
                lives += 1;
            }
        }

        assert_eq!(last_score, 9000);
        assert_eq!(lives, 6);
    }

    #[test]
    fn test_extra_life_single_step_even_on_big_jump() {
        let mut last_score = 0;
        assert!(check_extra_life(9000, &mut last_score));
        assert_eq!(last_score, 3000);
    }

    #[test]
    fn test_collision_edge_cases() {
        // Touching circles (distance == r1 + r2) -> Our check is <, so it should be false (no collision if just touching)
        // Adjust logic if "touching" counts as collision depending on desired physics.
        let p1 = vec2(0., 0.);
        let r1 = 1.0;
        let p2 = vec2(2.0, 0.);
        let r2 = 1.0;
        assert!(!check_collision(p1, r1, p2, r2)); // Distance is 2.0, sum is 2.0. 2.0 < 2.0 is False.

        // Slight overlap
        let p3 = vec2(1.9, 0.);
        assert!(check_collision(p1, r1, p3, r2));
    }
}
