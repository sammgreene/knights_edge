// get random asset from directory
// load_random_from_dir("grass_variants/grass_full")
use bevy::prelude::*;
use std::fs;
use std::path::Path;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub fn load_random_asset_from_dir(dir: &str, coord: (i32, i32), asset_server: &AssetServer) -> Handle<Image> {
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

use std::path::{PathBuf};

/// Search recursively in "assets" for a file with the given name.
/// If not found, returns a blank 1x1 white image handle.
pub fn load_asset(asset_name: &str, asset_server: &AssetServer) -> Handle<Image> {
    let assets_path = Path::new("assets");

    if let Some(found_path) = find_file_recursive(assets_path, asset_name) {
        return asset_server.load(found_path);
    }
    return Handle::<Image>::default();
}

/// Recursively search a directory for a file with the given name
fn find_file_recursive(dir: &Path, target_file: &str) -> Option<PathBuf> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(found) = find_file_recursive(&path, target_file) {
                    return Some(found);
                }
            } else if path.file_name().and_then(|n| n.to_str()) == Some(target_file) {
                return Some(path);
            }
        }
    }
    None
}
