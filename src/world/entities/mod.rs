use bevy::prelude::*;

#[derive(Component)]
#[require(crate::physics::PhysicsObject)]
struct Entity {
    ai: Option<AI>
}

struct AI {
    kind: AIType
}

enum AIType {
    Fox,
    Hare
}