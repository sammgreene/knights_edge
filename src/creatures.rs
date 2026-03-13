pub mod types;
pub mod states;
pub mod attributes;

use bevy::prelude::*;
use seldom_state::prelude::*;

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_systems(Startup, )
            .add_plugins(StateMachinePlugin::default())
            .add_systems(Update, (
                states::chasing,
                states::idle,
                states::eye_hunting,
                states::fox_hunting,
            ));
    }
}