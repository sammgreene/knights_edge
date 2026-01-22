use bevy::prelude::*;

mod world_lib;
pub mod world_generator;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(world_generator::WorldGenerator::with_seed("cyan"))
            .insert_resource(world_generator::LoadedChunks::default())
            .add_systems(Startup, (
                startup_create_world,
                world_generator::setup_world_noise
            ))
            .add_systems(Update, (
                // Add update systems here
                world_generator::print_world_generator_info,
                world_generator::check_to_load_chunks,
                world_generator::spawn_chunk_tiles_for_new_chunks,
                world_generator::despawn_distant_chunks,
            ));
    }
}


// Systems
fn startup_create_world() {}
