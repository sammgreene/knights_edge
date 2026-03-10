use bevy::prelude::*;
use bevy_firefly::prelude::*;

const CAMERA_DECAY_RATE: f32 = 3.5;

// Player Camera Marker Component
#[derive(Component)]
pub struct PlayerCamera {
    pub decay_rate: f32,
}

// Systems
pub fn spawn_player_camera(
    mut commands: Commands,
) {
    // --- Camera entity ---
    let mut projection = OrthographicProjection::default_2d();
    projection.near = -1000.0;
    projection.far = 1000.0;
    projection.scale = 0.025; // larger number zooms out

    // Lighting Scheme
    let mut lighting = FireflyConfig::default();
    lighting.ambient_brightness = 0.25;
    lighting.softness = Some(0.5);
    lighting.z_sorting = true;
    lighting.ambient_color = Color::hsl(35., 0.89, 0.9);

    commands.spawn((
        Camera2d,
        lighting,
        PlayerCamera { decay_rate: CAMERA_DECAY_RATE },
        Projection::Orthographic(projection),
        Transform::default(),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}


// Systems
pub fn update_camera(
    mut params: ParamSet<(
        Single<(&mut Transform, &PlayerCamera)>,
        Single<&Transform, With<crate::player::Player>>,
    )>,
    time: Res<Time>,
) {
    let player_transform = params.p1();
    let Vec3 { x, y, .. } = player_transform.translation;

    let (mut camera_transform, camera) = params.p0().into_inner();
    let direction = Vec3::new(x, y, camera_transform.translation.z);
    let decay_rate = camera.decay_rate;

    camera_transform
        .translation
        .smooth_nudge(&direction, decay_rate, time.delta_secs());
}