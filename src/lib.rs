mod error;
mod sg;

pub use sg::{NodeFeature, SceneGraph};

/// A stream of events recorded through several scene graph snapshot from the environment.
/// Each snapshot represents the state of the scene graph at a specific point in time.
pub struct SceneGraphStream {
    /// A list of snapshots in the live scene graph.
    pub snapshots: Vec<SceneGraph>,
}

impl SceneGraphStream {
    /// Returns a reference to the top snapshot in the stream, if any.
    pub fn top(&self) -> Option<&SceneGraph> {
        self.snapshots.last()
    }

    /// Removes and returns the top snapshot in the stream, if any.
    pub fn pop(&mut self) -> Option<SceneGraph> {
        self.snapshots.pop()
    }

    /// Adds a new snapshot to the top of the stream.
    pub fn push(&mut self, snapshot: SceneGraph) {
        self.snapshots.push(snapshot);
    }
}
