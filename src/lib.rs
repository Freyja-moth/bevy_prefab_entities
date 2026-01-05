use std::marker::PhantomData;

use bevy::{
    ecs::{
        entity::EntityCloner, lifecycle::HookContext, relationship::Relationship,
        system::SystemParam, world::DeferredWorld,
    },
    platform::collections::HashMap,
    prelude::*,
};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Prefab {0} could not be resolved")]
pub struct PrefabPathCouldNotBeResolved(String);

#[derive(Component)]
pub struct Prefab;

#[derive(Component)]
pub struct Domain;

#[derive(SystemParam)]
struct PrefabPaths<'w, 's> {
    names: Query<'w, 's, &'static Name, (Allow<Prefab>, Allow<Domain>)>,
    parents: Query<'w, 's, &'static ChildOf, (Allow<Prefab>, Allow<Domain>)>,
    domain: Query<'w, 's, (), With<Domain>>,
}
impl<'w, 's> PrefabPaths<'w, 's> {
    pub fn get_path(&self, entity: Entity) -> Result<String, BevyError> {
        let name = self.names.get(entity)?;

        let mut domains: Vec<&str> = self
            .parents
            .iter_ancestors(entity)
            .map_while(|entity| {
                self.domain
                    .contains(entity)
                    .then_some(self.names.get(entity).ok())
                    .flatten()
            })
            .map(|name| name.as_str())
            .collect();

        domains.reverse();
        domains.push(name);

        let path = domains.join("/");

        Ok(path)
    }
}

#[derive(Resource, Default, Debug)]
pub struct PrefabRegistery(HashMap<String, Entity>);
impl PrefabRegistery {
    pub fn get(&self, path: impl AsRef<str>) -> Option<Entity> {
        self.0.get(path.as_ref()).copied()
    }
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
pub struct CloneFromPrefab(pub String);
impl CloneFromPrefab {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().queue(move |world: &mut World| {
            let CloneFromPrefab(path) = world.entity_mut(ctx.entity).take::<Self>().unwrap();

            let registery = world.resource::<PrefabRegistery>();

            let prefab = registery
                .get(&path)
                .ok_or(PrefabPathCouldNotBeResolved(path))?;

            EntityCloner::build_opt_out(world)
                .finish()
                .clone_entity(world, prefab, ctx.entity);

            Ok::<(), BevyError>(())
        });
    }
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
pub struct ReferencePrefab<R: Relationship>(String, PhantomData<R>);
impl<R: Relationship> ReferencePrefab<R> {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into(), PhantomData)
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .queue(|mut entity: EntityWorldMut| {
                let Self(path, _) = entity.take::<Self>().unwrap();

                let registery = entity.resource::<PrefabRegistery>();

                let prefab = registery
                    .get(&path)
                    .ok_or(PrefabPathCouldNotBeResolved(path))?;

                entity.insert(R::from(prefab));

                Ok::<(), BevyError>(())
            });
    }
}

pub struct PrefabPlugin;
impl Plugin for PrefabPlugin {
    fn build(&self, app: &mut App) {
        app.register_disabling_component::<Prefab>();
        app.register_disabling_component::<Domain>();

        app.init_resource::<PrefabRegistery>()
            .add_observer(insert_prefab)
            .add_observer(remove_prefab)
            .add_observer(remove_domain);
    }
}

fn insert_prefab(
    prefab: On<Insert, Prefab>,
    prefab_paths: PrefabPaths,
    mut prefab_registery: ResMut<PrefabRegistery>,
) -> Result<(), BevyError> {
    let path = prefab_paths.get_path(prefab.entity)?;

    prefab_registery.0.insert(path, prefab.entity);

    Ok(())
}

fn remove_prefab(prefab: On<Remove, Prefab>, mut prefab_registery: ResMut<PrefabRegistery>) {
    prefab_registery
        .0
        .retain(|_, entity| *entity != prefab.entity);
}

fn remove_domain(
    domain: On<Remove, Domain>,
    prefab_paths: PrefabPaths,
    mut prefab_registery: ResMut<PrefabRegistery>,
) -> Result<(), BevyError> {
    let domain_path = prefab_paths.get_path(domain.entity)?;

    prefab_registery
        .0
        .retain(|path, _| !path.starts_with(&domain_path));

    Ok(())
}
