
mod error;
mod sg;

use crate::error::{AtlasError, Result};
use sg::SceneGraph;

/// A stream of events recorded through several scene graph snapshot from the environment.
/// Each snapshot represents the state of the scene graph at a specific point in time.
pub struct SceneGraphStream {
    /// A list of snapshots in the live scene graph.
    pub snapshots: Vec<SceneGraph>,
}

impl SceneGraphStream {
    /// Create a new empty SceneGraphStream.
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }
}

