use bevy::prelude::*;

mod world_lib;
pub mod world_generation;
pub mod world_noise;
pub mod world_rendering;
mod daycycle;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(world_generation::WorldMap::default())
            .insert_resource(world_noise::WorldNoise::default("cyan"))
            .insert_resource(daycycle::GameTime::default())
            // .add_systems(Startup, )
            .add_systems(    Update,
                (
                    (
                        world_generation::load_near_chunks,
                        world_generation::register_new_chunks,
                        world_generation::despawn_distant_chunks,
                        world_rendering::spawn_tile_sprites_for_new_chunks,
                        daycycle::advance_game_time,
                        daycycle::update_ambient_light
                    ).chain(),
                    daycycle::advance_game_time,
                    daycycle::update_ambient_light
                )
            );
    }
}


// Systems
