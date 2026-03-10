use bevy::{prelude::*, render};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(Startup, ())
            .add_systems(Update,(
                y_sort,
            ));
    }
}

#[derive(Component, Copy, Clone, PartialEq)]
pub enum RenderLayer {
    Background = 0,
    World = 1,
    FoliageFront = 3,
    FoliageBack = 2,
    UI = 5,
}

pub fn y_sort(
    mut q: Query<(&GlobalTransform, &mut Transform, &RenderLayer, &Sprite), Without<crate::player::Player>>,
    mut players: Query<(&mut Transform, &RenderLayer), With<crate::player::Player>>
) {
    for (gt, mut tf, ysort, sprite) in &mut q {
        let world_y = gt.translation().y;

        let sprite_height = sprite
            .custom_size
            .map(|size| size.y)
            .unwrap_or(0.0);

        tf.translation.z =
            (ysort.clone() as i32 as f32) - (world_y - sprite_height) * 0.001;
    }
    for (mut player, layer) in &mut players {
        player.translation.z = *layer as i32 as f32 + 1.0
    }
}