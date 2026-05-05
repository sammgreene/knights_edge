use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
use crate::creatures::*;
use crate::physics::*;
use crate::render::*;

fn spawn_creature(
    kind: &'static str,
    translation: Vec3,
    commands: &mut Commands,
    state: CreatureState,
    asset_server: &Res<AssetServer>
) {

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 50.0,
        ..default()
    };
    let text_justification = Justify::Center;

    commands.spawn((
        Creature(kind.to_string()),
        state,
        CreatureMemory::default(),
        RenderLayer::Foliage,
        // Sprite {
        //     image: asset_server.load("items/apple.png"),
        //     custom_size: Some(vec2(1.0, 1.5)),
        //     ..default()
        // },
        PhysicsObject,
        Transform::from_translation(translation),
        crate::physics::collision::Collider::circle(0.25, 0.8),
        crate::physics::collision::DynamicBody::new(1.0),
        Visibility::Visible,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text2d::new(kind),
            TextFont { font_size: 21.0, ..default() },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(vec3(0.02, 0.02, 1.0)),
            TextBackgroundColor(Color::BLACK.with_alpha(0.7)),
        ));
    });
}

pub fn spawn_randoms(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    spawn_creature("Deer", vec3(-5., 6.0, 0.0), &mut commands, CreatureState::default(), &asset_server);
    spawn_creature("Deer", vec3(5., 6.0, 0.0), &mut commands, CreatureState::default(), &asset_server);
    spawn_creature("Deer", vec3(5., 6.0, 0.0), &mut commands, CreatureState::default(), &asset_server);
    spawn_creature("Wolf", vec3(5., 6.0, 0.0), &mut commands, CreatureState::default().with_saturation(5.0), &asset_server);
}