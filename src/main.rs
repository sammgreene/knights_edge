use bevy::prelude::*;
mod world;
mod player;
mod physics;
mod asset_loading;
mod debug;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(world::WorldPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(debug::DebugPlugin)
        .run();
}

pub enum ZLayers {
    Background,
    World,
    Foliage,
    Entities,
    UI,
}