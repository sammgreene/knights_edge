use bevy::prelude::*;
use fastnoise_lite::{FastNoiseLite, NoiseType};

use crate::{ZLayers, world::world_lib::*};

// Constants
const CHUNK_SIZE: usize = 16;

// Resources
#[derive(Resource)]
pub struct WorldGenerator {
    pub seed: u64,
    pub settings: WorldSettings,
}
impl WorldGenerator {
    // pub fn default() -> Self {
    //     Self {
    //         seed: numeric_seed_from_string("cyan"),
    //         settings: WorldSettings::default(),
    //     }
    // }
    pub fn with_seed(seed_str: &str) -> Self {
        Self {
            seed: numeric_seed_from_string(seed_str),
            settings: WorldSettings::default()
        }
    }
}
#[derive(Resource)]
pub struct WorldNoise {
    pub altitude: FastNoiseLite,
    pub vegetation: FastNoiseLite,
    pub temp: FastNoiseLite,
    pub moisture: FastNoiseLite,
}

pub struct WorldSettings {
    pub world_type: WorldType,
}
impl WorldSettings {
    pub fn default() -> Self {
        Self {
            world_type: WorldType::Featured,
        }
    }
}
pub enum WorldType {
    Featured,
    // Plains
}
#[derive(Clone, Copy, Component, PartialEq)]
pub enum WorldTiles {
    Grass,
    // Stone,
    Water
}

#[derive(Resource, Default)]
pub struct LoadedChunks {
    pub coords: std::collections::HashSet<IVec2>,
}

// Components
#[derive(Component)]
pub struct Chunk {
    pub coord: IVec2,
    data: [[WorldTiles; CHUNK_SIZE + 2]; CHUNK_SIZE + 2] // + 2 is for overflow tiles
}

// Systems
pub fn print_world_generator_info(
    generator: Res<WorldGenerator>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyI) {
        println!("World Generator Seed: {}", generator.seed);
        match generator.settings.world_type {
            WorldType::Featured => println!("World Type: Featured"),
            // WorldType::Plains => println!("World Type: Plains"),
        }
    }
}

pub fn check_to_load_chunks(
    mut loaded_chunks: ResMut<LoadedChunks>,
    player_positions: Query<&Transform, With<crate::player::Player>>,
    noise: Res<WorldNoise>,
    mut commands: Commands
) {
    for player_position in player_positions.iter() {
        let player_chunk_x = (player_position.translation.x / CHUNK_SIZE as f32).floor() as i32;
        let player_chunk_y = (player_position.translation.y / CHUNK_SIZE as f32).floor() as i32;

        let nearby_chunks = get_points_in_radius(player_chunk_x, player_chunk_y, 2);
        
        let mut chunks_loaded_this_frame = 0;
        // Check 3x3 area around player for any chunks that need to be generated
        for chunk_coord in nearby_chunks {
            if chunks_loaded_this_frame >= 3 {
                break;
            }
            if !loaded_chunks.coords.contains(&chunk_coord) {
                spawn_and_gen_chunk(chunk_coord, &mut commands, &mut loaded_chunks, &noise);
                chunks_loaded_this_frame += 1;
            }
        }
    }
}

