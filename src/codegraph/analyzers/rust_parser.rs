use std::path::PathBuf;
use crate::codegraph::analyzers::{ParserError, AstLanguageParser};
use crate::codegraph::treesitter::ast_instance_structs::AstSymbolInstanceArc;

pub struct RustParser;

impl RustParser {
    pub fn new() -> Result<Self, ParserError> {
        Ok(Self)
    }
}

impl AstLanguageParser for RustParser {
    fn parse(&mut self, _code: &str, _path: &PathBuf) -> Vec<AstSymbolInstanceArc> {
        // TODO: Implement Rust parsing
        Vec::new()
    }
} 