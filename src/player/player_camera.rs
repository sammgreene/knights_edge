use bevy::prelude::*;

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
    projection.scale = 0.025;

    commands.spawn((
        Camera2d,
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