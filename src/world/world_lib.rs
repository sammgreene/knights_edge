use sha2::{Digest, Sha256};
use bevy::prelude::*;

pub fn numeric_seed_from_string(input: &str) -> u64 {
    // 1. Hash the input string
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize(); // result is a GenericArray<u8, U32>, essentially 32 bytes

    // 2. Convert the first 8 bytes of the hash into a u64 numeric seed
    // We can use the first 8 bytes as a u64 seed for Pcg32
    // `try_into()` safely converts a slice of length 8 into an [u8; 8]
    let bytes: [u8; 8] = result[0..8].try_into().expect("Slice length is 8");
    u64::from_le_bytes(bytes)
}

pub fn get_points_in_radius(center_x: i32, center_y: i32, radius: u32) -> Vec<IVec2> {
    let mut points = Vec::new();
    // Calculate the squared radius to avoid using square root in the loop
    let radius_sq = (radius as i64).pow(2);

    // Determine the bounding box coordinates
    let x_min = center_x.saturating_sub(radius as i32);
    let x_max = center_x.saturating_add(radius as i32);
    let y_min = center_y.saturating_sub(radius as i32);
    let y_max = center_y.saturating_add(radius as i32);

    // Iterate through every integer coordinate in the bounding box
    for x in x_min..=x_max {
        for y in y_min..=y_max {
            // Calculate the squared distance from the center to the current point
            let dx = x - center_x;
            let dy = y - center_y;
            let distance_sq = (dx as i64).pow(2) + (dy as i64).pow(2); // Use i64 for larger radii

            // Check if the squared distance is less than or equal to the squared radius
            if distance_sq <= radius_sq {
                points.push(IVec2::new(x, y));
            }
        }
    }

    points
}