use bevy::prelude::*;

use crate::world::world_generation:: { CHUNK_SIZE, Chunk, WorldTile, Foliage, TreeType };

use crate::render;

// Generates tiles for any newly created chunks on the frame they are added
pub fn spawn_tile_sprites_for_new_chunks(
    mut commands: Commands,
    new_chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, chunk) in new_chunks.iter() {
        commands.entity(entity).with_children(|parent| {

            for chunk_x in 1..CHUNK_SIZE+1 {
                for chunk_y in 1..CHUNK_SIZE+1 {
                    let world_x = chunk.coord.x * CHUNK_SIZE as i32 + chunk_x as i32;
                    let world_y = chunk.coord.y * CHUNK_SIZE as i32 + chunk_y as i32;
                    
                    // WORLD TILES
                    let (tile_asset, rotation) = get_tile_asset_and_rotation(&asset_server, chunk, chunk_x, chunk_y);
                    parent.spawn((
                        chunk.tile_data[chunk_x][chunk_x],
                        Sprite {
                            image: tile_asset,
                            custom_size: Some(Vec2::splat(1.0)),
                            ..default()
                        },
                        Transform {
                            translation: Vec3::new(chunk_x as f32, chunk_y as f32, render::ZLayers::World as i32 as f32),
                            rotation,
                            ..default()
                        },
                    ));

                    // Foliage Spawning
                    let fx = chunk_x-1;
                    let fy = chunk_y-1;
                    if chunk.foliage_data[fx][fy] == Foliage::None { continue; }

                    let foliage_asset = match chunk.foliage_data[fx][fy] {
                        Foliage::Rock => 
                            crate::asset_loading::load_random_from_dir(
                                "foliage/rocks/small_rock", 
                                (world_x, world_y), 
                                &asset_server
                            ),
                        Foliage::Bush =>
                            crate::asset_loading::load_random_from_dir(
                                "foliage/bush", 
                                (world_x, world_y), 
                                &asset_server
                            ),
                        Foliage::Tree(TreeType::Oak) => {
                            crate::asset_loading::load_random_from_dir(
                                "foliage/tree", 
                                (world_x, world_y), 
                                &asset_server
                            )
                            // Handle::default()
                        }
                        Foliage::None => Handle::default(),
                    };
                    let mut new_x = chunk_x;
                    let mut new_y = chunk_y;
                    let mut size_x = 1.0;
                    let mut size_y = 1.0;
                    if chunk.foliage_data[fx][fy] == Foliage::Tree(TreeType::Oak) {
                        size_x = 5.;
                        size_y = 7.;
                        new_y = chunk_y + 3;
                        new_x = chunk_x;
                    }
                    parent.spawn((
                        render::YSort::on_layer(render::ZLayers::Foliage).with_offset(new_y as f32 - chunk_y as f32),
                        Sprite {
                            image: foliage_asset,
                            custom_size: Some(Vec2::new(size_x, size_y)),
                            ..default()
                        },
                        Transform {
                            translation: Vec3::new(new_x as f32, new_y as f32, 0.0),
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
    if chunk.tile_data[x][y + 1] == WorldTile::Grass {
        if chunk.tile_data[x-1][y] == WorldTile::Grass {
            Some(WaterEdge::TopLeft)
        }
        else if chunk.tile_data[x+1][y] == WorldTile::Grass {
            Some(WaterEdge::TopRight)
        }
        else {
            Some(WaterEdge::Top)
        }
    } else if chunk.tile_data[x][y - 1] == WorldTile::Grass {
        if chunk.tile_data[x-1][y] == WorldTile::Grass {
            Some(WaterEdge::BottomLeft)
        }
        else if chunk.tile_data[x+1][y] == WorldTile::Grass {
            Some(WaterEdge::BottomRight)
        }
        else {
            Some(WaterEdge::Bottom)
        }
    } else if chunk.tile_data[x - 1][y] == WorldTile::Grass {
        Some(WaterEdge::Left)
    } else if chunk.tile_data[x + 1][y] == WorldTile::Grass {
        Some(WaterEdge::Right)
    } else {
        None
    }
}

fn water_edge_rotation(edge: WaterEdge) -> Quat {
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

fn get_water_tile_and_rotation(asset_server: &Res<AssetServer>, chunk: &Chunk, chunk_x: usize, chunk_y: usize, world_x: i32, world_y: i32) -> (Handle<Image>, Quat) {
    if let Some(edge) = water_edge(chunk, chunk_x, chunk_y) {
        if edge == WaterEdge::TopLeft {
            (
                crate::asset_loading::load_random_from_dir(
                    "water/water_edge_top/water_edge_top_left_grass",
                    (world_x, world_y),
                    &asset_server,
                ),
                water_edge_rotation(edge),
            )
        }
        else if edge == WaterEdge::TopRight {
            (
                crate::asset_loading::load_random_from_dir(
                    "water/water_edge_top/water_edge_top_right_grass",
                    (world_x, world_y),
                    &asset_server,
                ),
                water_edge_rotation(edge),
            )
        }
        else if edge == WaterEdge::Top {
            (
                crate::asset_loading::load_random_from_dir(
                    "water/water_edge_top/water_edge_top_grass",
                    (world_x, world_y),
                    &asset_server,
                ),
                water_edge_rotation(edge) // Quat::IDENTITY,
            )
        }
        else if edge == WaterEdge::BottomLeft {
            (
                crate::asset_loading::load_random_from_dir(
                    "water/water_edge_bottom/water_edge_bottom_left_grass",
                    (world_x, world_y),
                    &asset_server,
                ),
                water_edge_rotation(edge),
            )
        }
        else if edge == WaterEdge::BottomRight {
            (
                crate::asset_loading::load_random_from_dir(
                    "water/water_edge_bottom/water_edge_bottom_right_grass",
                    (world_x, world_y),
                    &asset_server,
                ),
                water_edge_rotation(edge),
            )
        }
        else {
            (
                crate::asset_loading::load_random_from_dir(
                    "water/water_edge/water_edge_grass",
                    (world_x, world_y),
                    &asset_server,
                ),
                water_edge_rotation(edge),
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

// world_tile_at(x,y)
fn get_tile_asset_and_rotation(asset_server: &Res<AssetServer>, chunk: &Chunk, x: usize, y: usize) -> (Handle<Image>, Quat) {
    let world_x = chunk.coord.x * CHUNK_SIZE as i32 + x as i32 - 1;
    let world_y = chunk.coord.y * CHUNK_SIZE as i32 + y as i32 - 1;

    match chunk.tile_data[x][y] {
        WorldTile::Grass => (
            crate::asset_loading::load_random_from_dir(
                "grass_variants/grass_full",
                (world_x, world_y),
                &asset_server,
            ),
            Quat::IDENTITY,
        ),
        WorldTile::Snow => (
            crate::asset_loading::load_random_from_dir(
                "snow/snow_full",
                (world_x, world_y),
                &asset_server,
            ),
            Quat::IDENTITY,
        ),
        WorldTile::Water => get_water_tile_and_rotation(&asset_server, chunk, x, y, world_x, world_y)
    }
}