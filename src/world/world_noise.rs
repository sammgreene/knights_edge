use bevy::prelude::*;
use fastnoise_lite::{FastNoiseLite, NoiseType};
use crate::world::world_lib::numeric_seed_from_string;

#[derive(Resource)]
pub struct WorldNoise {
    pub seed: u64,
    pub altitude: FastNoiseLite,
    // pub vegetation: FastNoiseLite,
    pub temp: FastNoiseLite,
    pub moisture: FastNoiseLite,
    // pub mountain_mask: FastNoiseLite,
}
impl WorldNoise {
    pub fn default(seed: &str) -> Self {
        let seed = numeric_seed_from_string(seed);

        let mut altitude = FastNoiseLite::new();
        altitude.set_seed(Some(seed as i32));
        altitude.set_noise_type(Some(NoiseType::OpenSimplex2));
        altitude.set_frequency(Some(0.02)); // VERY large features
        altitude.octaves = 10;
        // experimental
        altitude.set_frequency(Some(0.04)); // VERY large features
        altitude.octaves = 2;

        // let mut vegetation = FastNoiseLite::new();
        // vegetation.set_seed(Some((generator.seed + 1337) as i32)); // offset for variation
        // vegetation.set_noise_type(Some(NoiseType::Perlin));
        // vegetation.set_frequency(Some(0.8)); // high frequency

        let mut temp = FastNoiseLite::new();
        temp.set_seed(Some((seed + 1337) as i32)); // offset for variation
        temp.set_noise_type(Some(NoiseType::Perlin));
        temp.set_frequency(Some(0.01));

        let mut moisture = FastNoiseLite::new();
        moisture.set_seed(Some((seed + 1337) as i32)); // offset for variation
        moisture.set_noise_type(Some(NoiseType::Perlin));
        moisture.set_frequency(Some(0.01));

        Self {
            seed,
            altitude,
            // vegetation,
            temp,
            moisture,
        }
    }

    pub fn get_climate(&self, x: f32, y: f32) -> (f32, f32, f32) {
        let altitude = self.get_altitude(x, y); // already [0,1]


        let raw = self.temp.get_noise_2d(x, y);
        let mut temp = raw * 0.5 + 0.5; // [0,1]
        temp = temp.powf(2.0);

        
        let raw = self.moisture.get_noise_2d(x, y);
        let mut moisture = raw * 0.5 + 0.5; // [0,1]

        (altitude, temp, moisture)
    }

    pub fn get_altitude(&self, x: f32, y: f32) -> f32 {
        // Base raw noise
        let raw = self.altitude.get_noise_2d(x, y); // ~[-1, 1]
        
        // Normalize to [0,1]
        let mut altitude = raw * 0.5 + 0.5;

        // Add subtle mid-world bias
        // This moves most of the world toward 0.5 without killing variation
        altitude = 0.5 + (altitude - 0.5) * 0.3; // 0.3 = strength of variation around 0.5

        // Shape mountains and oceans
        // Powf >1 makes mountains steeper, <1 makes oceans flatter
        altitude = altitude.powf(1.5);  // big, smooth mountains
        altitude = altitude * 0.9 + 0.05; // shrink oceans slightly, raise overall terrain

        // Clamp to [0,1] just in case
        altitude.clamp(0.0, 1.0)
    }

    pub fn white_noise_2d(&self, x: i32, y: i32) -> f32 {
        // Combine coordinates
        let mut h = self.seed
            ^ (x as u64).wrapping_mul(0x9E3779B185EBCA87)
            ^ (y as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);

        // MurmurHash3 finalizer (64-bit avalanche)
        h ^= h >> 33;
        h = h.wrapping_mul(0xFF51AFD7ED558CCD);
        h ^= h >> 33;
        h = h.wrapping_mul(0xC4CEB9FE1A85EC53);
        h ^= h >> 33;

        // Convert to uniform [0,1)
        let v = (h >> 40) as u32; // top 24 bits
        v as f32 / (1u32 << 24) as f32
    }
}