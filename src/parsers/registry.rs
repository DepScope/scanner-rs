//! Parser registry for managing file format parsers

use crate::parsers::Parser;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of parsers for different file formats
pub struct ParserRegistry {
    parsers: HashMap<String, Arc<dyn Parser>>,
}

impl ParserRegistry {
    /// Create a new parser registry
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }

    /// Register a parser for a specific filename
    pub fn register(&mut self, parser: Arc<dyn Parser>) {
        let filename = parser.filename().to_string();
        self.parsers.insert(filename, parser);
    }

    /// Get a parser for a specific filename
    pub fn get_parser(&self, filename: &str) -> Option<Arc<dyn Parser>> {
        self.parsers.get(filename).cloned()
    }

    /// Get all registered filenames
    pub fn registered_filenames(&self) -> Vec<String> {
        self.parsers.keys().cloned().collect()
    }

    /// Check if a filename has a registered parser
    pub fn has_parser(&self, filename: &str) -> bool {
        self.parsers.contains_key(filename)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}
