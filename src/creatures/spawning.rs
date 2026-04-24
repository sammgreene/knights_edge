use bevy::prelude::*;
use crate::creatures::*;
use crate::physics::*;
use crate::render::*;

fn spawn_creature(
    kind: &'static str,
    translation: Vec3,
    mut commands: Commands,
    asset_server: &Res<AssetServer>
) {
    commands.spawn((
        Creature(kind.to_string()),
        CreatureState::default(),
        CreatureMemory::default(),

        RenderLayer::Foliage,
        Sprite {
            image: asset_server.load("items/apple.png"),
            custom_size: Some(vec2(1.0, 1.5)),
            ..default()
        },

        PhysicsObject,
        Transform::from_translation(translation),
        crate::physics::collision::Collider::circle(0.25, 0.8),
        crate::physics::collision::DynamicBody::new(1.0),
        Visibility::Visible,
    ));
}

pub fn spawn_randoms(
    commands: Commands,
    asset_server: Res<AssetServer>
) {
    spawn_creature("Deer", vec3(-5., 6.0, 0.0), commands, &asset_server);
}