// get random asset from directory
// load_random_from_dir("grass_variants/grass_full")
use bevy::prelude::*;
use std::fs;
use std::path::Path;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub fn load_random_from_dir(dir: &str, coord: (i32, i32), asset_server: &AssetServer) -> Handle<Image> {
    // loads a random asset from the given directory (dependent on world vegetation noise)
    let true_path = Path::new("assets");
    let variants = fs::read_dir(true_path.join(dir)).unwrap().count() / 2;

    let hash = coord_hash(coord);
    let choice = (hash as usize) % variants;

    let file_name_prefix = "/".to_owned() + dir.split("/").last().unwrap_or("test") + "_";
    return asset_server.load(format!("{}{}{}.png", dir, file_name_prefix, choice));
}

fn coord_hash(coord: (i32, i32)) -> u64 {
    let mut hasher = DefaultHasher::new();
    coord.hash(&mut hasher);
    hasher.finish()
}