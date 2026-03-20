use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_firefly::prelude::*;

use crate::world::world_generation:: { CHUNK_SIZE, Chunk, WorldTile, Foliage, TreeType };

use crate::render::{self, OccludesPlayer, SortOffset};

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

                    let foliage = chunk.foliage_data[chunk_x][chunk_y];
                    let (foliage_asset, size_x, size_y, sort_offset) = match foliage {
                        // Foliage::Rock => 
                        //     (
                        //         crate::asset_loading::load_random_asset_from_dir(
                        //             "foliage/rocks/small_rock", 
                        //             (world_x, world_y), 
                        //             &asset_server
                        //         ),
                        //         1.0,
                        //         1.0,
                        //         SortOffset(0.0)
                        //     ),
                        Foliage::Bush =>
                            (
                                crate::asset_loading::load_random_asset_from_dir(
                                    "foliage/bush", 
                                    (world_x, world_y), 
                                    &asset_server
                                ),
                                1.0,
                                2.0,
                                SortOffset(0.0)
                            ),
                        Foliage::Tree(TreeType::Oak) =>
                            (
                                crate::asset_loading::load_random_asset_from_dir(
                                    "foliage/tree", 
                                    (world_x, world_y), 
                                    &asset_server
                                ),
                                5.0,
                                7.0,
                                SortOffset(0.0) // hmmm
                            ),
                        Foliage::None => (Handle::default(),1.0,1.0,SortOffset(0.0))
                    };

                    if foliage == Foliage::Tree(TreeType::Oak) {
                        parent.spawn((
                            OccludesPlayer,
                            sort_offset,
                            foliage,
                            Occluder2d::circle(0.25).with_offset(vec3(0.5, 0.5, 0.0)).with_opacity(1.0),
                            render::RenderLayer::Foliage,// .with_offset(new_y as f32 - chunk_y as f32),
                            Sprite {
                                image: foliage_asset,
                                custom_size: Some(Vec2::new(size_x, size_y)),
                                ..default()
                            },
                            Anchor::BOTTOM_CENTER,
                            Transform {
                                translation: Vec3::new(chunk_x as f32 + 0.5, chunk_y as f32 - 0.3, 0.0),
                                ..default()
                            },
                        ));
                    } else {
                        parent.spawn((
                            foliage,
                            sort_offset,
                            Occluder2d::circle(0.25).with_offset(vec3(0.5, 0.5, 0.0)).with_opacity(0.3),
                            render::RenderLayer::Foliage,// .with_offset(new_y as f32 - chunk_y as f32),
                            Sprite {
                                image: foliage_asset,
                                custom_size: Some(Vec2::new(size_x, size_y)),
                                ..default()
                            },
                            Anchor::BOTTOM_LEFT,
                            Transform {
                                translation: Vec3::new(chunk_x as f32, chunk_y as f32, 0.0),
                                ..default()
                            },
                        ));
                    }
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

// fn get_foliage_entity(foliage: Foliage, world_x: i32, world_y: i32, chunk_x: i32, chunk_y: i32, asset_server: &AssetServer) -> impl Bundle {
//     let (foliage_asset, size_x, size_y, offset_x, offset_y, sort_offset) = match foliage {
//         Foliage::Rock => 
//             (
//                 crate::asset_loading::load_random_asset_from_dir(
//                     "foliage/rocks/small_rock", 
//                     (world_x, world_y), 
//                     &asset_server
//                 ),
//                 1.0,
//                 1.0,
//                 0.0,
//                 0.0,
//                 SortOffset(0.0)
//             ),
//         Foliage::Bush =>
//             (
//                 crate::asset_loading::load_random_asset_from_dir(
//                     "foliage/bush", 
//                     (world_x, world_y), 
//                     &asset_server
//                 ),
//                 1.0,
//                 1.0,
//                 0.0,
//                 0.0,
//                 SortOffset(0.0)
//             ),
//         Foliage::Tree(TreeType::Oak) =>
//             (
//                 crate::asset_loading::load_random_asset_from_dir(
//                     "foliage/tree", 
//                     (world_x, world_y), 
//                     &asset_server
//                 ),
//                 5.0,
//                 7.0,
//                 -2.0,
//                 0.0,
//                 SortOffset(1.0) // hmmm
//             ),
//         Foliage::None => (Handle::default(),1.0,1.0,0.,0.,SortOffset(0.0))
//     };

//     (
//         OccludesPlayer,
//         foliage,
//         Occluder2d::circle(0.25).with_offset(vec3(0.5, 0.5, 0.0)).with_opacity(0.3),
//         render::RenderLayer::Foliage,// .with_offset(new_y as f32 - chunk_y as f32),
//         Sprite {
//             image: foliage_asset,
//             custom_size: Some(Vec2::new(size_x, size_y)),
//             ..default()
//         },
//         Anchor::BOTTOM_LEFT,
//         Transform {
//             translation: Vec3::new(chunk_x as f32 + offset_x, chunk_y as f32 + offset_y, 0.0),
//             ..default()
//         },
//     );
// }