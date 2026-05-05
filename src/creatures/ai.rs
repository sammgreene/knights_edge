use bevy::prelude::*;
use crate::world::world_generation::*;
use crate::{creatures::*, physics::Velocity};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub enum Goal {
    Wander,   // locate random location, go to location
    GatherIntel,
    Feed,     // locate food, go to food, eat food

    Hunt,     // locate prey, go to prey transition to attack
    Sleep,    // remember nest, go to nest, sleep
    Nest,     // locate suitable nest, go to location, set nest
    Stash,    // collect food, go to nest, stash food
    Retrieve, // locate nest, go to nest, retrieve food, eat food
    Mate,     // locate mate, go to mate, mate
    Follow,   // locate target, follow target
    
    // together with evade creates the attacking system
    Attack,   // rush, attack
    // evade goal is used in tandem with attack during cooldowns
    Evade,    // retreat, maintain distance
}
impl Goal {
    pub fn tasks_for_goal(self) -> &'static [Task] {
        match self {
            Goal::Wander =>     &[Task::LocateRandom, Task::GoTo],
            Goal::GatherIntel =>&[Task::LocateFood, Task::LocateMate],
            Goal::Feed =>       &[Task::LocateFood, Task::GoTo, Task::CollectFood, Task::Eat],
            Goal::Hunt =>       &[Task::LocatePrey, Task::Chase, Task::Attack],
            Goal::Sleep =>      &[Task::LocateNest, Task::GoTo, Task::Sleep],
            Goal::Nest =>       &[Task::LocateProspectiveNest, Task::GoTo, Task::RememberNest],
            Goal::Stash =>      &[Task::LocateNest, Task::GoTo, Task::StashFood],
            Goal::Retrieve =>   &[Task::LocateNest, Task::GoTo, Task::RetrieveFood, Task::Eat],
            Goal::Mate =>       &[Task::LocateMate, Task::GoTo, Task::Mate],
            Goal::Follow =>     &[Task::LocateTarget, Task::Chase],
            Goal::Attack =>     &[Task::Rush, Task::Strike],
            Goal::Evade =>      &[Task::Retreat, Task::MaintainDistance],
        }
    }
}

#[derive(Component)]
pub struct GoalProgress(usize);

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum Task {
    LocateFood,
    LocatePrey,
    LocateProspectiveNest,
    LocateMate,
    LocateTarget,
    LocateRandom,
    LocateNest,

    RememberNest, // maybe not

    // GoToFood,
    // GoToPrey,
    // GoToNest,
    // GoToProspectiveNest,
    // GoToMate,
    // GoToTarget,
    // GoToRandom,
    GoTo,

    Eat,
    Sleep,
    CollectFood,
    StashFood,
    RetrieveFood,
    Mate,

    Attack,
    Chase,

    Rush,
    Strike,
    Retreat,
    MaintainDistance
}
impl Task {
    /// Returns how long this task is allowed to run in virtual game time.
    /// None means no limit (runs until it self-reports TaskFinished).
    pub fn time_limit(&self) -> Option<Duration> {
        match self {
            // Locate tasks: short window — if nothing found, move on
            Task::LocateFood            => Some(Duration::from_secs(4)),
            Task::LocatePrey            => Some(Duration::from_secs(4)),
            Task::LocateMate            => Some(Duration::from_secs(4)),
            Task::LocateProspectiveNest => Some(Duration::from_secs(6)),
            Task::LocateTarget          => Some(Duration::from_secs(3)),
            Task::LocateNest            => Some(Duration::from_secs(5)),
            Task::LocateRandom          => Some(Duration::from_millis(500)),

            // Travel tasks: longer window before giving up
            Task::GoTo                  => Some(Duration::from_secs(20)),
            Task::Chase                 => Some(Duration::from_secs(10)),
            Task::Rush                  => Some(Duration::from_secs(10)),

            // Action tasks
            Task::Sleep                 => Some(Duration::from_secs(15)),
            Task::Eat                   => Some(Duration::from_secs(5)),
            Task::CollectFood           => Some(Duration::from_secs(3)),
            Task::StashFood             => Some(Duration::from_secs(3)),
            Task::RetrieveFood          => Some(Duration::from_secs(3)),
            Task::Mate                  => Some(Duration::from_secs(5)),
            Task::Strike                => Some(Duration::from_secs(3)),
            Task::Retreat               => Some(Duration::from_secs(8)),
            Task::MaintainDistance      => Some(Duration::from_secs(8)),
            Task::Attack                => Some(Duration::from_secs(4)),

            // Instant / no limit needed
            Task::RememberNest          => None,
        }
    }

