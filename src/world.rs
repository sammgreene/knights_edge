use bevy::prelude::*;

mod world_lib;
pub mod world_generation;
pub mod world_noise;
pub mod world_rendering;
pub mod entities;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(world_generation::LoadedChunks::default())
            .insert_resource(world_noise::WorldNoise::default("cyan"))
            // .add_systems(Startup, )
            .add_systems(Update, (
                // Add update systems here
                world_generation::load_near_chunks,
                world_rendering::spawn_tile_sprites_for_new_chunks,
                world_generation::despawn_distant_chunks,
            ).chain());
    }
}


// Systems
