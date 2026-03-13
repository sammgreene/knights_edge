use bevy::prelude::*;
use bevy_firefly::prelude::*;
mod world;
mod player;
mod physics;
mod asset_loading;
mod debug;
mod render;
mod creatures;
mod spawning;
mod items;
mod triggers;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FireflyPlugin,
            world::WorldPlugin,
            player::PlayerPlugin,
            physics::PhysicsPlugin,
            debug::DebugPlugin,
            render::RenderPlugin,
            creatures::MobPlugin,
            spawning::MobSpawningPlugin,
            items::ItemsPlugin,
            bevy_spritesheet_animation::prelude::SpritesheetAnimationPlugin
        ))
        .run();
}