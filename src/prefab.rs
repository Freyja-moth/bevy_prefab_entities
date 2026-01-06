use crate::error::PrefabPathCouldNotBeResolved;
use bevy_ecs::{
    entity::EntityCloner, lifecycle::HookContext, prelude::*, relationship::Relationship,
    system::SystemParam, world::DeferredWorld,
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component)]
pub struct Prefab;

#[derive(Component)]
pub struct Domain;

#[derive(SystemParam)]
pub(crate) struct PrefabPaths<'w, 's> {
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
pub struct PrefabRegistery(pub(crate) HashMap<String, Entity>);
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
