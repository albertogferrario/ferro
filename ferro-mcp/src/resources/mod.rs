//! MCP Resources for Ferro Framework introspection

pub mod error_patterns;
pub mod glossary;

pub use error_patterns::{get_error_patterns, ErrorPattern, ErrorPatternsCatalog};
pub use glossary::{generate_glossary, DomainGlossary, GlossaryEntry};