    /// If true, timing out abandons the whole goal (GoalFinished).
    /// If false, timing out just skips to the next task (TaskFinished).
    pub fn timeout_fails_goal(&self) -> bool {
        matches!(self,
            Task::GoTo          // couldn't reach destination → whole goal stale
            | Task::Rush        // couldn't close gap → goal re-evaluated
            | Task::Chase
        )
    }
}

// Systems
#[derive(Component)]
pub struct TaskFinished;
#[derive(Component)]
pub struct GoalFinished;

// Stores the virtual elapsed time at which this task expires.
#[derive(Component)]
pub struct TaskTimeLimit(pub Duration); // deadline = Time::<Virtual>::elapsed() at expiry

// assign a fresh deadline whenever the active Task changes
pub fn assign_task_deadlines(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    changed: Query<(Entity, &Task), Changed<Task>>,
) {
    for (entity, task) in &changed {
        // Always clear the old deadline first
        commands.entity(entity).remove::<TaskTimeLimit>();

        if let Some(duration) = task.time_limit() {
            let deadline = time.elapsed() + duration;
            commands.entity(entity).insert(TaskTimeLimit(deadline));
        }
    }
}

// check deadlines every frame, emit Finished/Failed
pub fn check_task_deadlines(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    entities: Query<
        (Entity, &Task, &TaskTimeLimit),
        (Without<TaskFinished>, Without<GoalFinished>),
    >,
) {
    let now = time.elapsed();
    for (entity, task, deadline) in &entities {
        if now >= deadline.0 {
            let mut cmd = commands.entity(entity);
            cmd.remove::<TaskTimeLimit>();

            if task.timeout_fails_goal() {
                info!("Task {:?} timed out — abandoning goal", task);
                cmd.insert(GoalFinished); // transition_goals will pick a new goal
            } else {
                info!("Task {:?} timed out — skipping to next task", task);
                cmd.insert(TaskFinished); // transition_tasks advances the task index
            }
        }
    }
}

pub fn transition_tasks(
    creatures: Query<(Entity, &mut Task, &Goal, &mut GoalProgress), (With<TaskFinished>, Without<GoalFinished>)>,
    mut commands: Commands,
) {
    for (entity, mut task, goal, mut progress) in creatures {
        progress.0 += 1;
        let tasks = goal.tasks_for_goal();

        let mut cmd = commands.entity(entity);
        cmd.remove::<TaskFinished>()
           .remove::<TaskTimeLimit>(); // clear stale deadline; assign_task_deadlines
                                       // will set a new one via Changed<Task>

        if progress.0 >= tasks.len() {
            cmd.insert(GoalFinished);
        } else {
            *task = tasks[progress.0];
        }
    }
}

pub fn transition_goals(
    creatures: Query<(Entity, &Creature, &CreatureState, &CreatureMemory, &mut Task, &mut Goal, &mut GoalProgress), With<GoalFinished>>,
    creatures_data: Res<CreatureDatabase>,
    creature_relations: Res<CreatureRelations>,
    mut commands: Commands
) {
    // info!("transitioning {:?} creatures", creatures.count());
    for (entity, creature, creature_state, creature_memory,  mut task, mut goal, mut progress) in creatures {
        let creature_data = creatures_data.entries.iter().find(|x| x.kind == creature.0).unwrap();

        let new_goal = get_new_goal(&goal, creature, creature_state, creature_memory, &creature_data, &creature_relations);
        *goal = new_goal;
        progress.0 = 0; // start at first task in goal
        *task = new_goal.tasks_for_goal()[progress.0];
        commands.entity(entity).remove::<GoalFinished>();
    }
}

