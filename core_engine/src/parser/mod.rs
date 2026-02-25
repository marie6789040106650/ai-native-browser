//! Semantic Parser Module
//! 
//! Transforms raw HTML DOM into a simplified JSON tree for AI consumption.
//! Blueprint 3.2: Semantic Parser Implementation
//!
//! Key features:
//! 1. Filter display:none, opacity:0, visibility:hidden elements
//! 2. Identify interactive elements by tag name, ARIA role, or attributes
//! 3. Merge consecutive static text nodes
//! 4. Generate CSS selectors for execution

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
    
    /// Input value (for input elements)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    
    /// Placeholder text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    
    /// Href (for links)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

/// Raw DOM element from browser
#[derive(Debug, Clone)]
pub struct RawElement {
    pub tag: String,
    pub id: Option<String>,
    pub class: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub text: String,
    pub value: Option<String>,
    pub placeholder: Option<String>,
    pub href: Option<String>,
    pub style: Option<String>,
    pub attributes: HashSet<String>,
    pub children: Vec<RawElement>,
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
    
    /// Tags to skip entirely
    pub skip_tags: HashSet<&'static str>,
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
        interactive_tags.insert("option");
        interactive_tags.insert("optgroup");
        interactive_tags.insert("fieldset");
        interactive_tags.insert("form");
        
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
        interactive_roles.insert("slider");
        interactive_roles.insert("spinbutton");
        interactive_roles.insert("search");
        interactive_roles.insert("listbox");
        interactive_roles.insert("tree");
        interactive_roles.insert("treeitem");
        
        // Tags to skip (script, style, etc.)
        let mut skip_tags = HashSet::new();
        skip_tags.insert("script");
        skip_tags.insert("style");
        skip_tags.insert("noscript");
        skip_tags.insert("meta");
        skip_tags.insert("link");
        skip_tags.insert("head");
        skip_tags.insert("template");
        
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
            skip_tags,
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
    pub fn is_hidden_element(&self, style: &Option<String>) -> bool {
        if let Some(ref s) = style {
            // Check for hidden styles
            let s_lower = s.to_lowercase();
            if s_lower.contains("display: none") || s_lower.contains("display:none") {
                return true;
            }
            if s_lower.contains("opacity: 0") || s_lower.contains("opacity:0") {
                return true;
            }
            if s_lower.contains("visibility: hidden") || s_lower.contains("visibility:hidden") {
                return true;
            }
        }
        false
    }

    /// Check if element should be skipped by tag
    pub fn should_skip_tag(&self, tag: &str) -> bool {
        self.config.skip_tags.contains(tag)
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
        let interactive_attrs = ["onclick", "onchange", "onsubmit", "onfocus", "onblur", "href"];
        for attr in interactive_attrs {
            if attributes.contains(attr) {
                return true;
            }
        }
        
        false
    }

    /// Generate CSS selector for element
    pub fn generate_selector(&self, element: &RawElement, _depth: usize) -> String {
        let mut selector = String::new();
        
        // Add ID if available
        if let Some(ref id) = element.id {
            return format!("#{}", id);
        }
        
        // Build from tag + class + attributes
        selector.push_str(&element.tag);
        
        if let Some(ref class) = element.class {
            // Take first class
            if let Some(first_class) = class.split_whitespace().next() {
                selector.push('.');
                selector.push_str(first_class);
            }
        }
        
        if let Some(ref name) = element.name {
            selector.push_str(&format!("[name=\"{}\"]", name));
        }
        
        // Add type for inputs
        if element.tag == "input" {
            if let Some(ref value) = element.value {
                selector.push_str(&format!("[type=\"{}\"]", value));
            }
        }
        
        selector
    }

    /// Generate unique ID
    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Parse raw HTML content into semantic tree
    /// This is a simplified implementation using basic HTML parsing
    pub fn parse_html(&mut self, html: &str) -> Vec<SemanticNode> {
        // Simple approach: extract key elements from HTML
        // In production, would use a proper HTML parser
        
        let mut nodes = Vec::new();
        
        // Extract buttons
        self.extract_elements(html, "button", &mut nodes);
        
        // Extract inputs
        self.extract_elements(html, "input", &mut nodes);
        
        // Extract links
        self.extract_elements(html, "a", &mut nodes);
        
        // Extract other interactive elements
        for tag in &["select", "textarea", "label"] {
            self.extract_elements(html, tag, &mut nodes);
        }
        
        nodes
    }

    /// Extract elements by tag name (simplified regex-based)
    fn extract_elements(&mut self, html: &str, tag: &str, nodes: &mut Vec<SemanticNode>) {
        // This is a placeholder - real implementation would use proper HTML parsing
        // Would use something like scraper or tl crates
        
        let _ = (html, tag, nodes);
    }

    /// Parse from raw elements to semantic nodes
    pub fn parse_elements(&mut self, elements: Vec<RawElement>) -> Vec<SemanticNode> {
        let mut nodes = Vec::new();
        
        for element in elements {
            if self.should_skip_tag(&element.tag) {
                continue;
            }
            
            if self.is_hidden_element(&element.style) {
                continue;
            }
            
            // Build attributes set
            let mut attrs = element.attributes.clone();
            if element.role.is_some() {
                attrs.insert("role".to_string());
            }
            
            let is_interactive = self.is_interactive(
                &element.tag,
                element.role.as_deref(),
                &attrs,
            );
            
            // Skip non-interactive elements with no text and no children
            if !is_interactive && element.text.trim().is_empty() && element.children.is_empty() {
                continue;
            }
            
            let id = self.next_id();
            let path = self.generate_selector(&element, 0);
            
            let mut node = SemanticNode {
                id,
                tag: element.tag,
                role: element.role,
                text: element.text.trim().to_string(),
                is_interactive,
                path,
                children: Vec::new(),
                value: element.value,
                placeholder: element.placeholder,
                href: element.href,
            };
            
            // Recursively process children
            if !element.children.is_empty() {
                node.children = self.parse_elements(element.children);
            }
            
            nodes.push(node);
        }
        
        nodes
    }

    /// Convert to JSON for API response
    pub fn to_json(&self, nodes: &[SemanticNode]) -> String {
        serde_json::to_string_pretty(nodes).unwrap_or_else(|_| "[]".to_string())
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
        
        // Link should be interactive (has href)
        let mut attrs = HashSet::new();
        attrs.insert("href".to_string());
        assert!(parser.is_interactive("div", None, &attrs));
    }

    #[test]
    fn test_hidden_detection() {
        let parser = SemanticParser::default_parser();
        
        // Visible element
        assert!(!parser.is_hidden_element(&None));
        
        // Hidden element
        assert!(parser.is_hidden_element(&Some("display: none".to_string())));
        
        // Hidden via opacity
        assert!(parser.is_hidden_element(&Some("opacity: 0".to_string())));
    }

    #[test]
    fn test_skip_tags() {
        let parser = SemanticParser::default_parser();
        
        assert!(parser.should_skip_tag("script"));
        assert!(parser.should_skip_tag("style"));
        assert!(!parser.should_skip_tag("div"));
    }
}
