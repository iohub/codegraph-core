pub mod language_id;
pub mod parsers;
pub mod structs;
pub mod ast_instance_structs;
pub mod skeletonizer;
pub mod file_ast_markup;
pub mod queries;

use std::path::PathBuf;
use crate::codegraph::treesitter::parsers::{get_ast_parser_by_filename, ParserError, CodeAnalyzer};

pub use language_id::LanguageId;
pub use structs::*;
pub use ast_instance_structs::*;
pub use skeletonizer::*;
pub use file_ast_markup::*;

/// TreeSitter解析器的主要接口
pub struct TreeSitterParser;

impl TreeSitterParser {
    /// 创建新的TreeSitter解析器实例
    pub fn new() -> Self {
        TreeSitterParser
    }

    /// 分析文件并返回分析结果
    pub fn analyze_file(&self, file_path: &PathBuf) -> Result<(), ParserError> {
        let (mut analyzer, _language_id) = get_ast_parser_by_filename(file_path)?;
        
        // 使用CodeAnalyzer接口分析文件
        analyzer.analyze_file(file_path)
            .map_err(|e| ParserError {
                message: format!("Failed to analyze file {}: {}", file_path.display(), e)
            })
    }

    /// 分析目录并返回分析结果
    pub fn analyze_directory(&self, dir_path: &PathBuf) -> Result<(), ParserError> {
        let (mut analyzer, _language_id) = get_ast_parser_by_filename(dir_path)?;
        
        // 使用CodeAnalyzer接口分析目录
        analyzer.analyze_directory(dir_path)
            .map_err(|e| ParserError {
                message: format!("Failed to analyze directory {}: {}", dir_path.display(), e)
            })
    }

    /// 保持向后兼容的parse_file方法
    /// 注意：现在返回空结果，因为我们现在使用Analyzer系统
    pub fn parse_file(&self, file_path: &PathBuf) -> Result<Vec<AstSymbolInstanceArc>, ParserError> {
        // 先分析文件
        self.analyze_file(file_path)?;
        
        // 返回空结果，因为我们现在使用Analyzer系统
        // 如果需要获取分析结果，应该直接使用相应的Analyzer
        Ok(Vec::new())
    }
} 