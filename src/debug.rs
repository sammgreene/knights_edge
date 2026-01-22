use bevy::prelude::*;

#[derive(Resource)]
pub struct DebugMenu {
    pub ui_toggle: bool,
    // pub show_chunks: bool,
    // pub show_hitboxes: bool,
    // pub god_mode: bool,
    pub super_speed: bool,

    pub loaded_chunk_count: usize,
    pub total_spawned_entites: u32,
    pub player_coords: Vec2
}
impl Default for DebugMenu {
    fn default() -> Self {
        Self {
            ui_toggle: false,
            // show_chunks: true,
            // show_hitboxes: false,
            // god_mode: false,
            super_speed: false,

            loaded_chunk_count: 0,
            total_spawned_entites: 0,
            player_coords: Vec2::new(0.,0.)
        }
    }
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(DebugMenu::default())
            .add_systems(Startup, ui_setup)
            .add_systems(Update, (
                update_debug_menu,
                toggle_text_key,
                count_entities,
                //print_num_entities_with_component::<Sprite>,
            ))
            ;
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
    chunks_query: Query<&crate::world::world_generator::Chunk>
) {
    debug_menu.loaded_chunk_count = chunks_query.count();
    debug_menu.player_coords = Vec2::new(player_pos.translation.x, player_pos.translation.y);

    for (debug_text, mut text) in debug_ui_query.iter_mut() {
        match debug_text.name.as_str() {
            "loaded_chunk_count" => text.0 = format!("C: {}", debug_menu.loaded_chunk_count.to_string()),
            "total_entity_count" => text.0 = format!("BE: {}", debug_menu.total_spawned_entites),
            "player_position" => text.0 = format!("{}", debug_menu.player_coords),
            _ => ()
        }
    }
    if keys.just_pressed(KeyCode::Equal) {
        debug_menu.super_speed = !debug_menu.super_speed;
    }
}

fn toggle_text_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_menu: ResMut<DebugMenu>,
    mut query: Query<&mut Visibility, With<DebugText>>,
) {
    let mut visible = debug_menu.ui_toggle;
    if keys.just_pressed(KeyCode::F3) {
        visible = !visible;
        debug_menu.ui_toggle = visible;

        for mut v in &mut query {
            *v = if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

#[derive(Component)]
struct DebugText {
    name: String
}

fn ui_setup(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            ..default()
        },
        Text::new(
            "C: 0"
        ),
        DebugText { name: String::from("loaded_chunk_count") },
        Visibility::Hidden
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            left: Val::Px(8.0),
            ..default()
        },
        Text::new(
            "BE: 0"
        ),
        DebugText { name: String::from("total_entity_count") },
        Visibility::Hidden
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(64.0),
            left: Val::Px(8.0),
            ..default()
        },
        Text::new(
            "{}"
        ),
        DebugText { name: String::from("player_position") },
        Visibility::Hidden
    ));
}

// function to print count of number of entities that implement <T> component
pub fn _print_num_entities_with_component<T: Component + std::fmt::Debug>(entities: Query<(Entity, &T), With<T>> ) {
    info!("entities with component: {}", entities.count());
}