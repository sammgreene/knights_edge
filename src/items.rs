use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_firefly::lights::PointLight2d;
use crate::physics::PhysicsObject;
use crate::asset_loading;
use crate::render::RenderLayer;

mod inventory;
mod actions;

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, test_items)
        .add_systems(Update, (
            // set_sprite_for_dropped_items,
            inventory::remove_despawned_items
        ));
    }
}

#[derive(Component, Debug)]
struct ItemStack {
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
        let count = u8::min(count, kind.max());
        Self { kind, count }
    }
}

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

    fn asset_key(&self) -> &str {
        match self {
            Self::BasicSword => "basic_sword",
            Self::Apple => "apple",
        }
    }

    fn sprite(&self, asset_server: &AssetServer) -> Sprite {
        Sprite {
            image: asset_server.load("items/apple.png"),
            custom_size: Some(Vec2::splat(1.0)),
            ..default()
        }
    }

    fn spawn(&self, count: u8, commands: &mut Commands, asset_server: &AssetServer) -> Entity {
        let base = (
            ItemStack::new(self.clone(), count),
            self.sprite(asset_server),
            RenderLayer::FoliageBack,
            Anchor::BOTTOM_LEFT,
            PhysicsObject,
            Dropped,
            PointLight2d {
                color: Color::srgb(1.0, 0.6, 1.0),
                range: 0.5,
                intensity: 0.2,
                offset: Vec3 { x: 0.5, y: 0.5, z: 0.0 },
                ..default()
            },
        );
        match self {
            Self::BasicSword => commands.spawn((
                base,
                actions::MeleeWeapon { damage: 10.0, range: 1.5, animation: "sword_swing" },
            )),
            Self::Apple => commands.spawn((
                base,
                actions::Consumable { health: 5.0, hunger: 10.0 },
            )),
        }.id()
    }
}

#[derive(Component)]
struct Dropped;

fn set_sprite_for_dropped_items(
    dropped_item_sprites: Query<(&mut Sprite, &ItemStack), Added<Dropped>>,
    asset_server: Res<AssetServer>
) {
    for (mut sprite, item_stack) in dropped_item_sprites {
        info!("item stack dropped: {:?}", item_stack);
        *sprite = item_stack.kind.sprite(&asset_server)
    }
}

fn test_items(mut commands: Commands, asset_server: Res<AssetServer>) {
    ItemType::Apple.spawn(2, &mut commands, &asset_server);
}