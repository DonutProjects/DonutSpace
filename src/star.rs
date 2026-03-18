use ::rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub brightness: f32,
}

#[derive(Debug)]
pub struct StarChunk {
    pub stars: Vec<Star>,
}

impl StarChunk {
    pub fn new(seed: i64) -> Self {
        let mut rng: ChaCha8Rng = SeedableRng::seed_from_u64(seed as u64);
        let mut stars = Vec::new();
        for _ in 0..super::STARS_PER_CHUNK {
            stars.push(Star {
                x: rng.gen_range(0.0..super::CHUNK_SIZE as f32),
                y: rng.gen_range(0.0..super::CHUNK_SIZE as f32),
                size: rng.gen_range(0.5..2.5),
                brightness: rng.gen_range(0.5..1.0),
            });
        }
        Self { stars }
    }
}