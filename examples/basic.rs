use bevy::prelude::*;
use prefab::{CloneFromPrefab, Domain, Prefab, PrefabPlugin, ReferencePrefab};

#[derive(Component, Reflect, Default, Debug)]
#[relationship_target(relationship = PrefabRef)]
pub struct PrefabOf(Vec<Entity>);

#[derive(Component, Reflect, Debug)]
#[relationship(relationship_target = PrefabOf)]
pub struct PrefabRef(pub Entity);

#[derive(Component, Reflect, Clone, Default, Debug)]
pub struct Player;

#[derive(Component, Reflect, Default, Debug)]
#[relationship_target(relationship = ItemOf)]
pub struct Inventory(Vec<Entity>);

#[derive(Component, Reflect, Debug)]
#[relationship(relationship_target = Inventory)]
pub struct ItemOf(pub Entity);

#[derive(Component, Reflect, Clone, Default, Debug)]
pub struct TwoHanded;

#[derive(Component, Reflect, Clone, Default, Debug)]
pub struct OneHanded;

#[derive(Component, Reflect, Clone, Default, Debug)]
pub struct MeleeWeapon;

#[derive(Component, Reflect, Clone, Default, Debug)]
pub struct RangedWeapon;

#[derive(Component, Reflect, Clone, Default, Debug)]
pub struct Damage(u8);

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, PrefabPlugin))
        .add_systems(Startup, (spawn_weapon_prefabs, spawn_scene).chain())
        .run()
}

fn spawn_weapon_prefabs(mut commands: Commands) {
    commands.spawn((
        Name::new("Weapons"),
        Domain,
        children![
            (
                Name::new("Melee"),
                Domain,
                children![
                    (
                        Name::new("Sword"),
                        Prefab,
                        MeleeWeapon,
                        OneHanded,
                        Damage(10)
                    ),
                    (
                        Name::new("Spear"),
                        Prefab,
                        MeleeWeapon,
                        TwoHanded,
                        Damage(12)
                    )
                ]
            ),
            (
                Name::new("Ranged"),
                Domain,
                children![(
                    Prefab,
                    Name::new("Bow"),
                    RangedWeapon,
                    TwoHanded,
                    Damage(15)
                )]
            )
        ],
    ));
}

fn spawn_scene(mut commands: Commands) {
    commands.spawn((
        Player,
        related!(Inventory[
            CloneFromPrefab::new("Weapons/Melee/Spear"),
        ]),
    ));

    commands.spawn((
        Name::new("Treasure Chest"),
        ReferencePrefab::<PrefabRef>::new("Weapons/Ranged/Bow"),
    ));
}
