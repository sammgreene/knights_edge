use bevy::prelude::*;
use crate::creatures::*;

pub fn near<T: Component>(
    In(entity): In<Entity>,
    transforms: Query<&Transform>,
    targets: Query<(Entity, &Transform), With<T>>,
) -> bool {
    let Ok(self_transform) = transforms.get(entity) else {
        return false;
    };

    let self_pos = self_transform.translation;
    let radius_sq = 6.0 * 6.0;

    for (target_entity, target_transform) in &targets {
        // Skip self if same entity
        if target_entity == entity {
            continue;
        }

        if self_pos.distance_squared(target_transform.translation) < radius_sq {
            return true;
        }
    }

    false
}

pub fn nearest<T: Component>(origin: &Vec3, targets: Query<(Entity, &Transform), With<T>>) -> Option<Entity> {
    // Now you have this entity's translation
    let mut nearest: Option<Entity> = None;
    for (target, target_transform) in &targets {
        if origin.distance_squared(target_transform.translation) < 6.0_f32.powf(2.0) {
            nearest = Some(target);
        }
    }
    nearest
}