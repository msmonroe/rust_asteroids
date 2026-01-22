use macroquad::prelude::*;
use ::rand::Rng;
use ::rand::thread_rng;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Clone, Debug)]
pub struct ParticleInit {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub color: Color,
    pub size: f32,
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub size: f32,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self { particles: Vec::new() }
    }

    pub fn spawn_batch(&mut self, batch: Vec<ParticleInit>) {
        self.particles.reserve(batch.len());
        for init in batch {
            self.particles.push(Particle {
                pos: init.pos,
                vel: init.vel,
                life: init.life,
                max_life: init.life.max(0.01),
                color: init.color,
                size: init.size,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            p.pos += p.vel * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let mut c = p.color;
            c.a *= (p.life / p.max_life).clamp(0.0, 1.0);
            draw_circle(p.pos.x, p.pos.y, p.size, c);
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpawnRequest {
    pub pos: Vec2,
    pub color: Color,
    pub count: usize,
    pub speed: f32,
    pub life: f32,
    pub size: f32,
}

pub struct ParticleSpawnBridge {
    tx: Sender<SpawnRequest>,
    rx: Receiver<Vec<ParticleInit>>,
}

impl ParticleSpawnBridge {
    pub fn new() -> Self {
        let (tx_req, rx_req) = mpsc::channel::<SpawnRequest>();
        let (tx_out, rx_out) = mpsc::channel::<Vec<ParticleInit>>();

        thread::spawn(move || {
            let mut rng = thread_rng();
            while let Ok(req) = rx_req.recv() {
                let count = clamp_count(req.count, 1, 250);
                let mut batch = Vec::with_capacity(count);
                for _ in 0..count {
                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                    let speed = rng.gen_range(req.speed * 0.5..req.speed);
                    let vel = vec2(angle.cos(), angle.sin()) * speed;
                    let life = rng.gen_range(req.life * 0.6..req.life);
                    let size = rng.gen_range(req.size * 0.6..req.size * 1.2);
                    batch.push(ParticleInit {
                        pos: req.pos,
                        vel,
                        life,
                        color: req.color,
                        size,
                    });
                }
                let _ = tx_out.send(batch);
            }
        });

        Self { tx: tx_req, rx: rx_out }
    }

    pub fn request(&self, req: SpawnRequest) {
        let _ = self.tx.send(req);
    }

    pub fn try_receive(&self) -> Option<Vec<ParticleInit>> {
        self.rx.try_recv().ok()
    }
}

pub fn clamp_count(value: usize, min: usize, max: usize) -> usize {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_count() {
        assert_eq!(clamp_count(0, 1, 10), 1);
        assert_eq!(clamp_count(5, 1, 10), 5);
        assert_eq!(clamp_count(50, 1, 10), 10);
    }

    #[test]
    fn test_particle_update_culls_dead() {
        let mut system = ParticleSystem::new();
        system.spawn_batch(vec![ParticleInit {
            pos: vec2(0.0, 0.0),
            vel: vec2(1.0, 0.0),
            life: 0.05,
            color: WHITE,
            size: 1.0,
        }]);
        system.update(0.1);
        assert_eq!(system.particles.len(), 0);
    }
}
