use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_firefly::lights::PointLight2d;
use crate::physics::PhysicsObject;
use crate::render::RenderLayer;

pub mod inventory;
mod actions;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, test_items)
        .add_systems(Update, (
            // set_sprite_for_dropped_items,
            inventory::remove_despawned_items_from_inventorys,
            inventory::pickup_nearby_items,
            animate_bobbing,
        ));
    }
}

#[derive(Component, Debug)]
pub struct ItemStack {
    kind: ItemType,
    count: u8
}
impl ItemStack {
    fn new(kind: ItemType, count: u8) -> Self {
        debug_assert!(
            count <= kind.max(),
            "Tried to spawn {} of {:?}, max is {}",
            count, kind, kind.max()
        );
        // let count = u8::min(count, kind.max());
        Self { 
            kind,
            count 
        }
    }
}

#[derive(Component)]
pub struct ItemShadow;

#[derive(Debug, Clone, Copy)]
enum ItemType {
    BasicSword,
    Apple
}
impl ItemType {
    fn max(&self) -> u8 {
        match self {
            Self::BasicSword => 1,
            Self::Apple => 128,
        }
    }

    fn asset_key(&self) -> &'static str {
        match self {
            Self::BasicSword => "items/basic_sword.png",
            Self::Apple => "items/apple.png",
        }
    }

    fn sprite(&self, asset_server: &AssetServer) -> (Sprite, crate::render::SortOffset) {
        (
            Sprite {
            image: asset_server.load(self.asset_key()),
            custom_size: Some(Vec2::splat(1.0)),
            ..default()
            },
            crate::render::SortOffset(0.1)
        )
    }

    fn spawn(&self, count: u8, translation: Vec3, commands: &mut Commands, asset_server: &AssetServer) -> Entity {
        let base = (
            ItemStack::new(self.clone(), count),
            Dropped,

            self.sprite(asset_server),
            RenderLayer::Foliage,
            Anchor::BOTTOM_CENTER,

            PhysicsObject,
            Transform::from_translation(translation),

            Visibility::Visible,
            PointLight2d {
                color: Color::srgb(1.0, 0.6, 1.0),
                range: 0.6,
                intensity: 0.2,
                offset: vec3(0.0,0.3,0.0),
                ..default()
            },
        );
        let item_entity = match self {
            Self::BasicSword => commands.spawn((
                base,
                actions::MeleeWeapon { damage: 10.0, range: 1.5, animation: "sword_swing" },
            )),
            Self::Apple => commands.spawn((
                base,
                actions::Consumable { health: 5.0, hunger: 10.0 },
            )),
        }.id();

        // Spawn shadow as child
        commands.entity(item_entity).with_children(|parent| {
            parent.spawn((
                ItemShadow,
                Sprite {
                    image: asset_server.load("items/item_shadow.png"),
                    custom_size: Some(Vec2::new(0.7, 0.7)),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.4),
                    ..default()
                },
                Anchor::BOTTOM_CENTER,
                // Offset downward to sit at the item's feet, negative z so it renders behind
                Transform::from_xyz(0.0, -0.2, -0.001),
            ));
        });

        item_entity
    }
}

#[derive(Component)]
pub struct Dropped;

const BOB_SPEED: f32 = 2.0;
const BOB_AMPLITUDE: f32 = 0.15;
const SQUASH_AMPLITUDE: f32 = 0.05;

fn animate_bobbing(
    mut dropped_items: Query<(&mut Transform, &mut Sprite, &Children), With<Dropped>>,
    mut shadows: Query<&mut Transform, (With<ItemShadow>, Without<Dropped>)>,
    time: Res<Time>,
) {
    for (mut transform, mut sprite, children) in &mut dropped_items {
        let t = (time.elapsed_secs() * BOB_SPEED).sin().powf(2.0);
        let bob = t * BOB_AMPLITUDE;
        transform.translation.y += bob;
        sprite.custom_size = Some(Vec2::new(
            0.8 - t.abs() * SQUASH_AMPLITUDE,
            0.8 + t.abs() * SQUASH_AMPLITUDE,
        ));

        // Push shadow down to cancel out the bob
        for child in children {
            if let Ok(mut shadow_tf) = shadows.get_mut(*child) {
                shadow_tf.translation.y = -0.2 - bob;
            }
        }
    }
}

fn test_items(mut commands: Commands, asset_server: Res<AssetServer>) {
    ItemType::Apple.spawn(2, vec3(-10.,-3.5,0.0), &mut commands, &asset_server);
    ItemType::BasicSword.spawn(2, vec3(10.,-3.5,0.0), &mut commands, &asset_server);
}