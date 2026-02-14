//! # Scene Graph Module
//!
//! Arena-based storage for the node hierarchy.
//!
// TODO: Cache PathMeasure and total length in PathAnimationState.
//!
//! ## Responsibilities
//! - **Node Storage**: `Vec<Option<SceneNode>>` arena with `NodeId` indices.
//! - **Hierarchy**: Parent-child relationships via `children` and `parent`.
//! - **Node Operations**: Add, remove, reparent nodes with cycle prevention.
//!
//! ## Key Types
//! - `SceneGraph`: The arena container.
//! - `SceneNode`: Wraps an `Element` with layout and hierarchy data.
//! - `NodeId`: A `usize` index into the arena (defined in `types.rs`).

use crate::element::Element;
use crate::types::{NodeId, PathAnimationState, Transform};

/// Runtime binding of an audio analysis value to a node property.
///
/// Enables beat-reactive visuals by mapping frequency band energy to node properties.
#[derive(Clone, Debug)]
pub struct AudioBinding {
    /// Index of the audio track in the mixer
    pub track_id: usize,
    /// Frequency band: "bass", "mids", "highs"
    pub band: String,
    /// Property to animate: "scale", "opacity", "x", "y", "rotation"
    pub property: String,
    /// Minimum output value (when energy is 0)
    pub min_value: f32,
    /// Maximum output value (when energy is 1)
    pub max_value: f32,
    /// Smoothing factor (0.0 = instant, 0.9 = heavy smoothing)
    pub smoothing: f32,
    /// Previous smoothed value for temporal smoothing
    pub prev_value: f32,
}

/// A wrapper around an `Element` that adds scene graph relationships and state.
///
/// `SceneNode` encapsulates the specific logic for hierarchy, layout positioning,
/// masking, and temporal state (local time).
#[derive(Clone)]
pub struct SceneNode {
    /// The actual visual element (Box, Text, etc.)
    pub element: Box<dyn Element>,
    /// Indices of child nodes.
    pub children: Vec<NodeId>,
    /// Index of parent node.
    pub parent: Option<NodeId>,
    /// The computed absolute layout rectangle (set by `LayoutEngine`).
    pub layout_rect: skia_safe::Rect,
    /// The local time for the current frame (computed during update pass).
    pub local_time: f64,
    /// The global time when this node was last visited/prepared for update.
    pub last_visit_time: f64,

    // Path Animation
    pub path_animation: Option<PathAnimationState>,
    pub transform: Transform,

    // Masking & Compositing
    pub mask_node: Option<NodeId>,
    pub blend_mode: skia_safe::BlendMode,

    /// Explicit render order z-index (default: 0).
    /// Higher values render on top of lower values.
    /// Sorting is local to the parent's children list.
    pub z_index: i32,

    pub dirty_style: bool,

    /// Audio-reactive bindings for this node
    pub audio_bindings: Vec<AudioBinding>,
}

impl SceneNode {
    /// Creates a new SceneNode wrapping the given Element.
    pub fn new(element: Box<dyn Element>) -> Self {
        Self {
            element,
            children: Vec::new(),
            parent: None,
            layout_rect: skia_safe::Rect::default(),
            local_time: 0.0,
            last_visit_time: -1.0,
            path_animation: None,
            transform: Transform::new(),
            mask_node: None,
            blend_mode: skia_safe::BlendMode::SrcOver,
            z_index: 0,
            dirty_style: true,
            audio_bindings: Vec::new(),
        }
    }
}

