//! # Scripting Types
//!
//! Handle types for Rhai scripting integration.
//!
//! ## Responsibilities
//! - **MovieHandle**: Wrapper around `Director` for script access
//! - **SceneHandle**: Reference to a timeline scene
//! - **NodeHandle**: Reference to a scene graph node
//! - **AudioTrackHandle**: Reference to an audio track
//! - **Handle Validation**: Shared helpers for lock/stale-handle safety

use crate::director::Director;
use crate::types::NodeId;
use rhai::EvalAltResult;
use std::sync::{Arc, Mutex, MutexGuard};

/// Wrapper around `Director` for Rhai scripting.
#[derive(Clone)]
pub struct MovieHandle {
    pub director: Arc<Mutex<Director>>,
}

/// Handle to a specific Scene (or time segment) in the movie.
#[derive(Clone)]
pub struct SceneHandle {
    pub director: Arc<Mutex<Director>>,
    pub root_id: NodeId,
    pub start_time: f64,
    pub duration: f64,
    pub audio_tracks: Vec<usize>,
}

/// Handle to a specific Node in the scene graph.
#[derive(Clone)]
pub struct NodeHandle {
    pub director: Arc<Mutex<Director>>,
    pub id: NodeId,
}

/// Handle to an audio track.
#[derive(Clone)]
pub struct AudioTrackHandle {
    pub director: Arc<Mutex<Director>>,
    pub id: usize,
}

impl MovieHandle {
    pub fn lock_director(&self) -> Result<MutexGuard<'_, Director>, Box<EvalAltResult>> {
        self.director
            .lock()
            .map_err(|_| "Director lock poisoned".into())
    }
}

impl SceneHandle {
    pub fn lock_director(&self) -> Result<MutexGuard<'_, Director>, Box<EvalAltResult>> {
        self.director
            .lock()
            .map_err(|_| "Director lock poisoned".into())
    }
}

impl AudioTrackHandle {
    pub fn lock_director(&self) -> Result<MutexGuard<'_, Director>, Box<EvalAltResult>> {
        self.director
            .lock()
            .map_err(|_| "Director lock poisoned".into())
    }
}

impl NodeHandle {
    pub fn lock_director(&self) -> Result<MutexGuard<'_, Director>, Box<EvalAltResult>> {
        self.director
            .lock()
            .map_err(|_| "Director lock poisoned".into())
    }

    pub fn ensure_alive(&self, director: &Director) -> Result<(), Box<EvalAltResult>> {
        if director.scene.get_node(self.id).is_some() {
            Ok(())
        } else {
            Err(format!("NodeHandle {} is stale (node was destroyed)", self.id).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NodeHandle;
    use crate::node::BoxNode;
    use crate::video_wrapper::RenderMode;
    use crate::{DefaultAssetLoader, Director};
    use std::sync::{Arc, Mutex};

    #[test]
    fn ensure_alive_detects_destroyed_nodes() {
        let director = Arc::new(Mutex::new(Director::new(
            320,
            240,
            30,
            Arc::new(DefaultAssetLoader),
            RenderMode::Preview,
            None,
        )));

        let node_id = {
            let mut d = director.lock().unwrap();
            d.scene.add_node(Box::new(BoxNode::new()))
        };

        let handle = NodeHandle {
            director: director.clone(),
            id: node_id,
        };

        {
            let d = director.lock().unwrap();
            assert!(handle.ensure_alive(&d).is_ok());
        }

        {
            let mut d = director.lock().unwrap();
            d.scene.destroy_node(node_id);
        }

        let d = director.lock().unwrap();
        assert!(handle.ensure_alive(&d).is_err());
    }
}
