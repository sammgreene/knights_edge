use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Health(i8);

#[derive(Component, Clone, Copy)]
pub struct Speed(pub f32);
impl Default for Speed {
    fn default() -> Self {
        Self(1.0)
    }
}