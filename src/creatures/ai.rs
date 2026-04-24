use bevy::prelude::*;
use crate::creatures::*;

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
    
    // creates, together with evade, the attacking systems
    Attack,   // rush, attack
    // evade goal is used in tandem with attack during cooldowns
    Evade,    // retriet, maintain distance
}
impl Goal {
    pub fn tasks_for_goal(self) -> &'static [Task] {
        match self {
            Goal::Wander =>     &[Task::LocateRandom, Task::GoToRandom],
            Goal::Feed =>       &[Task::LocateFood, Task::GoToFood, Task::Eat],
            Goal::Hunt =>       &[Task::LocatePrey, Task::GoToPrey],
            Goal::Sleep =>      &[Task::GoToNest, Task::Sleep],
            Goal::Nest =>       &[Task::LocateProspectiveNest, Task::GoToProspectiveNest, Task::RememberNest],
            Goal::Stash =>      &[Task::GoToNest, Task::StashFood],
            Goal::Retrieve =>   &[Task::GoToNest, Task::RetrieveFood, Task::Eat],
            Goal::Mate =>       &[Task::LocateMate, Task::GoToMate, Task::Mate],
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

    RememberNest, // maybe not

    GoToFood,
    GoToPrey,
    GoToNest,
    GoToProspectiveNest,
    GoToMate,
    GoToTarget,
    GoToRandom,

    Eat,
    Sleep,
    CollectFood,
    StashFood,
    RetrieveFood,
    Mate,
    Share,

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
    creatures: Query<(Entity, &mut Task, &Goal, &mut GoalProgress), (Added<TaskFinished>, Without<GoalFinished>)>,
    mut commands: Commands
) {
    for (entity, mut task, goal, mut progress) in creatures {
        progress.0 += 1;

        let next_task = goal.clone().tasks_for_goal()[progress.0];

        if *task != next_task {
            *task = next_task;
            commands.entity(entity).remove::<TaskFinished>();
        } else {
            commands.entity(entity)
                .remove::<TaskFinished>()
                .insert(GoalFinished);
        }
    }
}

pub fn transition_goals(
    creatures: Query<(Entity, &Creature, &CreatureState, &CreatureMemory, &mut Task, &mut Goal, &mut GoalProgress), Added<GoalFinished>>,
    creatures_data: Res<CreatureDatabase>
) {
    for (entity, creature, creature_state, creature_memory,  mut task, mut goal, mut progress) in creatures {
        let creature_data = creatures_data.entries.iter().find(|x| x.kind == creature.0).unwrap();

        let new_goal = get_new_goal(&goal, creature, creature_state, creature_memory, creature_data.clone());
        if *goal != new_goal {
            info!("Transistioning Goals: id[{:?}][{:?} -> {:?}]", entity, goal, new_goal);
            *goal = new_goal;
            progress.0 = 0; // start at first task in goal
        }

        let new_task = goal.clone().tasks_for_goal()[progress.0];

        if *task != new_task {
            *task = new_task;
        } else { // if we cant find a first task for a new goal then throw error because its a bad goal: transition to Wander goal as default
            warn!("Bad Goal: missing first task");
            *goal = Goal::Wander;
        }
    }
}

fn get_new_goal(current_goal: &Goal, creature: &Creature, creature_state: &CreatureState, creature_memory: &CreatureMemory, creature_data: CreatureDataRaw) -> Goal {
    let fear = creature_state.fear.fraction(); // bravery
    let stamina = creature_state.stamina.fraction(); // endurance
    let health = creature_state.health.fraction(); // weakness
    let saturation = creature_state.saturation.fraction(); // appetite
    
    let fear_score = fear / creature_data.bravery * 10.;
    let tired_score = stamina / creature_data.endurance * 10.;
    let health_score = health / creature_data.weakness * 10.;
    let hunger_score = saturation / creature_data.appetite * 10.;

    let mut scores = [fear_score, tired_score, health_score, hunger_score];
    scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
    scores.reverse();

    if scores[0] == fear_score {
        return Goal::Evade;
    } else if scores[0] == tired_score {
        return Goal::Sleep;
    } else if scores[0] == health_score {
        return Goal::Nest;
    } else if scores[0] == hunger_score {
        // if predator
        return Goal::Hunt;
        // if prey
        return Goal::Feed;
    }

    Goal::Wander
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