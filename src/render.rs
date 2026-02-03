use bevy::prelude::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(Startup, ())
            .add_systems(Update,
                y_sort
            );
    }
}

#[derive(Component)]
pub struct YSort {
    pub layer: f32,
    pub y_offset: f32,
}
impl YSort {
    pub fn on_layer(layer: ZLayers) -> Self {
        Self {
            layer: layer as i32 as f32,
            y_offset: 0.0
        }
    }
    pub fn with_offset(self, y_offset: f32) -> Self {
        Self { layer: self.layer, y_offset: y_offset }
    }
}

pub fn y_sort(
    mut q: Query<(&GlobalTransform, &mut Transform, &YSort)>,
) {
    for (gt, mut tf, ysort) in &mut q {
        let world_y = gt.translation().y;
        tf.translation.z = ysort.layer - (world_y - ysort.y_offset as f32) * 0.001;
    }
}

pub enum ZLayers {
    Background,
    World,
    Foliage,
    UI,
}
