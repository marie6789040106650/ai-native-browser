//! Semantic Parser Module
//! 
//! Transforms raw HTML DOM into a simplified JSON tree for AI consumption.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Semantic node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticNode {
    /// Unique interaction ID for /act calls
    pub id: u32,
    
    /// HTML tag name
    pub tag: String,
    
    /// ARIA role if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    
    /// Core visible text content
    pub text: String,
    
    /// Whether element is interactive
    pub is_interactive: bool,
    
    /// CSS selector for execution
    pub path: String,
    
    /// Child nodes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<SemanticNode>,
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum depth to traverse
    pub max_depth: usize,
    
    /// Minimum text length to include
    pub min_text_length: usize,
    
    /// Hidden element selectors to skip
    pub hidden_selectors: Vec<String>,
    
    /// Interactive tag names
    pub interactive_tags: HashSet<&'static str>,
    
    /// Interactive ARIA roles
    pub interactive_roles: HashSet<&'static str>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        let mut interactive_tags = HashSet::new();
        interactive_tags.insert("a");
        interactive_tags.insert("button");
        interactive_tags.insert("input");
        interactive_tags.insert("select");
        interactive_tags.insert("textarea");
        interactive_tags.insert("details");
        interactive_tags.insert("label");
        
        let mut interactive_roles = HashSet::new();
        interactive_roles.insert("button");
        interactive_roles.insert("searchbox");
        interactive_roles.insert("textbox");
        interactive_roles.insert("checkbox");
        interactive_roles.insert("radio");
        interactive_roles.insert("menuitem");
        interactive_roles.insert("tab");
        interactive_roles.insert("link");
        interactive_roles.insert("combobox");
        interactive_roles.insert("switch");
        
        Self {
            max_depth: 20,
            min_text_length: 1,
            hidden_selectors: vec![
                "[style*='display: none']".to_string(),
                "[style*='display:none']".to_string(),
                "[style*='opacity: 0']".to_string(),
                "[style*='opacity:0']".to_string(),
                "[style*='visibility: hidden']".to_string(),
                "[style*='visibility:hidden']".to_string(),
                "[hidden]".to_string(),
                "[aria-hidden='true']".to_string(),
            ],
            interactive_tags,
            interactive_roles,
        }
    }
}

/// Semantic parser
pub struct SemanticParser {
    config: ParserConfig,
    next_id: u32,
}

impl SemanticParser {
    /// Create a new parser
    pub fn new(config: ParserConfig) -> Self {
        Self {
            config,
            next_id: 1,
        }
    }

    /// Create with default config
    pub fn default_parser() -> Self {
        Self::new(ParserConfig::default())
    }

    /// Check if element should be skipped (hidden)
    pub fn is_hidden_element(&self, attributes: &HashSet<String>) -> bool {
        for selector in &self.config.hidden_selectors {
            // Simple check for common hidden attributes
            if attributes.contains("hidden") {
                return true;
            }
            if selector.contains("aria-hidden") && attributes.contains("aria-hidden") {
                if attributes.iter().any(|a| a.contains("true")) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if element is interactive
    pub fn is_interactive(&self, tag: &str, role: Option<&str>, attributes: &HashSet<String>) -> bool {
        // Check by tag name
        if self.config.interactive_tags.contains(tag) {
            return true;
        }
        
        // Check by ARIA role
        if let Some(r) = role {
            if self.config.interactive_roles.contains(r) {
                return true;
            }
        }
        
        // Check for common interactive attributes
        let interactive_attrs = ["onclick", "onchange", "onsubmit", "href"];
        for attr in interactive_attrs {
            if attributes.contains(attr) {
                return true;
            }
        }
        
        false
    }

    /// Generate unique ID
    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Parse from CDP DOM nodes (simplified placeholder)
    /// In real implementation, this would traverse the CDP DOM tree
    pub fn parse_from_dom(&mut self, _dom_nodes: &[serde_json::Value]) -> Vec<SemanticNode> {
        // Placeholder implementation
        // Real implementation would:
        // 1. Traverse the CDP DOM tree
        // 2. Filter hidden elements
        // 3. Identify interactive elements
        // 4. Build semantic tree
        
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = SemanticParser::default_parser();
        assert_eq!(parser.config.max_depth, 20);
    }

    #[test]
    fn test_interactive_detection() {
        let parser = SemanticParser::default_parser();
        
        // Button should be interactive
        assert!(parser.is_interactive("button", None, &HashSet::new()));
        
        // Input should be interactive
        assert!(parser.is_interactive("input", None, &HashSet::new()));
        
        // Div should not be interactive
        assert!(!parser.is_interactive("div", None, &HashSet::new()));
    }
}
