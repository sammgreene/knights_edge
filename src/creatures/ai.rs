use bevy::prelude::*;
use crate::world::world_generation::*;
use crate::{creatures::*, physics::Velocity};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub enum Goal {
    Wander,   // locate random location, go to location
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
            Goal::Feed =>       &[Task::LocateFood, Task::GoTo, Task::CollectFood, Task::Eat],
            Goal::Hunt =>       &[Task::LocatePrey, Task::GoTo],
            Goal::Sleep =>      &[Task::LocateNest, Task::GoTo, Task::Sleep],
            Goal::Nest =>       &[Task::LocateProspectiveNest, Task::GoTo, Task::RememberNest],
            Goal::Stash =>      &[Task::LocateNest, Task::GoTo, Task::StashFood],
            Goal::Retrieve =>   &[Task::LocateNest, Task::GoTo, Task::RetrieveFood, Task::Eat],
            Goal::Mate =>       &[Task::LocateMate, Task::GoTo, Task::Mate],
            Goal::Follow =>     &[Task::LocateTarget, Task::Follow],
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

    GoToFood,
    GoToPrey,
    GoToNest,
    GoToProspectiveNest,
    GoToMate,
    // GoToTarget,
    GoToRandom,
    GoTo,

    Eat,
    Sleep,
    CollectFood,
    StashFood,
    RetrieveFood,
    Mate,

    Follow,

    Rush,
    Strike,
    Retreat,
    MaintainDistance
}

// Systems
#[derive(Component)]
pub struct TaskFinished;
#[derive(Component)]
pub struct GoalFinished;


pub fn transition_tasks(
    creatures: Query<(Entity, &mut Task, &Goal, &mut GoalProgress), (With<TaskFinished>, Without<GoalFinished>)>,
    mut commands: Commands
) {
    for (entity, mut task, goal, mut progress) in creatures {
        progress.0 += 1;
        let tasks = goal.tasks_for_goal();
        if progress.0 >= tasks.len() { // out of tasks in goal
            commands.entity(entity).insert(GoalFinished);
            commands.entity(entity).remove::<TaskFinished>();
            continue;
        } else {
            *task = tasks[progress.0];
            commands.entity(entity).remove::<TaskFinished>();
        }
    }
}

pub fn transition_goals(
    creatures: Query<(Entity, &Creature, &CreatureState, &CreatureMemory, &mut Task, &mut Goal, &mut GoalProgress, Option<&GoalFinished>)>,
    creatures_data: Res<CreatureDatabase>,
    creature_relations: Res<CreatureRelations>,
    mut commands: Commands
) {
    // info!("transitioning {:?} creatures", creatures.count());
    for (entity, creature, creature_state, creature_memory,  mut task, mut goal, mut progress, goal_finished) in creatures {
        let creature_data = creatures_data.entries.iter().find(|x| x.kind == creature.0).unwrap();

        let new_goal = get_new_goal(&goal, creature, creature_state, creature_memory, &creature_data, &creature_relations);
        if *goal != new_goal || goal_finished.is_some() {
            *goal = new_goal;
            progress.0 = 0; // start at first task in goal
            *task = new_goal.tasks_for_goal()[progress.0];
            commands.entity(entity).remove::<GoalFinished>();
        }
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

    let scores: &[(Goal, f32)] = &[
        (Goal::Evade,  (fear) * (1.0 / creature_data.bravery)),
        (Goal::Sleep,  (1.0 - stamina)    * (1.0 / creature_data.endurance)),
        (Goal::Nest,   (1.0 - health)     * (1.0 / creature_data.robustness)),
        (Goal::Wander, 0.1),
        (
            if creature_relations.is_prey(&creature.0) { Goal::Feed } else { Goal::Hunt },
            (1.0 - saturation) * (1.0 / creature_data.appetite),
        ),
    ];

    let new_goal = scores
        .iter()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .map(|(goal, _)| *goal)
        .unwrap_or(Goal::Wander);

    if new_goal != *current_goal {
        info!(
            "Goal Transition: {:?} -> {:?} | fear:{:.2} stamina:{:.2} health:{:.2} saturation:{:.2}",
            current_goal, new_goal, fear, stamina, health, saturation
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
                TextColor(Color::WHITE),
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

// Tasks Atomics

// Used by GoTo Task
#[derive(Component)]
pub struct Target(Vec2);

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

        info!("Locating Random");

        commands.entity(entity)
            .insert(Target(transform.translation.xy() + offset))
            .insert(TaskFinished);
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