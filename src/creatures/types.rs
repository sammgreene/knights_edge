use bevy::prelude::*;
use seldom_state::prelude::*;
use crate::creatures::*;

#[derive(Component)]
#[require(attributes::Health, attributes::Speed)]
pub struct Fox;

#[derive(Component)]
#[require(attributes::Health, attributes::Speed)]
pub struct Hare;

#[derive(Component)]
#[require(attributes::Health, attributes::Speed)]
pub struct Eye;

pub fn fox_state_machine() -> StateMachine {
    StateMachine::default()
        .trans::<states::Idle, _>(triggers::near::<types::Hare>, states::Hunting)
        .trans::<states::Hunting, _>(triggers::near::<types::Hare>.not(), states::Idle)
        .trans::<states::Chasing, _>(triggers::near::<types::Hare>.not(), states::Idle)
        .set_trans_logging(true)
}
pub fn eye_state_machine() -> StateMachine {
    StateMachine::default()
        .trans::<states::Idle, _>(triggers::near::<crate::player::Player>, states::Hunting)
        .trans::<states::Hunting, _>(triggers::near::<crate::player::Player>.not(), states::Idle)
        .trans::<states::Chasing, _>(triggers::near::<crate::player::Player>.not(), states::Idle)
        .set_trans_logging(true)
}