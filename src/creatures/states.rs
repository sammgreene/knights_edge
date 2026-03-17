use bevy::prelude::*;
use crate::creatures::*;
use crate::triggers;

#[derive(Clone, Component)]
pub struct Idle;
#[derive(Clone, Component)]
pub struct Sleep;
#[derive(Clone, Component)]
pub struct Eating;
#[derive(Clone, Component)]
pub struct Hunting;
#[derive(Copy, Clone, Component)]
pub struct Chasing {
    target: Entity
}

pub fn fox_hunting(
    mut commands: Commands,
    hares: Query<(Entity, &Transform), With<types::Hare>>,
    hunting_foxes: Query<(Entity, &Transform), (With<Hunting>, With<types::Fox>)>
) {
    for (fox, fox_transform) in hunting_foxes {
        // Fox just chooses the nearest hare
        // todo:
        //  - chase lowest health
        if let Some(target) = 
            triggers::nearest(&fox_transform.translation, hares)
        {
            commands.entity(fox)
                .remove::<Hunting>()
                .insert(Chasing { target: target } );
        };
    }
}
pub fn eye_hunting(
    mut commands: Commands,
    players: Query<(Entity, &Transform), With<crate::player::Player>>,
    hunting_eyes: Query<(Entity, &Transform), (With<Hunting>, With<types::Eye>)>
) {
    for (eye, transform) in hunting_eyes {
        // eye just chooses the nearest player
        // todo:
        //  - chase lowest health
        if let Some(target) = 
            triggers::nearest(&transform.translation, players)
        {
            commands.entity(eye)
                .remove::<Hunting>()
                .insert(Chasing { target: target } );
        };
    }
}

use crate::physics::{PhysicalTranslation, Velocity, PhysicsObject};

pub fn chasing(
    mut chasers: Query<(
        &mut Velocity,
        &PhysicalTranslation,
        &attributes::Speed,
        &Chasing
    ), With<PhysicsObject>>,
    targets: Query<&PhysicalTranslation>,
) {
    for (mut velocity, chaser_translation, speed, chasing) in &mut chasers {
        if let Ok(target_translation) = targets.get(chasing.target) {
            let direction = (target_translation.0 - chaser_translation.0).clamp_length_max(1.0);
            velocity.0 = direction * speed.0;
        } else {
            // Target is missing or dead, stop moving
            velocity.0 = Vec2::ZERO;
        }
    }
}

pub fn idle(
    mut idlers: Query<(&mut Velocity, &PhysicalTranslation, &attributes::Speed)>
) {
    for (mut velocity, idler_translation, speed) in &mut idlers {
        velocity.0 = Vec2::ZERO;
    }
}