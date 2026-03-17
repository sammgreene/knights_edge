use bevy::prelude::*;
use dynfmt::{Format, SimpleCurlyFormat};

use crate::{player::PlayerMarker, world::world_generation};

#[derive(Resource)]
pub struct DebugMenu {
    pub render: bool,
    ui_line_count: usize,
    // pub show_chunks: bool,
    // pub show_hitboxes: bool,
    // pub god_mode: bool,
    pub super_speed: bool,

    pub loaded_chunk_count: usize,
    pub total_spawned_entites: u32,
    pub player_coords: Vec3,
    // pub current_biome: world_generation::Biome
}
impl DebugMenu {
    fn default() -> Self {
        Self {
            render: false,
            ui_line_count: 0,
            
            // show_chunks: true,
            // show_hitboxes: false,
            // god_mode: false,
            super_speed: false,

            loaded_chunk_count: 0,
            total_spawned_entites: 0,
            player_coords: Vec3::new(0.,0., 0.),
            // current_biome: world_generation::Biome::Forest,
        }
    }
    fn with_enabled(mut self) -> Self {
        self.render = true;
        self
    }
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(DebugMenu::default().with_enabled())
            .add_systems(Startup, ui_setup)
            .add_systems(Update, (
                update_debug_menu,
                toggle_menu,
                count_entities,
                debug_event_playground
                //print_num_entities_with_component::<Sprite>,
            ));
    }
}

fn count_entities(world: &mut World) {
    let count = world.entities().len();

    let mut debug_menu = world.resource_mut::<DebugMenu>();
    debug_menu.total_spawned_entites = count;
}

fn update_debug_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_menu: ResMut<DebugMenu>,
    mut debug_ui_query: Query<(&DebugText, &mut Text)>,
    player_pos: Single<&Transform, With<crate::player::Player>>,
    chunks_query: Query<&crate::world::world_generation::Chunk>,
    noise: Res<crate::world::world_noise::WorldNoise>,
) {
    let (player_x, player_y) = (player_pos.translation.x, player_pos.translation.y);

    debug_menu.player_coords = Vec3::new(
        player_x, 
        player_y, 
        noise.get_altitude(player_x, player_y)
    );
    debug_menu.loaded_chunk_count = chunks_query.count();

    let p = debug_menu.player_coords;
    let coords = format!("{:.1}, {:.1}, {:.1}", p.x, p.y, p.z);

    for (debug_text, mut text) in debug_ui_query.iter_mut() {
        match debug_text.name.as_str() {
            "loaded_chunk_count" =>
                text.0 = SimpleCurlyFormat.format(&debug_text.text, &[debug_menu.loaded_chunk_count]).unwrap().to_string(),
            "total_entity_count" => 
                text.0 = SimpleCurlyFormat.format(&debug_text.text, &[debug_menu.total_spawned_entites]).unwrap().to_string(),
            "player_position" => 
                text.0 = SimpleCurlyFormat.format(&debug_text.text, &[&coords]).unwrap().to_string(),
            _ => ()
        }
    }
    if keys.just_pressed(KeyCode::Equal) {
        debug_menu.super_speed = !debug_menu.super_speed;
    }
}

fn toggle_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_menu: ResMut<DebugMenu>,
    mut text: Query<&mut Visibility, With<DebugText>>,
    mut player_marker: Query<&mut Visibility, (With<PlayerMarker>, Without<DebugText>)>
) {
    let mut visible = debug_menu.render;
    if keys.just_pressed(KeyCode::F3) {
        visible = !visible;
        debug_menu.render = visible;

        for mut v in &mut text {
            *v = if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        for mut player_marker in &mut player_marker {
            *player_marker = if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

#[derive(Component)]
struct DebugText {
    name: String,
    text: String
}

fn ui_setup(mut commands: Commands, mut debug_menu: ResMut<DebugMenu>) {
    new_debug_text("loaded_chunk_count", "C: {}", &mut commands, &mut debug_menu);
    new_debug_text("total_entity_count", "BE: {}", &mut commands, &mut debug_menu);
    new_debug_text("player_position", "{}", &mut commands, &mut debug_menu);
}

// function to print count of number of entities that implement <T> component
pub fn _print_num_entities_with_component<T: Component + std::fmt::Debug>(entities: Query<(Entity, &T), With<T>> ) {
    info!("entities with component: {}", entities.count());
}

fn new_debug_text(name: &str, text: &str, commands: &mut Commands, debug_menu: &mut ResMut<DebugMenu>) {
    let visibility = if debug_menu.render {Visibility::Visible} else {Visibility::Hidden};
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(debug_menu.ui_line_count as f32 * 25.0),
            left: Val::Px(8.0),
            ..default()
        },
        Text::new(
            text
        ),
        DebugText { name: String::from(name), text: String::from(text) },
        TextColor::WHITE,
        visibility
    ));
    debug_menu.ui_line_count += 1;
}

fn debug_event_playground(
    world_map: Res<world_generation::WorldMap>,
    player_pos: Single<&Transform, With<crate::player::Player>>,
    player_inventory: Single<&crate::items::inventory::Inventory, With<crate::player::Player>>,
    item_stacks: Query<&crate::items::ItemStack>,
    chunks_query: Query<&crate::world::world_generation::Chunk>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Backslash) {
        let (player_x, player_y) = (player_pos.translation.x.floor() as i32, player_pos.translation.y.floor() as i32);
        info!("{:?}{:?}", (player_x, player_y), world_generation::get_tile_at(world_map, chunks_query, player_x, player_y));
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        for entity in &player_inventory.items {
            match item_stacks.get(*entity) {
                Ok(stack) => info!("{:?}", stack),
                Err(_) => info!("entity {:?} has no ItemStack", entity),
            }
        }
    }
}