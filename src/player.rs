use bevy::prelude::*;
mod player_camera;

use player_camera::*;
use crate::ZLayers;

// Components
#[derive(Component)]
pub struct Player;


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
        Transform::from_xyz(0.0, 0.0, ZLayers::Entities as i32 as f32),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        crate::physics::PhysicsObject,
        crate::physics::player_movement_physics::PlayerController
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
            .add_systems(Update, 
                player_camera::update_camera
            );
    }
}