fn get_new_goal(
    current_goal: &Goal,
    creature: &Creature,
    creature_state: &CreatureState,
    creature_memory: &CreatureMemory,
    creature_data: &CreatureDataRaw,
    creature_relations: &CreatureRelations,
) -> Goal {
    let fear       = creature_state.fear.fraction();
    let stamina    = creature_state.stamina.fraction();
    let health     = creature_state.health.fraction();
    let saturation = creature_state.saturation.fraction();

    let evade_score   = fear                  * (1.0 / creature_data.bravery);
    let sleep_score   = (1.0 - stamina)       * (1.0 / creature_data.endurance);
    let nest_score    = (1.0 - health)        * (1.0 / creature_data.robustness);
    let hunger_score  = (1.0 - saturation)    * (1.0 / creature_data.appetite);

    // GatherIntel wins when nothing urgent is happening.
    // urgency is the strongest competing drive; contentment is its inverse.
    let urgency = evade_score
        .max(sleep_score)
        .max(nest_score)
        .max(hunger_score);
    let gather_intel_score = (1.0 - urgency) * 0.1;
    let wander_score       = (1.0 - urgency) * (rand::random::<f32>() * 0.5); // 0.0–0.25

    let scores: &[(Goal, f32)] = &[
        (Goal::Evade,       evade_score),
        (Goal::Sleep,       sleep_score),
        (Goal::Nest,        nest_score),
        (Goal::GatherIntel, gather_intel_score),
        (Goal::Wander,      wander_score),
        (
            if creature_relations.is_predator(&creature.0) { Goal::Hunt } else { Goal::Feed },
            hunger_score,
        ),
    ];

    let new_goal = scores
        .iter()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .map(|(goal, _)| *goal)
        .unwrap_or(Goal::Wander);

    if new_goal != *current_goal {
        info!(
            "Goal Transition: {:?} -> {:?} | fear:{:.2} stamina:{:.2} health:{:.2} saturation:{:.2} urgency:{:.2}",
            current_goal, new_goal, fear, stamina, health, saturation, urgency
        );
    }

    new_goal
}

pub fn assign_goals_and_tasks_to_new_creature_entities(
    entities: Query<Entity, (Added<Creature>, Without<Goal>)>,
    mut commands: Commands
) {
    for entity in entities {
        commands.entity(entity).insert((
            Goal::Wander,
            Goal::Wander.tasks_for_goal()[0],
            GoalProgress(0)
        ));
        info!("Initial goal created for: {:?}", entity);
    }
}

pub fn spawn_ai_debug_label(
    mut commands: Commands,
    items: Query<Entity, Added<Goal>>,
) {
    for entity in &items {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(""),
                crate::debug::DebugItem,
                TextFont { font_size: 21.0, ..default() },
                TextColor(Color::Srgba(Srgba { red: 0.7, green: 0.7, blue: 0.7, alpha: 0.8 })),
                Transform::from_xyz(0.0, 1.5, 100.0).with_scale(vec3(0.02, 0.02, 1.0)),
                AIStateLabel,
            ));
        });
    }
}

#[derive(Component)]
pub struct AIStateLabel;

pub fn update_ai_state_labels(
    items: Query<(&Goal, &Task, &GoalProgress, &Children)>,
    mut labels: Query<&mut Text2d, With<AIStateLabel>>,
) {
    for (goal, task, progress, children) in &items {
        for child in children {
            if let Ok(mut text) = labels.get_mut(*child) {
                text.0 = format!("{:?}[{:?}]: \"{:?}\"", goal, progress.0, task);
            }
        }
    }
}

use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::Ordering;

// Components

