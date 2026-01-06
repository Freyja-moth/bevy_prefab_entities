use thiserror::Error;

#[derive(Error, Debug)]
#[error("Prefab {0} could not be resolved")]
pub struct PrefabPathCouldNotBeResolved(pub String);
