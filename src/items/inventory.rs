use bevy::prelude::*;
use crate::items;

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<Entity>,
    slot_count: u8
}

pub fn remove_despawned_items(
    mut inventories: Query<&mut Inventory>,
    items: Query<(), With<items::ItemStack>>,
) {
    for mut inventory in &mut inventories {
        inventory.items.retain(|e| items.contains(*e));
    }
}