use bevy::prelude::*;
pub mod grass_sway;
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(Startup, ())
            .add_systems(Update,(
                y_sort,
                update_foliage_occlusion,
                grass_sway::player_grass_sway_system
            ));
    }
}

#[derive(Component, Copy, Clone, PartialEq)]
pub enum RenderLayer {
    // Background = 0,
    World = 1,
    Foliage = 2,
    UI = 5,
}
#[derive(Component)]
pub struct SortOffset(pub f32);

#[derive(Component)]
pub struct OccludesPlayer;

pub fn y_sort(
    q: Query<(&GlobalTransform, &mut Transform, &RenderLayer, Option<&SortOffset>), Without<crate::player::Player>>,
    players: Query<(&GlobalTransform, &mut Transform, &RenderLayer, Option<&SortOffset>), With<crate::player::Player>>,
) {
    for (gt, mut tf, layer, sort_offset) in q {
        let world_y = gt.translation().y;
        let offset = sort_offset.map(|s| s.0).unwrap_or(0.0);
        tf.translation.z = (layer.clone() as i32 as f32) - (world_y + offset) * 0.001;
    }

    for (gt, mut tf, layer, sort_offset) in players {
        let world_y = gt.translation().y;
        let foot_offset = sort_offset.map(|s| s.0).unwrap_or(0.2);
        tf.translation.z = (layer.clone() as i32 as f32) - (world_y - foot_offset) * 0.001;
    }
}

fn update_foliage_occlusion(
    players: Query<(&GlobalTransform, &crate::player::VisionProfile), With<crate::player::Player>>,
    mut foliage: Query<(&GlobalTransform, &mut Sprite), With<OccludesPlayer>>,
) {
    for (_, mut sprite) in &mut foliage {
        sprite.color.set_alpha(1.0);
    }

    for (player_gt, vision) in &players {
        let player_pos = player_gt.translation().truncate();

        for (gt, mut sprite) in &mut foliage {
            let tree_pos = gt.translation().truncate();
            let tree_center_y = tree_pos.y + sprite.custom_size.unwrap().y / 2.;

            let dx = (player_pos.y - tree_center_y).abs();
            let dy_below = (player_pos.x - tree_pos.x).min(0.0); // negative when below base
            let dy_above = (player_pos.x - tree_pos.x).max(0.0); // positive when above base

            // Fade out occlusion as player moves above the tree base
            let above_falloff = (dy_above / vision.occlusion_radius).clamp(0.0, 1.0);
            let above_falloff_smooth = above_falloff * above_falloff * (3.0 - 2.0 * above_falloff);

            let dist = Vec2::new(dx, dy_below.abs()).length();
            let t = (dist / vision.occlusion_radius).clamp(0.0, 1.0);
            let t_steep = t.powf(5.0);
            let t_smooth = t_steep * t_steep * (3.0 - 2.0 * t_steep);

            // Blend between occlusion alpha and 1.0 based on how far above we are
            let occluded_alpha = vision.min_alpha.lerp(vision.max_alpha, t_smooth);
            let alpha = occluded_alpha.lerp(1.0, above_falloff_smooth);

            let current = sprite.color.alpha();
            sprite.color.set_alpha(current.min(alpha));
        }
    }
}