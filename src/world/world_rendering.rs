use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_firefly::prelude::*;

use crate::player::Player;
use crate::world::world_generation:: { CHUNK_SIZE, Chunk, FlowerType, Foliage, TreeType, WorldMap, WorldTile };

use crate::render::{self, OccludesPlayer, SortOffset};

pub const RENDER_DISTANCE: u32 = 2; // chunks

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

                    spawn_foliage_entity(parent, foliage, world_x, world_y, chunk_x, chunk_y, &asset_server);
                }
            }

        });
    }
}

pub fn hide_loaded_chunks_out_of_render_distance(
    map: Res<WorldMap>,
    mut chunks: Query<(Entity, &mut Visibility), With<Chunk>>,
    player_transform: Single<&Transform, With<Player>>,
) {
    let player_pos = player_transform.translation;
    let player_chunk = IVec2::new(
        (player_pos.x / CHUNK_SIZE as f32).floor() as i32,
        (player_pos.y / CHUNK_SIZE as f32).floor() as i32,
    );

    for (chunk_coords, &chunk_entity) in map.chunks.iter() {
        let Ok((_, mut visibility)) = chunks.get_mut(chunk_entity) else {
            continue;
        };

        let distance = ((*chunk_coords) - player_chunk).abs();
        let in_range = distance.x <= RENDER_DISTANCE as i32 && distance.y <= RENDER_DISTANCE as i32;

        *visibility = if in_range {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
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

fn spawn_foliage_entity(
    parent: &mut bevy::ecs::relationship::RelatedSpawnerCommands<'_, ChildOf>,
    foliage: Foliage,
    world_x: i32,
    world_y: i32,
    chunk_x: usize,
    chunk_y: usize,
    asset_server: &Res<AssetServer>
) {
    let (foliage_asset, size_x, size_y, sort_offset) = match foliage {
        Foliage::Flower(FlowerType::Red) => 
            (
                crate::asset_loading::load_random_asset_from_dir(
                    "foliage/plants", 
                    (world_x, world_y), 
                    &asset_server
                ),
                1.0,
                1.0,
                SortOffset(0.0)
            ),
        Foliage::TallGrass =>
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
                4.0,
                8.0,
                SortOffset(0.0) // hmmm
            ),
        _ => (Handle::default(),1.0,1.0,SortOffset(0.0))
    };
    if foliage == Foliage::Tree(TreeType::Oak) {
        parent.spawn((
            foliage,
            OccludesPlayer,
            crate::physics::collision::Collider::circle(0.1, 1.0),
            crate::physics::collision::StaticBody,
            sort_offset,
            Occluder2d::circle(0.25).with_opacity(1.0),
            render::RenderLayer::Foliage,
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
    } else if foliage == Foliage::TallGrass {

        parent.spawn((
            foliage,
            sort_offset,
            render::grass_sway::PlayerSway,
            Occluder2d::circle(0.25)
                .with_offset(vec3(0.5, 0.5, 0.0))
                .with_opacity(0.3),
            render::RenderLayer::Foliage,
            Sprite {
                image: asset_server.load("foliage/bush/bush_0.png"),
                custom_size: Some(vec2(size_x,size_y)),
                ..default()
            },
            Anchor::BOTTOM_CENTER,
            Transform {
                translation: Vec3::new(chunk_x as f32 + 0.5, chunk_y as f32, 0.0),
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
            Anchor::BOTTOM_CENTER,
            Transform {
                translation: Vec3::new(chunk_x as f32 + 0.5, chunk_y as f32, 0.0),
                ..default()
            },
        ));
    }
}