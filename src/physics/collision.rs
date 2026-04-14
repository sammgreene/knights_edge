use bevy::prelude::*;
use crate::physics::*;

#[derive(Component)]
pub struct Collider {
    kind: ColliderShape,
    rigidity: f32 // 0 .. 1 (0 disables collisions, 1 is a perfect rigid body that pushes all bodies out)
}
impl Collider {
    /// Creates a circle collider with *radius* and *rigidity*
    pub fn circle(radius: f32, rigidity: f32) -> Self {
        Self {
            kind: ColliderShape::Circle(radius),
            rigidity
        }
    }
}
pub enum ColliderShape {
    Circle(f32),
    // Rect(f32,f32)
}

#[derive(Component)]
pub struct StaticBody;
#[derive(Component)]
pub struct DynamicBody {
    mass: f32
}
impl DynamicBody {
    pub fn new(mass: f32) -> Self {
        Self { mass }
    }
}

/// #### Bevy System
/// Displays a message in console warning for missed colliders
pub fn flag_bodies_without_colliders(
    static_bodies: Query<Entity, (Added<StaticBody>, Without<Collider>)>,
    dynamic_bodies: Query<Entity, (Added<DynamicBody>, Without<Collider>)>,
) {
    for entity in &static_bodies {
        warn!("StaticBody {:?} is missing a Collider component", entity);
    }
    for entity in &dynamic_bodies {
        warn!("DynamicBody {:?} is missing a Collider component", entity);
    }
}
/// #### Bevy System
/// Displays a message in console warning for missed bodies
pub fn flag_colliders_without_bodies(
    colliders: Query<Entity, (Added<Collider>, Without<StaticBody>, Without<DynamicBody>)>,
) {
    for entity in &colliders {
        warn!("Entity {:?} has a Collider but no StaticBody or DynamicBody", entity);
    }
}

// a wall would be a StaticBody with rigidity of 1
// an entity like a sheep would be a DynamicBody with rigidity of 0.5 and a little less mass than a player
pub fn resolve_static_collisions(
    mut dynamics: Query<(&mut PhysicalTranslation, &mut PreviousPhysicalTranslation, &mut Velocity, &Collider), Without<StaticBody>>,
    statics: Query<(&GlobalTransform, &Collider), With<StaticBody>>
) {
    // info!("COLLISION");
    // push dynamics out of statics (ignore masses)
    for (mut dyn_pos, mut dyn_prev_pos, mut dyn_velo, dyn_collider) in &mut dynamics {
        for (stat_transform, stat_collider) in &statics {
            let stat_pos = stat_transform.translation().xy();
            let Some(push) = compute_push(dyn_pos.0, &dyn_collider.kind, stat_pos, &stat_collider.kind) else {
                continue;
            };
            // push the dynamic entity
            let effective_push = push * dyn_collider.rigidity * stat_collider.rigidity;
            dyn_pos.0 += effective_push;
            dyn_prev_pos.0 += effective_push;

            // kill dynamic velocity in push dir
            let normal = effective_push.normalize_or_zero();
            let into_wall = dyn_velo.0.dot(normal).min(0.0);
            dyn_velo.0 -= normal * into_wall;
        }
    }
}

pub fn resolve_dynamic_collisions(
    mut dynamics: Query<(&mut PhysicalTranslation, &mut PreviousPhysicalTranslation, &mut Velocity, &Collider, &DynamicBody), Without<StaticBody>>,
) {
    let mut combinations = dynamics.iter_combinations_mut();
    while let Some([
        (mut pos_a, mut prev_a, mut vel_a, col_a, body_a),
        (mut pos_b, mut prev_b, mut vel_b, col_b, body_b),
    ]) = combinations.fetch_next()
    {
        let Some(push) = compute_push(pos_a.0, &col_a.kind, pos_b.0, &col_b.kind) else {
            continue;
        };
        let effective = push * col_a.rigidity * col_b.rigidity;
        let total_mass = body_a.mass + body_b.mass;
        let push_a = effective * (body_b.mass / total_mass);
        let push_b = effective * (body_a.mass / total_mass);
        pos_a.0 += push_a;
        prev_a.0 += push_a;
        pos_b.0 -= push_b;
        prev_b.0 -= push_b;
        // transfer velocity along push axis
        let normal = effective.normalize_or_zero();
        let vel_diff = vel_a.0.dot(normal) - vel_b.0.dot(normal);
        if vel_diff > 0.0 {
            let impulse = normal * vel_diff;
            vel_a.0 -= impulse * (body_b.mass / total_mass);
            vel_b.0 += impulse * (body_a.mass / total_mass);
        }
    }
}

/// Returns the push vector to move `pos_a` out of `pos_b`, or None if not overlapping.
fn compute_push(pos_a: Vec2, shape_a: &ColliderShape, pos_b: Vec2, shape_b: &ColliderShape) -> Option<Vec2> {
    match (shape_a, shape_b) {
        (ColliderShape::Circle(r_a), ColliderShape::Circle(r_b)) => {
            let diff = pos_a - pos_b;
            let dist = diff.length();
            let min_dist = r_a + r_b;
            if dist < min_dist && dist > 0.0 {
                Some(diff.normalize() * (min_dist - dist))
            } else {
                None
            }
        }
        // (_,_) => None
    }
}

pub fn spawn_debug_label(
    mut commands: Commands,
    dynamics: Query<Entity, Added<DynamicBody>>,
) {
    for entity in &dynamics {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(""),
                TextFont { font_size: 21.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, 1.0, 100.0).with_scale(vec3(0.02,0.02,1.0)), // above the entity
                DebugInfoLabel,
            ));
        });
    }
}

#[derive(Component)]
pub struct DebugInfoLabel;

pub fn update_debug_labels(
    dynamics: Query<(&Velocity, &Children), With<DynamicBody>>,
    mut labels: Query<&mut Text2d, With<DebugInfoLabel>>,
) {
    for (vel, children) in &dynamics {
        for child in children {
            if let Ok(mut text) = labels.get_mut(*child) {
                text.0 = format!("{:.2}", vel.0.length());
            }
        }
    }
}