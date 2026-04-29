use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

mod data;
use data::*;
mod ai;
mod spawning;

pub struct CreatureDataPlugin;

impl Plugin for CreatureDataPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<CreatureDatabase>()
            .init_asset_loader::<CreatureDatabaseLoader>()
            .add_systems(Startup, (
                load_creature_database,
                spawning::spawn_randoms,
            ))
            .add_systems(Update, (
                on_database_loaded.run_if(resource_exists::<CreatureDatabaseHandle>),
                ai::transition_goals.run_if(on_timer(Duration::from_secs(3))),
                ai::assign_goals_and_tasks_to_new_creature_entities,
                ai::spawn_ai_debug_label,
                ai::update_ai_state_labels,
                spawn_creature_status_debug_label,
                update_creature_state_labels,
                ai::task_go_to,
                ai::task_locate_random,
                ai::transition_tasks
            ).chain());
    }
}

fn load_creature_database(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let handle = asset_server.load("configs/creatures.ron");
    commands.insert_resource(CreatureDatabaseHandle(handle));
}

fn on_database_loaded(
    handle: Res<CreatureDatabaseHandle>,
    databases: ResMut<Assets<CreatureDatabase>>,
    mut commands: Commands,
) {
    // Once loaded, you can pull the data out and build whatever runtime
    // structures you need (e.g. a HashMap<String, CreatureDataRaw>)
    if let Some(db) = databases.get(&handle.0) {
        info!("Creature database loaded: {} entries", db.entries.len());

        info!("Loading relations resource...");
        for entry in &db.entries {
            info!("  {} preys on {:?}", entry.kind, entry.prey);
        }
        let relations = CreatureRelations::build(db);
        commands.insert_resource(relations);
        commands.remove_resource::<CreatureDatabaseHandle>();

        info!("Loading database resource");
        commands.insert_resource(db.clone());
    }
}

#[derive(Component)]
pub struct CreatureMemory {
    pub food_location: Option<Vec2>,
    pub nest_location: Option<Vec2>,
    pub predator_location: Option<Vec2>,
    pub mate: Option<Entity>
}
impl CreatureMemory {
    fn default() -> Self {
        Self {
            food_location: None,
            nest_location: None,
            predator_location: None,
            mate: None
        }
    }
}

#[derive(Component, Debug)]
pub struct CreatureState {
    pub health: Quantity,
    pub stamina: Quantity, 
    pub saturation: Quantity,
    pub fear: Quantity
}
impl CreatureState {
    fn default() -> Self {
        Self {
            health: Quantity::new(10.0, 10.0),
            stamina: Quantity::new(10.0, 10.0),
            saturation: Quantity::new(10.0, 10.0),
            fear: Quantity::new(0.0, 10.0)
        }
    }
}

#[derive(Component)]
pub struct Creature(String);

#[derive(Debug)]
pub struct Quantity {
    value: f32,
    max: f32,
    min: f32
}
impl Quantity {
    fn new(value: f32, max: f32) -> Self {
        Self {
            value,
            max,
            min: 0.0
        }
    }
    fn fraction(&self) -> f32 {
        return (self.value/self.max).clamp(0.0, 1.0)
    }
    fn is_zero(&self) -> bool {
        if self.value < self.min { warn!("Qauntity: {} is less than minimum: {}", self.value, self.min); }
        self.value <= f32::EPSILON
    }
    fn is_max(&self) -> bool {
        if self.value > self.max { warn!("Quantity: {} is more than maximum: {}", self.value, self.max); }
        self.value >= self.max
    }
    pub fn decrease(&mut self, amount: f32) {
        if self.value - amount < self.min {
            self.value = self.min
        } else {
            self.value -= amount;
        }
    }
}

pub fn spawn_creature_status_debug_label(
    mut commands: Commands,
    items: Query<Entity, Added<CreatureState>>,
) {
    for entity in &items {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(""),
                crate::debug::DebugItem,
                TextFont { font_size: 21.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, -1.0, 100.0).with_scale(vec3(0.02, 0.02, 1.0)),
                CreatureStateLabel,
            ));
        });
    }
}

#[derive(Component)]
pub struct CreatureStateLabel;

pub fn update_creature_state_labels(
    items: Query<(&CreatureState, &Children)>,
    mut labels: Query<&mut Text2d, With<CreatureStateLabel>>,
) {
    for (state, children) in &items {
        for child in children {
            if let Ok(mut text) = labels.get_mut(*child) {
                text.0 = format!("health: {:?}\nstamina: {:?}\nsaturation: {:?}\nfear: {:?}\n", state.health, state.stamina, state.saturation, state.fear);
            }
        }
    }
}