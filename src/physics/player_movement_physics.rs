use bevy::prelude::*;

/// A vector representing the player's input, accumulated over all frames that ran
/// since the last time the physics simulation was advanced.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct AccumulatedInput {
    // The player's movement input (WASD).
    pub movement: Vec2,
    // Other input that could make sense would be e.g.
    // boost: bool
}

#[derive(Component)]
#[require(AccumulatedInput)]
pub struct PlayerController;

/// Handle keyboard input and accumulate it in the `AccumulatedInput` component.
///
/// There are many strategies for how to handle all the input that happened since the last fixed timestep.
/// This is a very simple one: we just use the last available input.
/// That strategy works fine for us since the user continuously presses the input keys in this example.
/// If we had some kind of instantaneous action like activating a boost ability, we would need to remember that that input
/// was pressed at some point since the last fixed timestep.
pub fn accumulate_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    debug_menu: Res<crate::debug::DebugMenu>,
    player: Single<(&mut AccumulatedInput, &mut crate::physics::Velocity), With<PlayerController>>,
) {
    // Since Bevy's 3D renderer assumes SI units, this has the unit of meters per second.
    // Note that about 1.5 is the average walking speed of a human.
    let mut speed: f32 = 4.0;
    if debug_menu.super_speed {
        speed = 30.0;
    }
    let (mut input, mut velocity) = player.into_inner();
    // Reset the input to zero before reading the new input. As mentioned above, we can only do this
    // because this is continuously pressed by the user. Do not reset e.g. whether the user wants to boost.
    input.movement = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        input.movement.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        input.movement.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.movement.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.movement.x += 1.0;
    }

    // We need to normalize and scale because otherwise
    // diagonal movement would be faster than horizontal or vertical movement.
    // We use `clamp_length_max` instead of `.normalize_or_zero()` because gamepad input
    // may be smaller than 1.0 when the player is pushing the stick just a little bit.
    // if get_tile_at(world_map, chunks_query, x, y) == Some(WorldTile::Water) {

    // }
    velocity.0 = input.movement.clamp_length_max(1.0) * speed;
}

// Clear the input after it was processed in the fixed timestep.
pub fn clear_input(mut input: Single<&mut AccumulatedInput>) {
    **input = AccumulatedInput::default();
}