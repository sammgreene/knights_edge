use core::f32;
use bevy::prelude::*;

pub fn get_nearest_entity_with<T: Component>(origin: &Vec3, targets: Query<(Entity, &Transform), With<T>>) -> Option<Entity> {
    // Now you have this entity's translation
    let mut nearest: Option<Entity> = None;
    let mut nearest_distance = f32::INFINITY;
    for (target, target_transform) in &targets {
        let dist_sq = origin.distance_squared(target_transform.translation);
        if dist_sq < nearest_distance {
            nearest = Some(target);
            nearest_distance = dist_sq;
        }
    }
    nearest
}