#[derive(Component)]
pub struct Path {
    pub waypoints: VecDeque<IVec2>,
}

// Internal A* node
#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    cost: u32,
    pos: IVec2,
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost) // min-heap
    }
}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Constants

const MOVE_COST_LAND:  u32 = 10;
const MOVE_COST_WATER: u32 = 40;
const MAX_SEARCH_TILES: usize = 512;

// Helpers

fn world_to_tile(pos: Vec2) -> IVec2 {
    IVec2::new(pos.x.floor() as i32, pos.y.floor() as i32)
}

fn tile_to_world(tile: IVec2) -> Vec2 {
    Vec2::new(tile.x as f32 + 0.5, tile.y as f32 + 0.5)
}

fn tile_cost(
    tile: IVec2,
    world_map: &WorldMap,
    chunks: &Query<&Chunk>,
) -> Option<u32> {
    let chunk_coord = IVec2::new(
        if tile.x >= 0 { tile.x / CHUNK_SIZE as i32 } else { (tile.x + 1 - CHUNK_SIZE as i32) / CHUNK_SIZE as i32 },
        if tile.y >= 0 { tile.y / CHUNK_SIZE as i32 } else { (tile.y + 1 - CHUNK_SIZE as i32) / CHUNK_SIZE as i32 },
    );
    let local = IVec2::new(
        ((tile.x % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32,
        ((tile.y % CHUNK_SIZE as i32) + CHUNK_SIZE as i32) % CHUNK_SIZE as i32,
    );

    let chunk_entity = world_map.chunks.get(&chunk_coord)?;
    let chunk = chunks.get(*chunk_entity).ok()?;

    let foliage = chunk.foliage_data[local.x as usize][local.y as usize];
    if foliage == Foliage::Tree(crate::world::world_generation::TreeType::Oak) {
        return None; // impassable
    }

    let cost = match chunk.tile_data[local.x as usize][local.y as usize] {
        WorldTile::Water => MOVE_COST_WATER,
        _                => MOVE_COST_LAND,
    };
    Some(cost)
}

fn heuristic(a: IVec2, b: IVec2) -> u32 {
    // Chebyshev distance scaled to land cost
    let dx = (a.x - b.x).unsigned_abs();
    let dy = (a.y - b.y).unsigned_abs();
    MOVE_COST_LAND * dx.max(dy)
}

const NEIGHBORS: [IVec2; 8] = [
    IVec2::new( 1,  0), IVec2::new(-1,  0),
    IVec2::new( 0,  1), IVec2::new( 0, -1),
    IVec2::new( 1,  1), IVec2::new(-1,  1),
    IVec2::new( 1, -1), IVec2::new(-1, -1),
];

pub fn astar(
    start: IVec2,
    goal: IVec2,
    world_map: &WorldMap,
    chunks: &Query<&Chunk>,
) -> Option<VecDeque<IVec2>> {
    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<IVec2, IVec2> = HashMap::new();
    let mut g_score: HashMap<IVec2, u32> = HashMap::new();

    g_score.insert(start, 0);
    open.push(Node { cost: 0, pos: start });

    while let Some(Node { pos: current, .. }) = open.pop() {
        if current == goal {
            // Reconstruct path
            let mut path = VecDeque::new();
            let mut cursor = current;
            while cursor != start {
                path.push_front(cursor);
                cursor = came_from[&cursor];
            }
            return Some(path);
        }

        if came_from.len() >= MAX_SEARCH_TILES {
            return None; // give up
        }

        for offset in &NEIGHBORS {
            let neighbor = current + *offset;
            let step_cost = match tile_cost(neighbor, world_map, chunks) {
                Some(c) => c,
                None => continue, // impassable
            };

            // Diagonal costs a bit more
            let move_cost = if offset.x != 0 && offset.y != 0 {
                step_cost * 14 / 10
            } else {
                step_cost
            };

            let tentative_g = g_score[&current] + move_cost;

            if tentative_g < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g);
                let f = tentative_g + heuristic(neighbor, goal);
                open.push(Node { cost: f, pos: neighbor });
            }
        }
    }

    None // no path found
}

