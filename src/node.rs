use serde::{Deserialize, Serialize};

use crate::error::{AtlasError, Result};

/// A node in the scene graph representing an object.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Node {
    /// Unique identifier for the node.
    #[serde(rename = "node_id")]
    pub id: usize,

    /// Labels associated with the node.
    label: Vec<String>,

    /// Affordance labels associated with the node, if any.
    label_affordance: Option<Vec<String>>,

    /// Last snapshot index when the node was processed.
    processed_last: usize,

    /// list of features representing the object.
    features: Vec<f32>,

    // TODO: bind colors and Points together
    /// 3D points of the point cloud representing the object.
    pub pcd_points: Vec<[f32; 3]>,

    /// Colors of the points in the point cloud.
    pub pcd_colors: Vec<[f32; 3]>,
}

impl Node {
    /// Create a new empty node with the given id.
    #[cfg(test)]
    pub fn new(id: usize) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    /// Add a point to the node's point cloud.
    pub fn add_point<L: Point, C: Point>(&mut self, loc: L, color: C) {
        self.pcd_points.push(loc.as_array());
        self.pcd_colors.push(color.as_array());
    }

    /// Delete a point from the node's point cloud if it exists.
    /// If the point does not exist, do nothing.
    pub fn del_point<L: Point>(&mut self, loc: &L) -> Result<(f32, f32, f32)> {
        if let Some(pos) = self.pcd_points.iter().position(|p| *p == loc.as_array()) {
            // TODO: ensure order is not important here
            self.pcd_points.swap_remove(pos);
            Ok(self.pcd_colors.swap_remove(pos).into())
        } else {
            Err(AtlasError::PointNotFound)
        }
    }

    /// Update a point in the node's point cloud if it exists.
    /// If the point does not exist, return an error since color information can not be retrieved.
    pub fn update_point<L1: Point, L2: Point>(&mut self, from: L1, to: L2) -> Result<()> {
        match self.del_point(&from) {
            Ok(c) => {
                self.add_point(to, c);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

/// Point trait to abstract over different point representations.
/// Point are required to expose x, y, z methods to access their coordinates.
pub trait Point {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn z(&self) -> f32;
    fn as_array(&self) -> [f32; 3] {
        [self.x(), self.y(), self.z()]
    }
}

impl Point for (f32, f32, f32) {
    #[inline]
    fn x(&self) -> f32 {
        self.0
    }
    #[inline]
    fn y(&self) -> f32 {
        self.1
    }
    #[inline]
    fn z(&self) -> f32 {
        self.2
    }
}

impl Point for [f32; 3] {
    #[inline]
    fn x(&self) -> f32 {
        self[0]
    }
    #[inline]
    fn y(&self) -> f32 {
        self[1]
    }
    #[inline]
    fn z(&self) -> f32 {
        self[2]
    }
    #[inline]
    fn as_array(&self) -> [f32; 3] {
        *self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn point_crud() {
        let mut n = Node::new(0);

        // add points
        n.add_point((1.0, 2.0, 3.0), (0.1, 0.2, 0.3));
        n.add_point([4.0, 5.0, 6.0], [0.4, 0.5, 0.6]);
        assert_eq!(n.pcd_points.len(), 2);
        assert_eq!(n.pcd_colors.len(), 2);
        assert_eq!(n.pcd_points[0], [1.0, 2.0, 3.0]);
        assert_eq!(n.pcd_colors[0], [0.1, 0.2, 0.3]);
        assert_eq!(n.pcd_points[1], [4.0, 5.0, 6.0]);
        assert_eq!(n.pcd_colors[1], [0.4, 0.5, 0.6]);

        // delete points
        assert!(n.del_point(&(1.0, 2.0, 3.0)).is_ok());
        assert_eq!(n.pcd_points.len(), 1);
        assert_eq!(n.pcd_colors.len(), 1);
        assert_eq!(n.pcd_points[0], [4.0, 5.0, 6.0]);

        // delete non-existing point
        assert!(n.del_point(&(1.0, 2.0, 3.0)).is_err());
        assert_eq!(n.pcd_points.len(), 1);
        assert_eq!(n.pcd_colors.len(), 1);
        assert_eq!(n.pcd_points[0], [4.0, 5.0, 6.0]);

        // update points
        assert!(n.update_point((4.0, 5.0, 6.0), (7.0, 8.0, 9.0)).is_ok());
        assert_eq!(n.pcd_points.len(), 1);
        assert_eq!(n.pcd_colors.len(), 1);
        assert_eq!(n.pcd_points[0], [7.0, 8.0, 9.0]);
        assert_eq!(n.pcd_colors[0], [0.4, 0.5, 0.6]);

        // update non-existing points
        assert!(n.update_point((4.0, 5.0, 6.0), (10.0, 11.0, 12.0)).is_err());
        assert_eq!(n.pcd_points.len(), 1);
        assert_eq!(n.pcd_colors.len(), 1);
        assert_eq!(n.pcd_points[0], [7.0, 8.0, 9.0]);
        assert_eq!(n.pcd_colors[0], [0.4, 0.5, 0.6]);
    }
}
