pub mod error;
pub mod plugin;
pub mod prefab;

pub mod prelude {
    pub(crate) use crate::prefab::PrefabPaths;
    pub use crate::{
        error::PrefabPathCouldNotBeResolved,
        plugin::PrefabPlugin,
        prefab::{CloneFromPrefab, Domain, Prefab, PrefabRegistery, ReferencePrefab},
    };
}