// Systems
// Used by GoTo Task
#[derive(Component)]
pub struct Target(Vec2);

pub fn task_go_to(
    mut commands: Commands,
    world_map: Res<WorldMap>,
    chunks: Query<&Chunk>,
    mut entities: Query<(Entity, &Task, &crate::physics::PhysicalTranslation, &Target, Option<&mut Path>, &mut Velocity, &Creature)>,
    creature_data_base: Res<CreatureDatabase>
) {
    for (entity, task, phys_pos, target, maybe_path, mut velocity, creature) in &mut entities {
        if *task != Task::GoTo { continue; } // swap per GoTo variant

        let current_tile = world_to_tile(phys_pos.0);
        let target_tile  = world_to_tile(target.0.trunc());

        // Already there
        if current_tile == target_tile {
            commands.entity(entity)
                .remove::<Path>()
                .remove::<Target>()
                .insert(TaskFinished);
            continue;
        }

        match maybe_path {
            // No path yet — compute one
            None => {
                match astar(current_tile, target_tile, &world_map, &chunks) {
                    Some(path) => { commands.entity(entity).insert(Path { waypoints: path }); }
                    None => {
                        // Unreachable — skip to next task anyway
                        commands.entity(entity).insert(TaskFinished);
                    }
                }
            }

            // Path exists — follow it
            Some(mut path) => {
                // Pop waypoints we've already reached
                while let Some(&next) = path.waypoints.front() {
                    if world_to_tile(phys_pos.0) == next {
                        path.waypoints.pop_front();
                    } else {
                        break;
                    }
                }

                if let Some(&next_waypoint) = path.waypoints.front() {
                    let target_world = tile_to_world(next_waypoint);
                    let direction = (target_world - phys_pos.0).normalize_or_zero();
                    let creature_speed = creature_data_base.entries.iter().find(|x| x.kind == creature.0).unwrap().move_speed;
                    velocity.0 += direction * creature_speed * 0.05; // tune this scalar to taste
                } else {
                    // Exhausted all waypoints
                    commands.entity(entity)
                        .remove::<Path>()
                        .remove::<Target>()
                        .insert(TaskFinished);
                }
            }
        }
    }
}

// Component set by LocatePrey / LocateTarget
#[derive(Component)]
pub struct ChaseTarget(pub Entity);

// How far away before we re-path (in tiles). Tune to taste.
const CHASE_REPATH_DISTANCE: i32 = 3;
// How close before Chase reports success
const CHASE_ARRIVAL_DISTANCE: f32 = 1.0;

