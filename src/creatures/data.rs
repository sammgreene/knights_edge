use std::collections::HashMap;
use bevy::prelude::*;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::Deserialize;

// Raw deserialized form from RON
#[derive(Debug, Deserialize, Clone)]
pub struct CreatureDataRaw {
    pub kind: String,
    pub eats: Vec<String>,
    pub prey: Vec<String>,
    pub move_speed: f32,
    pub sprint_speedup: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub attack_range: f32,
    pub bravery: f32,
    pub endurance: f32,
    pub robustness: f32,
    pub appetite: f32
}

#[derive(Resource, Debug, Deserialize)]
pub struct CreatureDataFile {
    pub creatures: Vec<CreatureDataRaw>,
}

// The actual Bevy asset that gets stored
#[derive(Resource, Asset, TypePath, Debug, Clone)]
pub struct CreatureDatabase {
    pub entries: Vec<CreatureDataRaw>,
}

// Resource holding the handle so it stays loaded
#[derive(Resource, Clone)]
pub struct CreatureDatabaseHandle(pub Handle<CreatureDatabase>);

// AssetLoader
#[derive(Default, Reflect)]
pub struct CreatureDatabaseLoader;

impl AssetLoader for CreatureDatabaseLoader {
    type Asset = CreatureDatabase;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let data: CreatureDataFile = ron::de::from_bytes(&bytes)?;
        Ok(CreatureDatabase { entries: data.creatures })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Resource, Default, Clone)]
pub struct CreatureRelations {
    prey_of: HashMap<String, Vec<String>>,
    predators_of: HashMap<String, Vec<String>>,
}

impl CreatureRelations {
    pub fn build(db: &CreatureDatabase) -> Self {
        let mut prey_of: HashMap<String, Vec<String>> = HashMap::new();
        let mut predators_of: HashMap<String, Vec<String>> = HashMap::new();

        for entry in &db.entries {
            prey_of.insert(entry.kind.clone(), entry.prey.clone());
            // ensure every creature has an entry even if nothing hunts it
            predators_of.entry(entry.kind.clone()).or_default();
        }

        // build reverse map from the forward map
        for (predator, prey_list) in &prey_of {
            for prey in prey_list {
                predators_of
                    .entry(prey.clone())
                    .or_default()
                    .push(predator.clone());
            }
        }

        Self { prey_of, predators_of }
    }

    pub fn prey_of(&self, kind: &str) -> &[String] {
        self.prey_of.get(kind).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn predators_of(&self, kind: &str) -> &[String] {
        self.predators_of.get(kind).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn is_prey(&self, kind: &str) -> bool {
        self.predators_of(kind).len() > 0
    }

    // Is `hunter` a predator of `target`?
    pub fn hunts(&self, hunter: &str, target: &str) -> bool {
        self.prey_of(hunter).iter().any(|p| p == target)
    }
}