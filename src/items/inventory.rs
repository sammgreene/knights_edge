use bevy::{audio::Volume, prelude::*};
use crate::items;
use std::collections::HashSet;

const PICKUP_DIST: f32 = 1.5;

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<Entity>,
    slot_count: usize
}
impl Inventory {
    fn is_full(&self) -> bool {
        self.items.len() >= self.slot_count
    }
    pub fn new(slot_count: usize) -> Self {
        Self {
            items: vec![],
            slot_count
        }
    }
}

pub fn remove_despawned_items_from_inventorys(
    mut inventories: Query<&mut Inventory>,
    items: Query<(), With<items::ItemStack>>,
) {
    for mut inventory in &mut inventories {
        inventory.items.retain(|e| items.contains(*e));
    }
}

pub fn pickup_nearby_items(
    inventories: Query<(&mut Inventory, &Transform)>,
    items: Query<(Entity, &Transform, &items::ItemStack), With<items::Dropped>>,
    mut visibility: Query<(&mut Visibility, &mut bevy_firefly::prelude::PointLight2d)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    let mut claimed: HashSet<Entity> = HashSet::new();
    for (mut inventory, inventory_pos) in inventories {
        for (item, item_position, item_stack) in &items {
            if inventory.is_full() {
                break;
            }
            if claimed.contains(&item) {
                continue;
            }
            if item_position.translation.distance(inventory_pos.translation) < PICKUP_DIST {
                info!("item picked up: {:?}", item_stack);
                commands.entity(item).insert((
                    AudioPlayer::new(asset_server.load("sounds/item_pickup.wav")),
                    PlaybackSettings::ONCE.with_volume(Volume::Linear(0.2))
                ));
                inventory.items.push(item);
                claimed.insert(item);
                commands.entity(item).remove::<items::Dropped>();
            }
        }
    }
    for item in &claimed {
        if let Ok((mut vis, mut light)) = visibility.get_mut(*item) {
            *vis = Visibility::Hidden;
            light.intensity = 0.0;
        }
    }
}