pub fn task_chase(
    mut commands: Commands,
    world_map: Res<WorldMap>,
    chunks: Query<&Chunk>,
    creature_data_base: Res<CreatureDatabase>,
    target_transforms: Query<&crate::physics::PhysicalTranslation, With<Creature>>, // fetch target's position
    mut chasers: Query<(
        Entity,
        &Task,
        &ChaseTarget,
        &crate::physics::PhysicalTranslation,
        Option<&mut Path>,
        &mut Velocity,
        &Creature,
    )>,
) {
    for (entity, task, chase_target, phys_pos, maybe_path, mut velocity, creature) in &mut chasers {
        if *task != Task::Chase { continue; }

        // If target entity is gone (dead, despawned), abandon goal
        let Ok(target_transform) = target_transforms.get(chase_target.0) else {
            warn!("DEAD ENTITY");
            commands.entity(entity)
                .remove::<Path>()
                .remove::<ChaseTarget>()
                .insert(GoalFinished);
            continue;
        };

        let target_pos = target_transform.xy();
        let current_tile = world_to_tile(phys_pos.0);
        let target_tile  = world_to_tile(target_pos);

        // Close enough — done chasing
        if phys_pos.0.distance(target_pos) <= CHASE_ARRIVAL_DISTANCE {
            commands.entity(entity)
                .remove::<Path>()
                .insert(TaskFinished);
            continue;
        }

        let needs_repath = match &maybe_path {
            None => true,
            // Re-path when target has drifted far from where we last pathed to
            Some(path) => match path.waypoints.back() {
                None => true,
                Some(&last_waypoint) => {
                    let dx = (last_waypoint.x - target_tile.x).abs();
                    let dy = (last_waypoint.y - target_tile.y).abs();
                    dx > CHASE_REPATH_DISTANCE || dy > CHASE_REPATH_DISTANCE
                }
            },
        };

        if needs_repath {
            match astar(current_tile, target_tile, &world_map, &chunks) {
                Some(new_path) => { commands.entity(entity).insert(Path { waypoints: new_path }); }
                None => {
                    warn!("UNREACHABLE TARGET");
                    // Target unreachable — give up
                    commands.entity(entity)
                        .remove::<Path>()
                        .remove::<ChaseTarget>()
                        .insert(GoalFinished);
                }
            }
            continue; // steer next frame once path is fresh
        }

        // Follow the path
        if let Some(mut path) = maybe_path {
            while let Some(&next) = path.waypoints.front() {
                if world_to_tile(phys_pos.0) == next {
                    path.waypoints.pop_front();
                } else {
                    break;
                }
            }

            if let Some(&next_waypoint) = path.waypoints.front() {
                let target_world = tile_to_world(next_waypoint);
                let direction = (target_world - phys_pos.0).normalize_or_zero();
                let speed = creature_data_base.entries.iter()
                    .find(|x| x.kind == creature.0).unwrap().move_speed;
                velocity.0 += direction * speed * 0.05;
            }
        }
    }
}

pub fn task_locate_random(
    mut commands: Commands,
    entities: Query<(Entity, &Transform, &Task)>,
) {
    for (entity, transform, task) in &entities {
        if *task != Task::LocateRandom { continue; }

        let offset = Vec2::new(
            rand::random::<f32>() * 20. - 10.,
            rand::random::<f32>() * 20. - 10.,
        );

        commands.entity(entity)
            .insert(Target(transform.translation.xy() + offset))
            .insert(TaskFinished);
    }
}

pub fn task_locate_prey(
    mut commands: Commands,
    predators: Query<(Entity, &Task, &Creature, &crate::physics::PhysicalTranslation)>,
    all_creatures: Query<(Entity, &Creature, &crate::physics::PhysicalTranslation)>,
    creature_database: Res<CreatureDatabase>,
) {
    for (entity, task, creature, pos) in &predators {
        if *task != Task::LocatePrey { continue; }

        let Some(data) = creature_database.entries.iter().find(|x| x.kind == creature.0) else {
            warn!("No database entry for predator {:?} — abandoning Hunt", creature.0);
            commands.entity(entity).insert(GoalFinished);
            continue;
        };

        if data.prey.is_empty() {
            // Creature has no prey defined — should probably not be hunting
            warn!("Creature has no prey defined — should probably not be hunting");
            commands.entity(entity).insert(GoalFinished);
            continue;
        }

        let closest_prey = all_creatures
            .iter()
            .filter(|(prey_entity, prey_creature, _)| {
                *prey_entity != entity && data.prey.contains(&prey_creature.0)
            })
            .min_by(|(_, _, a_pos), (_, _, b_pos)| {
                let da = a_pos.0.distance_squared(pos.0);
                let db = b_pos.0.distance_squared(pos.0);
                da.partial_cmp(&db).unwrap_or(Ordering::Equal)
            });

        match closest_prey {
            Some((prey_entity, prey_creature, _)) => {
                info!("{:?} found prey: {:?} ({:?})", creature.0, prey_entity, prey_creature.0);
                commands.entity(entity)
                    .insert(ChaseTarget(prey_entity))
                    .insert(TaskFinished);
            }
            None => {
                // No prey in the world right now — abandon Hunt, re-evaluate
                commands.entity(entity).insert(GoalFinished);
            }
        }
    }
}