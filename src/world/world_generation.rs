use std::collections::HashMap;
use bevy::prelude::*;

use crate::world::{world_lib, world_noise};

// Constants
pub const CHUNK_SIZE: usize = 16; // world units
pub const RENDER_DISTANCE: u32 = 4; // chunks

fn world_to_chunk_coord(x: i32, y: i32) -> IVec2 {
    // World chunk coords
    IVec2::new(
        if x >= 0 { x / CHUNK_SIZE as i32 } else { (x + 1 - CHUNK_SIZE as i32) / CHUNK_SIZE as i32 },
        if y >= 0 { y / CHUNK_SIZE as i32 } else { (y + 1 - CHUNK_SIZE as i32) / CHUNK_SIZE as i32 },
    )
}

fn world_to_chunk_position(x: i32, y: i32) -> IVec2 {
    // Local position inside chunk
    IVec2::new(
        ((x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32, 
        ((y % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32,
    )
}
// WorldMap Resource
// contains a hashmap of all chunk entities
// indexed by chunk coordinate
// 
// New chunks are regiserted automatically by register_new_chunks
#[derive(Resource, Default)]
pub struct WorldMap {
    pub chunks: HashMap<IVec2, Entity>
}
pub fn get_tile_at(
    world_map: Res<WorldMap>,
    chunks_query: Query<&Chunk>,
    x: i32,
    y: i32
) -> Option<WorldTile> {
    let chunk_coord = world_to_chunk_coord(x, y);
    let chunk_position = world_to_chunk_position(x, y);

    if let Some(chunk_entity) = world_map.chunks.get(&chunk_coord) {
        if let Ok(chunk) = chunks_query.get(*chunk_entity) {
            return Some(chunk.tile_data[chunk_position.x as usize][chunk_position.y as usize]);
        }
    }
    None
}

// Components
#[derive(Clone, Copy, Component, PartialEq, Debug)]
pub enum WorldTile {
    Grass,
    Snow,
    Water
}
#[derive(Clone, Copy, Component, PartialEq)]
pub enum Foliage {
    None, // most common case
    // Rock,
    Tree(TreeType),
    Bush
}
#[derive(Clone, Copy, PartialEq)]
pub enum TreeType {
    Oak,
}

// Components
#[derive(Component)]
pub struct Chunk {
    pub coord: IVec2,
    pub tile_data: [[WorldTile; CHUNK_SIZE]; CHUNK_SIZE],
    pub foliage_data: [[Foliage; CHUNK_SIZE]; CHUNK_SIZE],
    // pub biome_data: [[Biome; CHUNK_SIZE]; CHUNK_SIZE]
}

// Systems

// Calls spawn and gen chunks only on a nearby spiral of chunks
// This is where RENDER_DISTANCE is used
pub fn load_near_chunks(
    world_map: Res<WorldMap>,
    player_positions: Query<&Transform, With<crate::player::Player>>,
    noise: Res<world_noise::WorldNoise>,
    mut commands: Commands
) {
    for player_position in player_positions.iter() {
        let player_chunk_x = (player_position.translation.x / CHUNK_SIZE as f32).floor() as i32;
        let player_chunk_y = (player_position.translation.y / CHUNK_SIZE as f32).floor() as i32;

        let nearby_chunk_coords = world_lib::get_points_in_radius(player_chunk_x, player_chunk_y, RENDER_DISTANCE);
        
        let mut chunks_loaded_this_frame = 0;
        // Check 3x3 area around player for any chunks that need to be generated
        for chunk_coord in nearby_chunk_coords {
            if chunks_loaded_this_frame >= 1 {
                break;
            }
            if !world_map.chunks.contains_key(&chunk_coord) {
                commands.spawn((
                    gen_chunk(chunk_coord, &noise),

                    Transform::from_xyz(
                        chunk_coord.x as f32 * CHUNK_SIZE as f32,
                        chunk_coord.y as f32 * CHUNK_SIZE as f32,
                        0.,
                    ),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ));
                chunks_loaded_this_frame += 1;
            }
        }
    }
}

// When spawning a chunk:
fn gen_chunk(
    coord: IVec2, 
    noise: &Res<world_noise::WorldNoise>
) -> Chunk {
    // ### Generation RoadMap
    // 1. generate altitude, temperature, and moisture noises
    //    - some day maybe foliage low freq
    //    - also needs onion skin for these
    // 
    // 2. determine biome from said noise 
    //    - as a result of next step this needs onion skin as well
    // 
    // 3. update blocks from biome data 
    //    - these need onion skin for block transitions
    // 
    // 4. update foliage from biome, foliage amplifier, and foliage white noise
    //   - high freq for amplifier (~0.09) 
    //   - does not need onion skin for amplifier or white noise
    //

    // Actual chunk data
    let mut tile_data = [[WorldTile::Grass; CHUNK_SIZE]; CHUNK_SIZE];
    let mut foliage_data = [[Foliage::None; CHUNK_SIZE]; CHUNK_SIZE];
    let mut biome_data = [[Biome::Forest; CHUNK_SIZE]; CHUNK_SIZE];

    let chunk_origin_x = coord.x * CHUNK_SIZE as i32;
    let chunk_origin_y = coord.y * CHUNK_SIZE as i32;

    for chunk_x in 0..CHUNK_SIZE {
        for chunk_y in 0..CHUNK_SIZE {
            let world_x = chunk_origin_x + chunk_x as i32;
            let world_y = chunk_origin_y + chunk_y as i32;

            // 1. Create biome map
            let (altitude, temperature, moisture) = noise.get_climate(world_x as f32, world_y as f32);
            let biome = biome_lookup(altitude, temperature, moisture);

            let tile = tile_from_biome_data(biome);

            biome_data[chunk_x][chunk_y] = biome;
            tile_data[chunk_x][chunk_y] = tile;
        }
    }

    for chunk_x in 0 .. CHUNK_SIZE { // 0 - 15 inclusive
        for chunk_y in 0 .. CHUNK_SIZE {
            let world_x = coord.x * CHUNK_SIZE as i32 + chunk_x as i32;
            let world_y = coord.y * CHUNK_SIZE as i32 + chunk_y as i32;

            let vegetation_value = noise.get_vegetation(world_x, world_y);
            let white_noise = noise.white_noise_2d(world_x, world_y);

            let biome = biome_data[chunk_x][chunk_y];

            foliage_data[chunk_x][chunk_y] = foliage_from_biome_and_vegetation_amp(biome, vegetation_value, white_noise);
        }
    }

    Chunk { coord, tile_data, foliage_data, /* biome_data */ }
}

pub fn despawn_distant_chunks(
    chunks_query: Query<(Entity, &Chunk), With<Chunk>>,
    children_query: Query<&Children>, // <-- add this!
    player_transform: Single<&Transform, With<crate::player::Player>>,
    mut world_map: ResMut<WorldMap>,
    mut commands: Commands,
) {
    let player_pos = player_transform;

    let player_chunk = IVec2 {
        x: (player_pos.translation.x / CHUNK_SIZE as f32).floor() as i32,
        y: (player_pos.translation.y / CHUNK_SIZE as f32).floor() as i32,
    };

    let render_dist_squared: i32 = (RENDER_DISTANCE as i32).pow(2);

    for (entity, chunk) in chunks_query.iter() {
        if chunk.coord.distance_squared(player_chunk) > render_dist_squared {
            despawn_recursive(&mut commands, entity, &children_query);
            world_map.chunks.remove(&chunk.coord);
        }
    }
}

fn despawn_recursive(commands: &mut Commands, entity: Entity, query: &Query<&Children>) {
    if let Ok(children) = query.get(entity) {
        for child in children.iter() {
            despawn_recursive(commands, child, query);
        }
    }
    commands.entity(entity).despawn();
}

pub fn register_new_chunks(
    mut world_map: ResMut<WorldMap>,
    chunks_query: Query<(Entity, &Chunk), Added<Chunk>>
) {
    for (chunk_entity, chunk) in chunks_query {
        world_map.chunks.insert(chunk.coord, chunk_entity);   
    }
}


fn foliage_from_biome_and_vegetation_amp(biome: Biome, vegetation_value: f32, white_noise: f32) -> Foliage {
    match biome {
        Biome::Forest => forest_foliage(vegetation_value, white_noise),
        _ => Foliage::None
    }
}

fn forest_foliage(vegetation_value: f32, white_noise: f32) -> Foliage {
    if vegetation_value > 0.3 && white_noise > 0.25 {
        Foliage::Bush
    }
    else if vegetation_value < 0.2 && white_noise > 0.9 {
        Foliage::Tree(TreeType::Oak)
    }
    else { Foliage::None }
}

#[derive(Clone, Copy, Debug)]
pub enum Biome {
    Ocean,
    Beach,
    Forest,
    Desert,
    Snow,
}

struct BiomeDef {
    biome: Biome,

    // Ideal climate center
    temp_center: f32,
    moist_center: f32,

    // How tolerant the biome is
    temp_weight: f32,
    moist_weight: f32,
}

const BIOMES: &[BiomeDef] = &[
    BiomeDef {
        biome: Biome::Forest,
        temp_center: 0.5,
        moist_center: 0.5,
        temp_weight: 0.1,
        moist_weight: 0.1,
    },
    BiomeDef {
        biome: Biome::Desert,
        temp_center: 0.8,
        moist_center: 0.1,
        temp_weight: 1.0,
        moist_weight: 1.2,
    },
    BiomeDef {
        biome: Biome::Snow,
        temp_center: 0.05,
        moist_center: 0.25,
        temp_weight: 1.5,   // was 1.2
        moist_weight: 1.5, // was 0.8
    }
];

const SEA_LEVEL: f32 = 0.30;
const BEACH_BAND: f32 = 0.002;
const SNOW_ALTITUDE: f32 = 0.95;

#[inline(always)]
fn biome_lookup(a: f32, t: f32, m: f32) -> Biome {
    // Ocean
    if a < SEA_LEVEL {
        return Biome::Ocean;
    }

    // Beach
    if a < SEA_LEVEL + BEACH_BAND {
        return Biome::Beach;
    }

    // High-altitude snow override
    if a > SNOW_ALTITUDE {
        return Biome::Snow;
    }

    // Otherwise: climate-space lookup
    climate_biome_lookup(t, m)
}

#[inline(always)]
fn climate_biome_lookup(temp: f32, moist: f32) -> Biome {
    let mut best_score = f32::INFINITY;
    let mut best_biome = Biome::Forest; // default fallback

    for def in BIOMES {
        let dt = temp - def.temp_center;
        let dm = moist - def.moist_center;

        // Weighted squared distance (cheap + smooth)
        let score =
            dt * dt * def.temp_weight +
            dm * dm * def.moist_weight;

        // Single predictable branch
        if score < best_score {
            best_score = score;
            best_biome = def.biome;
        }
    }

    best_biome
}

fn tile_from_biome_data(biome: Biome) -> WorldTile {
    match biome {
        Biome::Ocean => WorldTile::Water,
        Biome::Forest => WorldTile::Grass,
        Biome::Snow => WorldTile::Snow,
        // Biome::Beach => WorldTile::Sand,
        // Biome::Desert => WorldTile::Sand,
        _ => WorldTile::Grass
    }
}