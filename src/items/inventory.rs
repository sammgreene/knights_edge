use bevy::{audio::Volume, prelude::*};
use crate::items::*;
use std::collections::HashSet;

const PICKUP_DIST: f32 = 1.5;

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<Entity>,
    slot_count: usize,
    pub selected_item: Option<usize>
}
impl Inventory {
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.slot_count
    }
    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }
    pub fn selecting_valid_item(&self) -> bool {
        if self.selected_item.is_none() { return false }
        if self.is_empty() { return false }
        if self.selected_item.unwrap() >= self.items.len() { return false }
        true
    }
    pub fn select_valid_item(&mut self) {
        if self.selecting_valid_item() { return }
        if self.is_empty() { self.selected_item = None; return }
        self.selected_item = Some(self.items.len() - 1)
    }
    pub fn new(slot_count: usize) -> Self {
        Self {
            items: vec![],
            slot_count,
            selected_item: None
        }
    }
}

pub fn remove_despawned_items_from_inventorys(
    mut inventories: Query<&mut Inventory>,
    items: Query<(), With<ItemStack>>,
) {
    for mut inventory in &mut inventories {
        inventory.items.retain(|e| items.contains(*e));
    }
}

pub fn inventories_pickup_nearby_items(
    inventories: Query<(Entity, &mut Inventory, &Transform)>,
    items: Query<(Entity, &Transform, &ItemStack, Option<&PickupCooldown>), With<Dropped>>,
    mut visibility: Query<(&mut Visibility, &mut PointLight2d)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut claimed: HashSet<Entity> = HashSet::new();
    for (inventory_entity, mut inventory, inventory_pos) in inventories {
        for (item, item_position, item_stack, cooldown) in items {
            if inventory.is_full() {
                break;
            }
            if claimed.contains(&item) {
                continue;
            }
            if cooldown.is_some() {
                let cooldown = cooldown.unwrap();
                if cooldown.effected_entity == inventory_entity {
                    continue;
                }
            }
            if item_position.translation.distance(inventory_pos.translation) < PICKUP_DIST {
                // item_stack.print_stack();
                commands.entity(item).insert((
                    AudioPlayer::new(asset_server.load("sounds/item_pickup.wav")),
                    PlaybackSettings::ONCE.with_volume(Volume::Linear(0.2))
                ));
                inventory.items.push(item);
                if inventory.selected_item.is_none() { inventory.selected_item = Some(0) }
                claimed.insert(item);
                commands.entity(item).remove::<Dropped>();
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