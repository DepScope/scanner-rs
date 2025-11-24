//! Dependency tree builder for constructing hierarchical dependency relationships
//!
//! This module builds dependency trees from classified dependencies,
//! showing parent-child relationships and detecting circular dependencies.

use crate::models::{
    Application, Classification, ClassifiedDependency, DependencyNode, DependencyTree,
};
use std::collections::{HashMap, HashSet};

/// Tree builder for constructing dependency trees
pub struct TreeBuilder;

impl TreeBuilder {
    /// Create a new TreeBuilder
    pub fn new() -> Self {
        Self
    }

    /// Build dependency trees for all applications
    pub fn build_trees(&self, applications: Vec<Application>) -> Vec<DependencyTree> {
        applications
            .into_iter()
            .map(|app| self.build_tree(app))
            .collect()
    }

    /// Build a dependency tree for a single application
    pub fn build_tree(&self, application: Application) -> DependencyTree {
        let mut tree = DependencyTree::new(application.clone());

        // Create a lookup map for dependencies
        let dep_map: HashMap<String, &ClassifiedDependency> = application
            .dependencies
            .iter()
            .map(|d| (d.name.clone(), d))
            .collect();

        // Build root nodes (direct dependencies with HAS classification)
        for dep in &application.dependencies {
            if dep.has_classification(Classification::Has) {
                let mut visited = HashSet::new();
                if let Some(node) = Self::build_node(dep, &dep_map, true, &mut visited) {
                    tree.add_root(node);
                }
            }
        }

        tree
    }

    /// Build a dependency node recursively
    fn build_node(
        dep: &ClassifiedDependency,
        dep_map: &HashMap<String, &ClassifiedDependency>,
        is_direct: bool,
        visited: &mut HashSet<String>,
    ) -> Option<DependencyNode> {
        // Detect circular dependencies
        if visited.contains(&dep.name) {
            eprintln!(
                "[warn] Circular dependency detected: {} (breaking cycle)",
                dep.name
            );
            return None;
        }

        visited.insert(dep.name.clone());

        // Get the version from the primary classification
        let version = dep
            .primary_classification()
            .and_then(|c| dep.get_version(c))
            .unwrap_or("unknown")
            .to_string();

        let classification = dep.primary_classification().unwrap_or(Classification::Can);

        let mut node = DependencyNode::new(dep.name.clone(), version, classification, is_direct);

        // Build child nodes for dependencies
        for child_name in &dep.dependencies {
            if let Some(child_dep) = dep_map.get(child_name) {
                if let Some(child_node) = Self::build_node(child_dep, dep_map, false, visited) {
                    node.add_dependency(child_node);
                }
            }
        }

        visited.remove(&dep.name);

        Some(node)
    }
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Ecosystem;
    use std::path::PathBuf;

