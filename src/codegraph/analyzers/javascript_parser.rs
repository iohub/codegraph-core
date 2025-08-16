use std::path::PathBuf;
use crate::codegraph::analyzers::{ParserError, AstLanguageParser};
use crate::codegraph::treesitter::ast_instance_structs::AstSymbolInstanceArc;

pub struct JavaScriptParser;

impl JavaScriptParser {
    pub fn new() -> Result<Self, ParserError> {
        Ok(Self)
    }
}

impl AstLanguageParser for JavaScriptParser {
    fn parse(&mut self, _code: &str, _path: &PathBuf) -> Vec<AstSymbolInstanceArc> {
        // TODO: Implement JavaScript parsing
        Vec::new()
    }
} 