use std::path::PathBuf;
use crate::codegraph::treesitter::parsers::{ParserError, AstLanguageParser};
use crate::codegraph::treesitter::ast_instance_structs::AstSymbolInstanceArc;

pub struct JSParser;

impl JSParser {
    pub fn new() -> Result<Self, ParserError> {
        Ok(Self)
    }
}

impl AstLanguageParser for JSParser {
    fn parse(&mut self, _code: &str, _path: &PathBuf) -> Vec<AstSymbolInstanceArc> {
        // TODO: Implement JavaScript parsing
        Vec::new()
    }
} 