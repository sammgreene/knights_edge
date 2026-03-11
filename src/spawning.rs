use bevy::prelude::*;
use crate::*;

pub struct MobSpawningPlugin;

impl Plugin for MobSpawningPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                spawn_baseline,
            ))
            // .add_systems(Update, 
            // )
        ;
    }
}

fn spawn_baseline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        creatures::types::Eye,
        creatures::states::Idle,
        creatures::types::eye_state_machine(),
        creatures::attributes::Speed(3.0),

        physics::PhysicsObject,
        Sprite {
            image: asset_server.load("eye/eye.png"),
            custom_size: Some(Vec2::splat(1.5)),
            ..default()
        },
        crate::render::RenderLayer::FoliageBack,

        Transform::from_xyz(10.0, 5.0, 0.0),
    ));
}