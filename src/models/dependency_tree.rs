//! Dependency tree structures for hierarchical dependency representation

use super::application::Application;
use super::classification::Classification;
use serde::{Deserialize, Serialize};

/// A node in the dependency tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    /// Package name
    pub name: String,

    /// Package version
    pub version: String,

    /// Classification (Has, Should, or Can)
    pub classification: Classification,

    /// Direct dependencies (children in tree)
    pub dependencies: Vec<DependencyNode>,

    /// Whether this is a direct dependency of the application
    pub is_direct: bool,
}

impl DependencyNode {
    /// Create a new DependencyNode
    pub fn new(
        name: String,
        version: String,
        classification: Classification,
        is_direct: bool,
    ) -> Self {
        Self {
            name,
            version,
            classification,
            dependencies: Vec::new(),
            is_direct,
        }
    }

    /// Add a child dependency
    pub fn add_dependency(&mut self, dependency: DependencyNode) {
        self.dependencies.push(dependency);
    }

    /// Get all child dependencies
    pub fn get_dependencies(&self) -> &[DependencyNode] {
        &self.dependencies
    }

    /// Count total dependencies (including transitive)
    pub fn count_total_dependencies(&self) -> usize {
        let mut count = self.dependencies.len();
        for dep in &self.dependencies {
            count += dep.count_total_dependencies();
        }
        count
    }

    /// Find a dependency by name (recursive search)
    pub fn find_dependency(&self, name: &str) -> Option<&DependencyNode> {
        if self.name == name {
            return Some(self);
        }

        for dep in &self.dependencies {
            if let Some(found) = dep.find_dependency(name) {
                return Some(found);
            }
        }

        None
    }

    /// Get the depth of this node in the tree
    pub fn max_depth(&self) -> usize {
        if self.dependencies.is_empty() {
            0
        } else {
            1 + self
                .dependencies
                .iter()
                .map(|d| d.max_depth())
                .max()
                .unwrap_or(0)
        }
    }
}

/// A complete dependency tree for an application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTree {
    /// Root application
    pub application: Application,

    /// Top-level dependency nodes (direct dependencies of the application)
    pub roots: Vec<DependencyNode>,
}

impl DependencyTree {
    /// Create a new DependencyTree
    pub fn new(application: Application) -> Self {
        Self {
            application,
            roots: Vec::new(),
        }
    }

    /// Add a root-level dependency
    pub fn add_root(&mut self, node: DependencyNode) {
        self.roots.push(node);
    }

    /// Get all root nodes
    pub fn get_roots(&self) -> &[DependencyNode] {
        &self.roots
    }

    /// Count total dependencies in the tree
    pub fn count_total_dependencies(&self) -> usize {
        let mut count = self.roots.len();
        for root in &self.roots {
            count += root.count_total_dependencies();
        }
        count
    }

    /// Find a dependency by name anywhere in the tree
    pub fn find_dependency(&self, name: &str) -> Option<&DependencyNode> {
        for root in &self.roots {
            if let Some(found) = root.find_dependency(name) {
                return Some(found);
            }
        }
        None
    }

    /// Get the maximum depth of the tree
    pub fn max_depth(&self) -> usize {
        self.roots.iter().map(|r| r.max_depth()).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dependency::Ecosystem;
    use std::path::PathBuf;

    #[test]
    fn test_new_dependency_node() {
        let node = DependencyNode::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Classification::Has,
            true,
        );

        assert_eq!(node.name, "react");
        assert_eq!(node.version, "18.2.0");
        assert_eq!(node.classification, Classification::Has);
        assert!(node.is_direct);
        assert_eq!(node.dependencies.len(), 0);
    }

    #[test]
    fn test_add_dependency() {
        let mut parent = DependencyNode::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Classification::Has,
            true,
        );

        let child = DependencyNode::new(
            "loose-envify".to_string(),
            "1.4.0".to_string(),
            Classification::Has,
            false,
        );

