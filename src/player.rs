use bevy::input::common_conditions::input_just_pressed;
use bevy::{prelude::*, sprite::Anchor, audio};
pub mod player_camera;
use bevy_firefly::lights::PointLight2d;
// use bevy_firefly::sprites;
use player_camera::*;
use crate::physics::*;
use crate::render::*;
use crate::items::*;
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
                update_player_animation,
                drop_held_item.run_if(input_just_pressed(KeyCode::KeyQ)),
                update_running_sounds
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
    // attack: Handle<Animation>
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

    // let attack_animation = spritesheet.create_animation().add_row(18).build();

    // let attack_animation_handle = animations.add(attack_animation);

    commands.insert_resource(PlayerAnimations {
        idle: idle_animation_handle.clone(),
        run: run_animation_handle,
        // attack: attack_animation_handle,
    });

    let mut sprite = spritesheet.with_size_hint(288,1152).sprite(&mut atlas_layouts);
    sprite.custom_size = Some(Vec2::splat(4.0));

    // Create an entity dedicated to playing our background sounds
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/forest_ambient.wav")),
        PlaybackSettings::LOOP.with_volume(audio::Volume::Linear(0.2)),
    ));

    // Create an entity dedicated to playing our background music
    commands.spawn((
        AudioPlayer::new(asset_server.load("sounds/field_theme.wav")),
        PlaybackSettings::LOOP.with_volume(audio::Volume::Linear(0.1)),
    ));

    // --- Player entity ---
    commands.spawn((
        Player,
        VisionProfile {
            occlusion_radius: 4.,
            min_alpha: 0.3,
            max_alpha: 0.9
        },
        PointLight2d {
            color: Color::hsl(25., 0.9, 0.5),
            intensity: 0.3,
            range: 4.5,
            ..Default::default()
        },
        crate::items::inventory::Inventory::new(16),

        AudioPlayer::new(asset_server.load("sounds/grass_running.wav")),
        PlaybackSettings::LOOP.with_volume(audio::Volume::Linear(0.45)),

        SpatialListener::new(0.5),

        sprite,
        SortOffset(0.2),

        SpritesheetAnimation::new(idle_animation_handle),
        RenderLayer::Foliage,

        Anchor(Vec2 { x: 0.0, y: -0.17 }),

        Transform::from_xyz(0.0, 0.0, 0.0),
        
        crate::physics::PhysicsObject,
        crate::physics::player_movement_physics::PlayerController,
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

fn update_running_sounds(
    player: Single<(&Velocity, &AudioSink), With<Player>>
) {
    let (velo, sink) = player.into_inner();

    if velo.is_moving() { sink.play(); }
    else { sink.pause(); }
}

fn drop_held_item(
    player: Single<(Entity, &mut inventory::Inventory, &Transform), With<Player>>,
    mut commands: Commands,
    mut physics: Query<(&mut PhysicalTranslation, &mut PreviousPhysicalTranslation)>,
) {
    let (player, mut player_inventory, player_transform) = player.into_inner();
    if player_inventory.selected_item.is_none() { return }
    let held_item_index = player_inventory.selected_item.unwrap();
    let held_item = player_inventory.items[held_item_index];
    let pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);

    crate::physics::teleport_physics_object(held_item, pos, &mut physics);

    commands.entity(held_item)
        .remove::<AudioPlayer>()
        .remove::<PlaybackSettings>()
        .remove::<AudioSink>()
        .insert(Dropped)
        .insert(BobOffset { elapsed: 0.0 })
        .insert(PickupCooldown { effected_entity: player, timer: Timer::from_seconds(2.0, TimerMode::Once) });
    
    
    player_inventory.items.remove(held_item_index);
    if !player_inventory.selecting_valid_item() {
        player_inventory.select_valid_item();
    }
}