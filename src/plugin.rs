use crate::prelude::*;
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

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
