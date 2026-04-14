use bevy::prelude::*;


// #[derive(Component)]
// pub struct WindSway {
//     pub offset: f32,
//     pub strength: f32,
//     // pub base_x: f32
// }

#[derive(Component)]
pub struct PlayerSway;

pub fn player_grass_sway_system(
    mut grasses: Query<(&mut Transform, &GlobalTransform), (With<PlayerSway>, Without<crate::player::Player>)>,
    players: Query<&Transform, With<crate::player::Player>>,
) {
    for (mut transform, global) in &mut grasses {
        let grass_pos = global.translation().xy();

        let closest = players
            .iter()
            .map(|p| {
                let diff = grass_pos - p.translation.xy();
                (diff, diff.length())
            })
            .filter(|(_, dist)| *dist <= 4.0)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let Some((away_dir, dist)) = closest else {
            transform.rotation = Quat::IDENTITY;
            continue;
        };

        // 1.0 when player is right on top, 0.0 at 4 units away
        let proximity = 1.0 - (dist / 4.0);

        // Lean away horizontally
        let sway_x = away_dir.normalize_or_zero().x * proximity * 0.2;

        transform.rotation = Quat::from_axis_angle(Vec3::Z, -sway_x);
    }
}