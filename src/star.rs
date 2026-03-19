use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub brightness: f32,
}

#[derive(Debug, Clone)]
pub struct Station {
    pub angle: f32,
    pub orbit_radius: f32,
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct Planet {
    pub orbit_radius: f32,
    pub angle: f32,
    pub size: f32,
    pub color: (f32, f32, f32),
    pub stations: Vec<Station>,
}

#[derive(Debug, Clone)]
pub struct StarSystem {
    pub x: f32,
    pub y: f32,
    pub star_radius: f32,
    pub star_color: (f32, f32, f32),
    pub planets: Vec<Planet>,
    pub is_trade_hub: bool,
}

#[derive(Debug)]
pub struct StarChunk {
    pub stars: Vec<Star>,
    pub systems: Vec<StarSystem>,
}

impl StarChunk {
    pub fn new(seed: i64, chunk_x: i32, chunk_y: i32) -> Self {
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

        let mut systems = Vec::new();
        let is_trade_hub_chunk = chunk_x == 0 && chunk_y == 0;
        let should_spawn_system = if is_trade_hub_chunk {
            true
        } else {
            rng.gen_bool(0.06)
        };

        if should_spawn_system {
            systems.push(generate_star_system(&mut rng, is_trade_hub_chunk));
        }

        Self { stars, systems }
    }
}

fn generate_star_system(rng: &mut ChaCha8Rng, is_trade_hub: bool) -> StarSystem {
    let planet_count = if is_trade_hub {
        rng.gen_range(5..=7)
    } else {
        rng.gen_range(3..=6)
    };

    let station_count = if is_trade_hub {
        5
    } else {
        rng.gen_range(2..=4)
    };

    let mut planets = Vec::new();
    for index in 0..planet_count {
        let orbit_radius = if is_trade_hub {
            90_000.0 + index as f32 * rng.gen_range(42_000.0..64_000.0)
        } else {
            60_000.0 + index as f32 * rng.gen_range(35_000.0..58_000.0)
        };
        let color = random_planet_color(rng);

        planets.push(Planet {
            orbit_radius,
            angle: rng.gen_range(0.0..std::f32::consts::TAU),
            size: rng.gen_range(1_800.0..4_500.0),
            color,
            stations: Vec::new(),
        });
    }

    let mut planet_indices: Vec<usize> = (0..planet_count).collect();
    planet_indices.shuffle(rng);

    for station_index in 0..station_count {
        let planet_index = planet_indices[station_index % planet_indices.len()];
        let station_orbit_radius = planets[planet_index].size + rng.gen_range(7_000.0..14_000.0);
        planets[planet_index].stations.push(Station {
            angle: rng.gen_range(0.0..std::f32::consts::TAU),
            orbit_radius: station_orbit_radius,
            size: if is_trade_hub {
                rng.gen_range(1_600.0..2_200.0)
            } else {
                rng.gen_range(1_100.0..1_700.0)
            },
        });
    }

    StarSystem {
        x: if is_trade_hub {
            super::CHUNK_SIZE as f32 * 0.5
        } else {
            rng.gen_range(220_000.0..(super::CHUNK_SIZE as f32 - 220_000.0))
        },
        y: if is_trade_hub {
            super::CHUNK_SIZE as f32 * 0.5
        } else {
            rng.gen_range(220_000.0..(super::CHUNK_SIZE as f32 - 220_000.0))
        },
        star_radius: if is_trade_hub {
            rng.gen_range(18_000.0..24_000.0)
        } else {
            rng.gen_range(12_000.0..18_000.0)
        },
        star_color: if is_trade_hub {
            (1.0, 0.88, 0.62)
        } else {
            random_star_color(rng)
        },
        planets,
        is_trade_hub,
    }
}

impl StarSystem {
    pub fn first_station_world_position(&self, chunk_x: i32, chunk_y: i32) -> Option<(f32, f32)> {
        let system_world_x = chunk_x as f32 * super::CHUNK_SIZE as f32 + self.x;
        let system_world_y = chunk_y as f32 * super::CHUNK_SIZE as f32 + self.y;

        for planet in &self.planets {
            let planet_world_x = system_world_x + planet.orbit_radius * planet.angle.cos();
            let planet_world_y = system_world_y + planet.orbit_radius * planet.angle.sin();

            if let Some(station) = planet.stations.first() {
                let station_world_x = planet_world_x + station.orbit_radius * station.angle.cos();
                let station_world_y = planet_world_y + station.orbit_radius * station.angle.sin();
                return Some((station_world_x, station_world_y));
            }
        }

        None
    }
}

fn random_star_color(rng: &mut ChaCha8Rng) -> (f32, f32, f32) {
    let presets = [
        (1.0, 0.95, 0.82),
        (0.95, 0.92, 1.0),
        (1.0, 0.84, 0.68),
        (0.85, 0.92, 1.0),
    ];
    presets[rng.gen_range(0..presets.len())]
}

fn random_planet_color(rng: &mut ChaCha8Rng) -> (f32, f32, f32) {
    let presets = [
        (0.39, 0.60, 0.82),
        (0.72, 0.50, 0.34),
        (0.48, 0.72, 0.56),
        (0.69, 0.64, 0.82),
        (0.82, 0.74, 0.47),
    ];
    presets[rng.gen_range(0..presets.len())]
}
