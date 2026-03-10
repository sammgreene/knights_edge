use bevy::{prelude::*, sprite::Anchor};
pub mod player_camera;
use player_camera::*;
use crate::render::RenderLayer;

// Components
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerMarker;

// Systems
pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // --- Player entity ---
    commands.spawn((
        Player,
        Sprite {
            image: asset_server.load("player/player.png"),
            custom_size: Some(Vec2::splat(1.0)),
            ..default()
        },
        RenderLayer::FoliageBack,

        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        
        crate::physics::PhysicsObject,
        crate::physics::player_movement_physics::PlayerController
    ));
    commands.spawn((
        PlayerMarker,
        RenderLayer::UI,
        Sprite {
            image: asset_server.load("snow/snow_full/snow_full_0.png"),
            custom_size: Some(Vec2::splat(1.0)),
            ..default()
        },
        Anchor::BOTTOM_LEFT,
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));

    spawn_player_camera(commands);
}


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                spawn_player,
            ))
            .add_systems(Update, (
                player_camera::update_camera,
                update_player_marker
            ));
    }
}

fn update_player_marker(
    player: Single<&Transform, (With<Player>, Without<PlayerMarker>)>,
    mut marker: Single<&mut Transform, With<PlayerMarker>>
) {
    marker.translation = vec3(player.translation.x.floor(), player.translation.y.floor(), RenderLayer::UI as i32 as f32)
}