fn get_points_in_radius(center_x: i32, center_y: i32, radius: u32) -> Vec<IVec2> {
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

// When spawning a chunk:
fn spawn_and_gen_chunk(
    coord: IVec2, 
    commands: &mut Commands, 
    loaded_chunks: &mut ResMut<LoadedChunks>,
    noise: &Res<WorldNoise>
) {
    // Example precompute chunk vegetation noise:
    let mut vegetation_chunk = [[0f32; CHUNK_SIZE + 2]; CHUNK_SIZE + 2];
    let mut temp_chunk = [[0f32; CHUNK_SIZE + 2]; CHUNK_SIZE + 2];
    let mut altitude_chunk = [[0f32; CHUNK_SIZE + 2]; CHUNK_SIZE + 2];
    let mut moisture = [[0f32; CHUNK_SIZE + 2]; CHUNK_SIZE + 2];

    for x in 0..CHUNK_SIZE + 2 {
        for y in 0..CHUNK_SIZE + 2 {
            let world_x = coord.x * CHUNK_SIZE as i32 + x as i32 - 1; // offset by 1
            let world_y = coord.y * CHUNK_SIZE as i32 + y as i32 - 1; // offset by 1
            vegetation_chunk[x][y] = noise.vegetation.get_noise_2d(world_x as f32, world_y as f32);
            temp_chunk[x][y] = noise.temp.get_noise_2d(world_x as f32, world_y as f32);
            altitude_chunk[x][y] = noise.altitude.get_noise_2d(world_x as f32, world_y as f32);
            moisture[x][y] = noise.moisture.get_noise_2d(world_x as f32, world_y as f32);
        }
    }
    
    let mut data = [[WorldTiles::Grass; CHUNK_SIZE + 2]; CHUNK_SIZE + 2];
    for (row_idx, row) in data.iter_mut().enumerate() {
        for (tile_idx, tile) in row.iter_mut().enumerate() {
            if altitude_chunk[row_idx][tile_idx] < 0.0 {
                *tile = WorldTiles::Water;
            }
        }
    }

    commands.spawn((
        Chunk { coord, data: data },
        Transform::from_xyz(
            coord.x as f32 * CHUNK_SIZE as f32,
            coord.y as f32 * CHUNK_SIZE as f32,
            ZLayers::World as i32 as f32,
        ),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
    loaded_chunks.coords.insert(coord);
    info!("Spawning {}", coord)
}

// Generates tiles for any newly created chunks on the frame they are added
pub fn spawn_chunk_tiles_for_new_chunks(
    mut commands: Commands,
    new_chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, chunk) in new_chunks.iter() {
        commands.entity(entity).with_children(|parent| {

            for x in 1..CHUNK_SIZE+1 {
                for y in 1..CHUNK_SIZE+1 {
                    let world_x = chunk.coord.x * CHUNK_SIZE as i32 + x as i32;
                    let world_y = chunk.coord.y * CHUNK_SIZE as i32 + y as i32;

                    let (sprite_image, rotation) = match chunk.data[x][y] {
                        WorldTiles::Grass => (
                            crate::asset_loading::load_random_from_dir(
                                "grass_variants/grass_full",
                                (world_x, world_y),
                                &asset_server,
                            ),
                            Quat::IDENTITY,
                        ),

                        WorldTiles::Water => {
                            if let Some(edge) = water_edge(chunk, x, y) {
                                if edge == WaterEdge::TopLeft {
                                    (
                                        crate::asset_loading::load_random_from_dir(
                                            "water/water_edge_top/water_edge_top_left_grass",
                                            (world_x, world_y),
                                            &asset_server,
                                        ),
                                        edge_rotation(edge),
                                    )
                                }
                                else if edge == WaterEdge::TopRight {
                                    (
                                        crate::asset_loading::load_random_from_dir(
                                            "water/water_edge_top/water_edge_top_right_grass",
                                            (world_x, world_y),
                                            &asset_server,
                                        ),
                                        edge_rotation(edge),
                                    )
                                }
                                else if edge == WaterEdge::Top {
                                    (
                                        crate::asset_loading::load_random_from_dir(
                                            "water/water_edge_top/water_edge_top_grass",
                                            (world_x, world_y),
                                            &asset_server,
                                        ),
                                        edge_rotation(edge) // Quat::IDENTITY,
                                    )
                                }
                                else if edge == WaterEdge::BottomLeft {
                                    (
                                        crate::asset_loading::load_random_from_dir(
                                            "water/water_edge_bottom/water_edge_bottom_left_grass",
                                            (world_x, world_y),
                                            &asset_server,
                                        ),
                                        edge_rotation(edge),
                                    )
                                }
                                else if edge == WaterEdge::BottomRight {
                                    (
                                        crate::asset_loading::load_random_from_dir(
                                            "water/water_edge_bottom/water_edge_bottom_right_grass",
                                            (world_x, world_y),
                                            &asset_server,
                                        ),
                                        edge_rotation(edge),
                                    )
                                }
                                else {
                                    (
                                        crate::asset_loading::load_random_from_dir(
                                            "water/water_edge/water_edge_grass",
                                            (world_x, world_y),
                                            &asset_server,
                                        ),
                                        edge_rotation(edge),
                                    )
                                }
                            } else {
                                (
                                    crate::asset_loading::load_random_from_dir(
                                        "water/water_full",
                                        (world_x, world_y),
                                        &asset_server,
                                    ),
                                    Quat::IDENTITY,
                                )
                            }
                        }
                    };


                    // Base grass tile
                    parent.spawn((
                        chunk.data[x][y],
                        Sprite {
                            image: sprite_image,
                            custom_size: Some(Vec2::splat(1.0)),
                            ..default()
                        },
                        Transform {
                            translation: Vec3::new(x as f32, y as f32, ZLayers::World as i32 as f32),
                            rotation,
                            ..default()
                        },
                    ));
                }
            }
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WaterEdge {
    Top,
    Bottom,
    Left,
    Right,
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight
}

fn water_edge(chunk: &Chunk, x: usize, y: usize) -> Option<WaterEdge> {
    if chunk.data[x][y + 1] == WorldTiles::Grass {
        if chunk.data[x-1][y] == WorldTiles::Grass {
            Some(WaterEdge::TopLeft)
        }
        else if chunk.data[x+1][y] == WorldTiles::Grass {
            Some(WaterEdge::TopRight)
        }
        else {
            Some(WaterEdge::Top)
        }
    } else if chunk.data[x][y - 1] == WorldTiles::Grass {
        if chunk.data[x-1][y] == WorldTiles::Grass {
            Some(WaterEdge::BottomLeft)
        }
        else if chunk.data[x+1][y] == WorldTiles::Grass {
            Some(WaterEdge::BottomRight)
        }
        else {
            Some(WaterEdge::Bottom)
        }
    } else if chunk.data[x - 1][y] == WorldTiles::Grass {
        Some(WaterEdge::Left)
    } else if chunk.data[x + 1][y] == WorldTiles::Grass {
        Some(WaterEdge::Right)
    } else {
        None
    }
}

fn edge_rotation(edge: WaterEdge) -> Quat {
    match edge {
        WaterEdge::Top => Quat::IDENTITY,
        WaterEdge::Right => Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        WaterEdge::Bottom => Quat::IDENTITY,
        WaterEdge::Left => Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
        WaterEdge::BottomLeft => Quat::IDENTITY,
        WaterEdge::BottomRight => Quat::IDENTITY,
        WaterEdge::TopLeft => Quat::IDENTITY,
        WaterEdge::TopRight => Quat::IDENTITY,
    }
}


pub fn setup_world_noise(
    mut commands: Commands,
    generator: Res<WorldGenerator>,
) {
    let mut altitude = FastNoiseLite::new();
    altitude.set_seed(Some(generator.seed as i32));
    altitude.set_noise_type(Some(NoiseType::Perlin));
    altitude.set_frequency(Some(0.08));

    let mut vegetation = FastNoiseLite::new();
    vegetation.set_seed(Some((generator.seed + 1337) as i32)); // offset for variation
    vegetation.set_noise_type(Some(NoiseType::Perlin));
    vegetation.set_frequency(Some(0.08));

    let mut temp = FastNoiseLite::new();
    temp.set_seed(Some((generator.seed + 1337) as i32)); // offset for variation
    temp.set_noise_type(Some(NoiseType::Perlin));
    temp.set_frequency(Some(0.08));

    let mut moisture = FastNoiseLite::new();
    moisture.set_seed(Some((generator.seed + 1337) as i32)); // offset for variation
    moisture.set_noise_type(Some(NoiseType::Perlin));
    moisture.set_frequency(Some(0.08));

    commands.insert_resource(WorldNoise {
        altitude,
        vegetation,
        temp,
        moisture
    });
}
pub fn despawn_distant_chunks(
    chunks_query: Query<(Entity, &Chunk), With<Chunk>>,
    player_transform: Single<&Transform, With<crate::player::Player>>,
    mut commands: Commands,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    // Get player position
    let player_pos = player_transform;

    let player_chunk = IVec2 {
        x: (player_pos.translation.x / CHUNK_SIZE as f32).floor() as i32,
        y: (player_pos.translation.y / CHUNK_SIZE as f32).floor() as i32,
    };

    for (entity, chunk) in chunks_query.iter() {
        if chunk.coord.distance_squared(player_chunk) > 8 && chunk.coord != IVec2::new(0,0) {
            // Remove chunk and all its children
            commands.entity(entity).despawn_related::<Children>();
            commands.entity(entity).despawn();

            // Remove from loaded chunks set
            loaded_chunks.coords.remove(&chunk.coord);
            info!("Despawning {}", chunk.coord)
        }
    }
}
