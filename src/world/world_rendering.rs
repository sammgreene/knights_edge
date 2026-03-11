use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_firefly::prelude::*;

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

            for chunk_x in 0..CHUNK_SIZE {
                for chunk_y in 0..CHUNK_SIZE {
                    let world_x = chunk.coord.x * CHUNK_SIZE as i32 + chunk_x as i32;
                    let world_y = chunk.coord.y * CHUNK_SIZE as i32 + chunk_y as i32;
                    
                    // WORLD TILES
                    let (tile_asset, rotation) = get_tile_asset_and_rotation(&asset_server, chunk, chunk_x, chunk_y);
                    parent.spawn((
                        // chunk.tile_data[chunk_x][chunk_x],
                        Sprite {
                            image: tile_asset,
                            custom_size: Some(Vec2::splat(1.0)),
                            ..default()
                        },
                        Anchor::BOTTOM_LEFT,
                        Transform {
                            translation: Vec3::new(chunk_x as f32, chunk_y as f32, render::RenderLayer::World as i32 as f32),
                            rotation,
                            ..default()
                        },
                    ));

                    // Foliage Spawning
                    if chunk.foliage_data[chunk_x][chunk_y] == Foliage::None { continue; }

                    let foliage_asset = match chunk.foliage_data[chunk_x][chunk_y] {
                        Foliage::Rock => 
                            crate::asset_loading::load_random_asset_from_dir(
                                "foliage/rocks/small_rock", 
                                (world_x, world_y), 
                                &asset_server
                            ),
                        Foliage::Bush =>
                            crate::asset_loading::load_random_asset_from_dir(
                                "foliage/bush", 
                                (world_x, world_y), 
                                &asset_server
                            ),
                        Foliage::Tree(TreeType::Oak) => {
                            crate::asset_loading::load_random_asset_from_dir(
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
                    if chunk.foliage_data[chunk_x][chunk_y] == Foliage::Tree(TreeType::Oak) {
                        size_x = 5.;
                        size_y = 7.;
                        new_y = chunk_y;
                        new_x = chunk_x - 2;
                    }
                    parent.spawn((
                        chunk.foliage_data[chunk_x][chunk_y], // Foliage
                        Occluder2d::circle(0.25).with_offset(vec3(0.5, 0.5, 0.0)).with_opacity(0.3),
                        render::RenderLayer::FoliageBack,// .with_offset(new_y as f32 - chunk_y as f32),
                        Sprite {
                            image: foliage_asset,
                            custom_size: Some(Vec2::new(size_x, size_y)),
                            ..default()
                        },
                        Anchor::BOTTOM_LEFT,
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

// world_tile_at(x,y)
fn get_tile_asset_and_rotation(asset_server: &Res<AssetServer>, chunk: &Chunk, x: usize, y: usize) -> (Handle<Image>, Quat) {
    let world_x = chunk.coord.x * CHUNK_SIZE as i32 + x as i32;
    let world_y = chunk.coord.y * CHUNK_SIZE as i32 + y as i32;

    match chunk.tile_data[x][y] {
        WorldTile::Grass => (
            crate::asset_loading::load_random_asset_from_dir(
                "grass_variants/grass_full",
                (world_x, world_y),
                &asset_server,
            ),
            Quat::IDENTITY,
        ),
        WorldTile::Snow => (
            crate::asset_loading::load_random_asset_from_dir(
                "snow/snow_full",
                (world_x, world_y),
                &asset_server,
            ),
            Quat::IDENTITY,
        ),
        WorldTile::Water => (
            crate::asset_loading::load_random_asset_from_dir(
                "water/water_full",
                (world_x, world_y),
                &asset_server,
            ),
            Quat::IDENTITY,
        )
    }
}