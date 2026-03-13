use bevy::{prelude::*, sprite::Anchor};
pub mod player_camera;
use bevy_firefly::sprites;
use player_camera::*;
use crate::render::RenderLayer;
use bevy_spritesheet_animation::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, (
                spawn_player,
            ))
            .add_systems(Update, (
                player_camera::update_camera,
                update_player_marker,
                update_player_animation
            ));
    }
}

// Components
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerMarker;


// player animations
#[derive(Resource)]
struct PlayerAnimations {
    idle: Handle<Animation>,
    run: Handle<Animation>,
    attack: Handle<Animation>
}

#[derive(Component)]
pub struct VisionProfile {
    pub occlusion_radius: f32,
    pub min_alpha: f32,
    pub max_alpha: f32,
}

// Systems
pub fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<Animation>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {

    let image = asset_server.load("player/player_sheet.png");

    let spritesheet = Spritesheet::new(&image, 6, 24);

    let idle_animation = spritesheet.create_animation().add_row(0).build();

    let idle_animation_handle = animations.add(idle_animation);

    let run_animation = spritesheet.create_animation().add_row(2).build();

    let run_animation_handle = animations.add(run_animation);

    let attack_animation = spritesheet.create_animation().add_row(18).build();

    let attack_animation_handle = animations.add(attack_animation);

    commands.insert_resource(PlayerAnimations {
        idle: idle_animation_handle.clone(),
        run: run_animation_handle,
        attack: attack_animation_handle,
    });

    let mut sprite = spritesheet.with_size_hint(288,1152).sprite(&mut atlas_layouts);
    sprite.custom_size = Some(Vec2::splat(4.0));

    // --- Player entity ---
    commands.spawn((
        Player,
        VisionProfile {
            occlusion_radius: 4.,
            min_alpha: 0.3,
            max_alpha: 0.9
        },

        SpatialListener::new(0.5),

        sprite,

        SpritesheetAnimation::new(idle_animation_handle),
        RenderLayer::Foliage,

        Anchor(Vec2 { x: 0.0, y: -0.17 }),

        Transform::from_xyz(0.0, 0.0, 0.0),
        
        crate::physics::PhysicsObject,
        crate::physics::player_movement_physics::PlayerController,
        crate::items::inventory::Inventory::new(16)
    ));

    // --- Debug Player MarkerThing ---
    commands.spawn((
        PlayerMarker,
        RenderLayer::UI,
        Sprite {
            image: asset_server.load("snow/snow_full/snow_full_0.png"),
            custom_size: Some(Vec2::splat(1.0)),
            color: Color::Srgba(Srgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.4 }),
            ..default()
        },
        bevy::sprite::Anchor::BOTTOM_LEFT,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Visible,
    ));

    spawn_player_camera(commands);
}

fn update_player_marker(
    player: Single<&Transform, (With<Player>, Without<PlayerMarker>)>,
    mut marker: Single<&mut Transform, With<PlayerMarker>>
) {
    marker.translation = vec3(player.translation.x.floor(), player.translation.y.floor(), RenderLayer::UI as i32 as f32)
}

fn update_player_animation(
    player: Single<(&crate::physics::Velocity, &mut Sprite, &mut SpritesheetAnimation)>,
    player_animations: Res<PlayerAnimations>
) {
    let (player_velocity, mut player_sprite, mut animation) = player.into_inner();

    if player_velocity.is_moving() { // if moving
        if animation.animation != player_animations.run {
            animation.switch(player_animations.run.clone());
        }
    }
    else { // not moving
        if animation.animation != player_animations.idle {
            animation.switch(player_animations.idle.clone());
        }
    }
    if player_velocity.moving_left() {
        player_sprite.flip_x = true;
    }
    else {
        player_sprite.flip_x = false;
    }
}