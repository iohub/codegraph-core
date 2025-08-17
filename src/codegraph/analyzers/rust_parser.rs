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
        // 这个实现现在由RustAnalyzer处理
        // 保留这个trait实现以保持向后兼容性
        Vec::new()
    }
} 