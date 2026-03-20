use bevy::prelude::*;
#[allow(dead_code)]
#[derive(Component)]
pub struct MeleeWeapon {
    pub damage: f32,
    pub range: f32,
    pub animation: &'static str,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct Consumable {
    pub health: f32,
    pub hunger: f32,
}

// #[derive(Component)]
// struct Placeable {
//     // what to spawn in the world when placed
// }