    #[test]
    fn test_build_simple_tree() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/app"),
            PathBuf::from("/app/package.json"),
            Ecosystem::Node,
        );

        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );

        app.add_dependency(dep);

        let builder = TreeBuilder::new();
        let tree = builder.build_tree(app);

        assert_eq!(tree.application.name, "myapp");
        assert_eq!(tree.roots.len(), 1);
        assert_eq!(tree.roots[0].name, "react");
        assert_eq!(tree.roots[0].version, "18.2.0");
        assert!(tree.roots[0].is_direct);
    }

    #[test]
    fn test_build_tree_with_transitive_deps() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/app"),
            PathBuf::from("/app/package.json"),
            Ecosystem::Node,
        );

        // Parent dependency
        let mut react = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        react.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );
        react.dependencies.push("loose-envify".to_string());

        // Child dependency
        let mut loose_envify =
            ClassifiedDependency::new("loose-envify".to_string(), Ecosystem::Node);
        loose_envify.add_classification(
            Classification::Has,
            "1.4.0".to_string(),
            PathBuf::from("/app/node_modules/loose-envify"),
        );

        app.add_dependency(react);
        app.add_dependency(loose_envify);

        let builder = TreeBuilder::new();
        let tree = builder.build_tree(app);

        // Both packages have HAS classification, so both are roots
        // React has loose-envify as a child in its dependency tree
        assert_eq!(tree.roots.len(), 2);

        let react_node = tree.roots.iter().find(|n| n.name == "react").unwrap();
        assert_eq!(react_node.dependencies.len(), 1);
        assert_eq!(react_node.dependencies[0].name, "loose-envify");
        assert!(!react_node.dependencies[0].is_direct);
    }

    #[test]
    fn test_build_tree_multiple_roots() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/app"),
            PathBuf::from("/app/package.json"),
            Ecosystem::Node,
        );

        let mut react = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        react.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );

        let mut lodash = ClassifiedDependency::new("lodash".to_string(), Ecosystem::Node);
        lodash.add_classification(
            Classification::Has,
            "4.17.21".to_string(),
            PathBuf::from("/app/node_modules/lodash"),
        );

        app.add_dependency(react);
        app.add_dependency(lodash);

        let builder = TreeBuilder::new();
        let tree = builder.build_tree(app);

        assert_eq!(tree.roots.len(), 2);
        assert!(tree.roots.iter().any(|n| n.name == "react"));
        assert!(tree.roots.iter().any(|n| n.name == "lodash"));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/app"),
            PathBuf::from("/app/package.json"),
            Ecosystem::Node,
        );

        // Create circular dependency: a -> b -> a
        let mut dep_a = ClassifiedDependency::new("pkg-a".to_string(), Ecosystem::Node);
        dep_a.add_classification(
            Classification::Has,
            "1.0.0".to_string(),
            PathBuf::from("/app/node_modules/pkg-a"),
        );
        dep_a.dependencies.push("pkg-b".to_string());

        let mut dep_b = ClassifiedDependency::new("pkg-b".to_string(), Ecosystem::Node);
        dep_b.add_classification(
            Classification::Has,
            "1.0.0".to_string(),
            PathBuf::from("/app/node_modules/pkg-b"),
        );
        dep_b.dependencies.push("pkg-a".to_string());

        app.add_dependency(dep_a);
        app.add_dependency(dep_b);

        let builder = TreeBuilder::new();
        let tree = builder.build_tree(app);

        // Should build tree but break the cycle
        assert_eq!(tree.roots.len(), 2);
    }

    #[test]
    fn test_build_trees_multiple_apps() {
        let mut app1 = Application::new(
            "app1".to_string(),
            PathBuf::from("/app1"),
            PathBuf::from("/app1/package.json"),
            Ecosystem::Node,
        );

        let mut dep1 = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep1.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app1/node_modules/react"),
        );
        app1.add_dependency(dep1);

        let mut app2 = Application::new(
            "app2".to_string(),
            PathBuf::from("/app2"),
            PathBuf::from("/app2/package.json"),
            Ecosystem::Node,
        );

        let mut dep2 = ClassifiedDependency::new("lodash".to_string(), Ecosystem::Node);
        dep2.add_classification(
            Classification::Has,
            "4.17.21".to_string(),
            PathBuf::from("/app2/node_modules/lodash"),
        );
        app2.add_dependency(dep2);

        let builder = TreeBuilder::new();
        let trees = builder.build_trees(vec![app1, app2]);

        assert_eq!(trees.len(), 2);
        assert_eq!(trees[0].application.name, "app1");
        assert_eq!(trees[1].application.name, "app2");
    }

    #[test]
    fn test_only_has_classification_in_tree() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/app"),
            PathBuf::from("/app/package.json"),
            Ecosystem::Node,
        );

        // Dependency with only SHOULD classification (no HAS)
        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Should,
            "18.2.0".to_string(),
            PathBuf::from("/app/package-lock.json"),
        );

        app.add_dependency(dep);

        let builder = TreeBuilder::new();
        let tree = builder.build_tree(app);

        // Should not include in tree since it's not installed (no HAS)
        assert_eq!(tree.roots.len(), 0);
    }
}