/// The Scene Graph data structure.
///
/// Manages the arena of nodes and their relationships.
#[derive(Clone)]
pub struct SceneGraph {
    /// The Arena of all nodes. Using `Option` allows for future removal/recycling.
    pub nodes: Vec<Option<SceneNode>>,
    /// Indices of nodes that have been removed and can be reused.
    pub free_indices: Vec<usize>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
        self.free_indices.clear();
    }

    /// Adds a new element to the scene graph and returns its ID.
    pub fn add_node(&mut self, element: Box<dyn Element>) -> NodeId {
        if let Some(id) = self.free_indices.pop() {
            self.nodes[id] = Some(SceneNode::new(element));
            id
        } else {
            let id = self.nodes.len();
            self.nodes.push(Some(SceneNode::new(element)));
            id
        }
    }

    /// Recursively destroys a node and its children, freeing their indices for reuse.
    pub fn destroy_node(&mut self, id: NodeId) {
        // 1. Check if node exists (and isn't already deleted)
        if id >= self.nodes.len() || self.nodes[id].is_none() {
            return;
        }

        // 2. Collect IDs to process (to avoid holding borrows on self.nodes)
        let (parent_id, children_ids) = {
            let Some(node) = self.nodes[id].as_ref() else {
                return;
            };
            (node.parent, node.children.clone())
        };

        // 3. Detach from Parent
        if let Some(pid) = parent_id {
            self.remove_child(pid, id);
        }

        // 4. Recursively destroy children
        for child_id in children_ids {
            self.destroy_node(child_id);
        }

        // 5. Free the slot
        self.nodes[id] = None;
        self.free_indices.push(id);
    }

    /// Establishes a parent-child relationship between two nodes.
    ///
    /// Invalid relationships (missing nodes, self-parenting, cycles) are ignored.
    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        let _ = self.try_add_child(parent, child);
    }

    /// Attempts to establish a parent-child relationship between two nodes.
    ///
    /// Returns `true` when the relationship is created and `false` when rejected
    /// (missing nodes, self-parenting, or cycle detection).
    pub fn try_add_child(&mut self, parent: NodeId, child: NodeId) -> bool {
        if parent == child {
            return false;
        }

        // Both nodes must exist.
        if self.get_node(parent).is_none() || self.get_node(child).is_none() {
            return false;
        }

        // Prevent hierarchy cycles by checking whether `child` is an ancestor of `parent`.
        let mut current = Some(parent);
        while let Some(node_id) = current {
            if node_id == child {
                return false;
            }
            current = self.get_node(node_id).and_then(|n| n.parent);
        }

        // Detach from previous parent if re-parenting.
        let old_parent = self.get_node(child).and_then(|n| n.parent);
        if let Some(old_parent_id) = old_parent {
            if old_parent_id == parent {
                // Already correctly parented.
                return true;
            }
            self.remove_child(old_parent_id, child);
        }

        if let Some(p_node) = self.nodes.get_mut(parent).and_then(|n| n.as_mut()) {
            if !p_node.children.contains(&child) {
                p_node.children.push(child);
            }
        } else {
            return false;
        }

        if let Some(c_node) = self.nodes.get_mut(child).and_then(|n| n.as_mut()) {
            c_node.parent = Some(parent);
            true
        } else {
            false
        }
    }

    /// Removes a child from a parent node's children list.
    /// Also clears the child's `parent` field when it points to this parent.
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        if let Some(p_node) = self.nodes.get_mut(parent).and_then(|n| n.as_mut()) {
            if let Some(pos) = p_node.children.iter().position(|&x| x == child) {
                p_node.children.remove(pos);
            }
        }
        if let Some(c_node) = self.nodes.get_mut(child).and_then(|n| n.as_mut()) {
            if c_node.parent == Some(parent) {
                c_node.parent = None;
            }
        }
    }

    /// Returns a mutable reference to the SceneNode.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(id).and_then(|n| n.as_mut())
    }

    /// Returns a shared reference to the SceneNode.
    pub fn get_node(&self, id: NodeId) -> Option<&SceneNode> {
        self.nodes.get(id).and_then(|n| n.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::SceneGraph;
    use crate::node::BoxNode;

    #[test]
    fn add_child_rejects_self_parent() {
        let mut scene = SceneGraph::new();
        let id = scene.add_node(Box::new(BoxNode::new()));

        assert!(!scene.try_add_child(id, id));
        assert!(scene.get_node(id).is_some());
    }

    #[test]
    fn add_child_rejects_cycle() {
        let mut scene = SceneGraph::new();
        let a = scene.add_node(Box::new(BoxNode::new()));
        let b = scene.add_node(Box::new(BoxNode::new()));
        let c = scene.add_node(Box::new(BoxNode::new()));

        assert!(scene.try_add_child(a, b));
        assert!(scene.try_add_child(b, c));
        assert!(
            !scene.try_add_child(c, a),
            "cycle creation must be rejected"
        );
    }

    #[test]
    fn reparent_child_detaches_from_old_parent() {
        let mut scene = SceneGraph::new();
        let p1 = scene.add_node(Box::new(BoxNode::new()));
        let p2 = scene.add_node(Box::new(BoxNode::new()));
        let child = scene.add_node(Box::new(BoxNode::new()));

        assert!(scene.try_add_child(p1, child));
        assert!(scene.try_add_child(p2, child));

        let p1_node = scene.get_node(p1).unwrap();
        let p2_node = scene.get_node(p2).unwrap();
        let child_node = scene.get_node(child).unwrap();

        assert!(!p1_node.children.contains(&child));
        assert!(p2_node.children.contains(&child));
        assert_eq!(child_node.parent, Some(p2));
    }
}
