use bevy::prelude::*;
pub mod player_movement_physics;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DidFixedTimestepRunThisFrame>()
            // At the beginning of each frame, clear the flag that indicates whether the fixed timestep has run this frame.
            .add_systems(PreUpdate, (clear_fixed_timestep_flag, initialize_physics_objects))
            // At the beginning of each fixed timestep, set the flag that indicates whether the fixed timestep has run this frame.
            .add_systems(FixedPreUpdate, set_fixed_time_step_flag)
            // Advance the physics simulation using a fixed timestep.
            .add_systems(FixedUpdate, advance_physics)
            .add_systems(
                // The `RunFixedMainLoop` schedule allows us to schedule systems to run before and after the fixed timestep loop.
                RunFixedMainLoop,
                (
                    (
                        // Accumulate our input before the fixed timestep loop to tell the physics simulation what it should do during the fixed timestep.
                        player_movement_physics::accumulate_player_input,
                    )
                        .chain()
                        .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
                    (
                        // Clear our accumulated input after it was processed during the fixed timestep.
                        // By clearing the input *after* the fixed timestep, we can still use `AccumulatedInput` inside `FixedUpdate` if we need it.
                        player_movement_physics::clear_input.run_if(did_fixed_timestep_run_this_frame),
                        // The player's visual representation needs to be updated after the physics simulation has been advanced.
                        // This could be run in `Update`, but if we run it here instead, the systems in `Update`
                        // will be working with the `Transform` that will actually be shown on screen.
                        interpolate_rendered_transform,
                    )
                        .chain()
                        .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
                ),
            );
    }
}

// Resources
/// A simple resource that tells us whether the fixed timestep ran this frame.
#[derive(Resource, Debug, Deref, DerefMut, Default)]
pub struct DidFixedTimestepRunThisFrame(bool);


// Components

/// A vector representing the player's velocity in the physics simulation.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

/// The actual position of the player in the physics simulation.
/// This is separate from the `Transform`, which is merely a visual representation.
///
/// If you want to make sure that this component is always initialized
/// with the same value as the `Transform`'s translation, you can
/// use a [component lifecycle hook](https://docs.rs/bevy/0.14.0/bevy/ecs/component/struct.ComponentHooks.html)
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct PhysicalTranslation(pub Vec2);

/// The value [`PhysicalTranslation`] had in the last fixed timestep.
/// Used for interpolation in the `interpolate_rendered_transform` system.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct PreviousPhysicalTranslation(Vec2);

// My own requiring component to avoid repeating multiple components in system queries
#[derive(Component, Default)]
#[require(PhysicalTranslation, PreviousPhysicalTranslation, Velocity)]
pub struct PhysicsObject;

/// Reset the flag at the start of every frame.
fn clear_fixed_timestep_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = false;
}

/// Set the flag during each fixed timestep.
fn set_fixed_time_step_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = true;
}

fn did_fixed_timestep_run_this_frame(
    did_fixed_timestep_run_this_frame: Res<DidFixedTimestepRunThisFrame>,
) -> bool {
    did_fixed_timestep_run_this_frame.0
}


// Systems

/// Advance the physics simulation by one fixed timestep. This may run zero or multiple times per frame.
///
/// Note that since this runs in `FixedUpdate`, `Res<Time>` would be `Res<Time<Fixed>>` automatically.
/// We are being explicit here for clarity.
fn advance_physics(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut PhysicalTranslation,
        &mut PreviousPhysicalTranslation,
        &Velocity,
    )>,
) {
    for (mut current_physical_translation, mut previous_physical_translation, velocity) in
        query.iter_mut()
    {
        previous_physical_translation.0 = current_physical_translation.0;
        current_physical_translation.0 += velocity.0 * fixed_time.delta_secs();
    }
}

fn interpolate_rendered_transform(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut Transform,
        &PhysicalTranslation,
        &PreviousPhysicalTranslation),
        // With<crate::player::Player>
    >,
) {
    for (mut transform, current_physical_translation, previous_physical_translation) in
        query.iter_mut()
    {
        let previous = previous_physical_translation.0;
        let current = current_physical_translation.0;
        // The overstep fraction is a value between 0 and 1 that tells us how far we are between two fixed timesteps.
        let alpha = fixed_time.overstep_fraction();

        let rendered_translation = previous.lerp(current, alpha);
        transform.translation.x = rendered_translation.x;
        transform.translation.y = rendered_translation.y;
    }
}

fn initialize_physics_objects(
    mut query: Query<(&Transform, &mut PhysicalTranslation, &mut PreviousPhysicalTranslation), Added<PhysicsObject>>
) {
    for (transform, mut phys, mut prev) in &mut query {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);
        phys.0 = pos;
        prev.0 = pos;
    }
}