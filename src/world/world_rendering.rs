use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_firefly::prelude::*;

use crate::world::world_generation:: { CHUNK_SIZE, Chunk, WorldTile, Foliage, TreeType };

use crate::render::{self, OccludesPlayer, SortOffset};

// Generates tiles for any newly created chunks on the frame they are added
pub fn spawn_tile_sprites_for_new_chunks(
    mut commands: Commands,
    new_chunks: Query<(Entity, &Chunk), Added<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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

                    spawn_foliage_entity(parent, foliage, world_x, world_y, chunk_x, chunk_y, &mut meshes, &mut materials, &asset_server);
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

fn spawn_foliage_entity(
    parent: &mut bevy::ecs::relationship::RelatedSpawnerCommands<'_, ChildOf>,
    foliage: Foliage,
    world_x: i32,
    world_y: i32,
    chunk_x: usize,
    chunk_y: usize,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>
) {
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
                4.0,
                SortOffset(0.0) // hmmm
            ),
        Foliage::None => (Handle::default(),1.0,1.0,SortOffset(0.0))
    };
    if foliage == Foliage::Tree(TreeType::Oak) {
        parent.spawn((
            foliage,
            OccludesPlayer,
            sort_offset,
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
    } else if foliage == Foliage::TallGrass {
        let mesh = create_grass_quad(size_x, size_y);
        let mesh_handle = meshes.add(mesh);

        parent.spawn((
            foliage,
            sort_offset,
            WindSway {
                sway_offset: (chunk_x as f32 * 1.3 + chunk_y as f32 * 0.7).fract() * std::f32::consts::TAU,
                sway_strength: 0.75,
            },
            Occluder2d::circle(0.25)
                .with_offset(vec3(0.5, 0.5, 0.0))
                .with_opacity(0.3),
            render::RenderLayer::Foliage,
            Mesh2d(mesh_handle),
            MeshMaterial2d(materials.add(ColorMaterial {
                texture: Some(foliage_asset),
                ..default()
            })),
            Anchor::BOTTOM_LEFT,
            Transform {
                translation: Vec3::new(chunk_x as f32, chunk_y as f32, 0.0),
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

#[derive(Component)]
pub struct WindSway {
    pub sway_offset: f32,
    pub sway_strength: f32
}

fn create_grass_quad(width: f32, height: f32) -> Mesh {
    // Vertex layout (indices):
    //  2 ------- 3   <- top (these will sway)
    //  |         |
    //  0 ------- 1   <- bottom (pinned)
    
    let positions: Vec<[f32; 3]> = vec![
        [0.0,   0.0,    0.0], // 0 bottom-left  (pinned)
        [width, 0.0,    0.0], // 1 bottom-right (pinned)
        [0.0,   height, 0.0], // 2 top-left     (sways)
        [width, height, 0.0], // 3 top-right    (sways)
    ];

    let uvs: Vec<[f32; 2]> = vec![
        [0.0, 1.0], // bottom-left
        [1.0, 1.0], // bottom-right
        [0.0, 0.0], // top-left
        [1.0, 0.0], // top-right
    ];

    let indices = vec![0u32, 1, 2,  1, 3, 2];

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::MAIN_WORLD | bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

pub fn wind_sway_system(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(&WindSway, &Mesh2d, &Transform)>,
) {
    let t = time.elapsed_secs();
    let wind_speed = 1.8_f32;
    let base_sway = 0.12_f32; // max X displacement in world units

    for (grass, mesh2d, transform) in &query {
        let Some(mesh) = meshes.get_mut(&mesh2d.0) else { continue };

        let phase = t * wind_speed + grass.sway_offset;
        // Use two frequencies for a more organic feel
        let sway_x = (phase.sin() + 0.4 * (phase * 2.3).sin())
            * base_sway
            * grass.sway_strength;

        let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        else { continue };

        let height = positions[2][1]; // top-left Y = grass height

        // Indices 2 and 3 are the top vertices
        positions[2][0] = 0.0 + sway_x;           // top-left X
        positions[3][0] = transform.scale.x + sway_x; // top-right X
        // If not using scale, store width in WindGrass and use that instead
    }
}