        parent.add_dependency(child);
        assert_eq!(parent.dependencies.len(), 1);
        assert_eq!(parent.dependencies[0].name, "loose-envify");
    }

    #[test]
    fn test_count_total_dependencies() {
        let mut parent = DependencyNode::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Classification::Has,
            true,
        );

        let mut child1 = DependencyNode::new(
            "loose-envify".to_string(),
            "1.4.0".to_string(),
            Classification::Has,
            false,
        );

        let grandchild = DependencyNode::new(
            "js-tokens".to_string(),
            "4.0.0".to_string(),
            Classification::Has,
            false,
        );

        child1.add_dependency(grandchild);
        parent.add_dependency(child1);

        // parent has 1 child + 1 grandchild = 2 total
        assert_eq!(parent.count_total_dependencies(), 2);
    }

    #[test]
    fn test_find_dependency() {
        let mut parent = DependencyNode::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Classification::Has,
            true,
        );

        let child = DependencyNode::new(
            "loose-envify".to_string(),
            "1.4.0".to_string(),
            Classification::Has,
            false,
        );

        parent.add_dependency(child);

        assert!(parent.find_dependency("react").is_some());
        assert!(parent.find_dependency("loose-envify").is_some());
        assert!(parent.find_dependency("nonexistent").is_none());
    }

    #[test]
    fn test_max_depth() {
        let mut parent = DependencyNode::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Classification::Has,
            true,
        );

        let mut child = DependencyNode::new(
            "loose-envify".to_string(),
            "1.4.0".to_string(),
            Classification::Has,
            false,
        );

        let grandchild = DependencyNode::new(
            "js-tokens".to_string(),
            "4.0.0".to_string(),
            Classification::Has,
            false,
        );

        child.add_dependency(grandchild);
        parent.add_dependency(child);

        assert_eq!(parent.max_depth(), 2);
    }

    #[test]
    fn test_new_dependency_tree() {
        let app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        let tree = DependencyTree::new(app);
        assert_eq!(tree.application.name, "myapp");
        assert_eq!(tree.roots.len(), 0);
    }

    #[test]
    fn test_add_root() {
        let app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        let mut tree = DependencyTree::new(app);

        let node = DependencyNode::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Classification::Has,
            true,
        );

        tree.add_root(node);
        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].name, "react");
    }

    #[test]
    fn test_tree_count_total_dependencies() {
        let app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        let mut tree = DependencyTree::new(app);

        let mut root1 = DependencyNode::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Classification::Has,
            true,
        );

        let child = DependencyNode::new(
            "loose-envify".to_string(),
            "1.4.0".to_string(),
            Classification::Has,
            false,
        );

        root1.add_dependency(child);
        tree.add_root(root1);

        let root2 = DependencyNode::new(
            "lodash".to_string(),
            "4.17.21".to_string(),
            Classification::Has,
            true,
        );

        tree.add_root(root2);

        // 2 roots + 1 child = 3 total
        assert_eq!(tree.count_total_dependencies(), 3);
    }

    #[test]
    fn test_tree_find_dependency() {
        let app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        let mut tree = DependencyTree::new(app);

        let mut root = DependencyNode::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Classification::Has,
            true,
        );

        let child = DependencyNode::new(
            "loose-envify".to_string(),
            "1.4.0".to_string(),
            Classification::Has,
            false,
        );

        root.add_dependency(child);
        tree.add_root(root);

        assert!(tree.find_dependency("react").is_some());
        assert!(tree.find_dependency("loose-envify").is_some());
        assert!(tree.find_dependency("nonexistent").is_none());
    }

    #[test]
    fn test_tree_max_depth() {
        let app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        let mut tree = DependencyTree::new(app);

        let mut root = DependencyNode::new(
            "react".to_string(),
            "18.2.0".to_string(),
            Classification::Has,
            true,
        );

        let mut child = DependencyNode::new(
            "loose-envify".to_string(),
            "1.4.0".to_string(),
            Classification::Has,
            false,
        );

        let grandchild = DependencyNode::new(
            "js-tokens".to_string(),
            "4.0.0".to_string(),
            Classification::Has,
            false,
        );

        child.add_dependency(grandchild);
        root.add_dependency(child);
        tree.add_root(root);

        assert_eq!(tree.max_depth(), 2);
    }
}
