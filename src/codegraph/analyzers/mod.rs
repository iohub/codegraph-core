// 语言分析器模块
pub mod java_analyzer;
pub mod python_analyzer;
pub mod typescript_analyzer;
pub mod javascript_analyzer;
pub mod cpp_analyzer;

// 语言特定解析器
pub mod java_parser;
pub mod typescript_parser;
pub mod javascript_parser;
pub mod rust_parser;
pub mod utils;

// 重新导出主要的分析器
pub use java_analyzer::JavaAnalyzer;
pub use python_analyzer::PythonAnalyzer;
pub use typescript_analyzer::TypeScriptAnalyzer;
pub use javascript_analyzer::JavaScriptAnalyzer;
pub use cpp_analyzer::CppAnalyzer;

// 重新导出解析器
pub use java_parser::JavaParser;
pub use typescript_parser::TypeScriptParser;
pub use javascript_parser::JavaScriptParser;
pub use rust_parser::RustParser;

// 导出通用接口和错误类型
use std::fmt::Display;
use std::path::PathBuf;
use crate::codegraph::treesitter::ast_instance_structs::AstSymbolInstanceArc;

#[derive(Debug, PartialEq, Eq)]
pub struct ParserError {
    pub message: String,
}

impl From<Box<dyn std::error::Error>> for ParserError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        ParserError {
            message: err.to_string(),
        }
    }
}

pub trait AstLanguageParser: Send {
    fn parse(&mut self, code: &str, path: &PathBuf) -> Vec<AstSymbolInstanceArc>;
}

// 新的统一接口，使用Analyzer系统
pub trait CodeAnalyzer: Send {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String>;
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String>;
}

// 为现有的Analyzer实现CodeAnalyzer trait
impl CodeAnalyzer for JavaAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        JavaAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        JavaAnalyzer::analyze_directory(self, dir.as_path())
    }
}

impl CodeAnalyzer for PythonAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        PythonAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        PythonAnalyzer::analyze_directory(self, dir.as_path())
    }
}

impl CodeAnalyzer for CppAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        CppAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        CppAnalyzer::analyze_directory(self, dir.as_path())
    }
}

impl CodeAnalyzer for TypeScriptAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        TypeScriptAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        TypeScriptAnalyzer::analyze_directory(self, dir.as_path())
    }
}

impl CodeAnalyzer for JavaScriptAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        JavaScriptAnalyzer::analyze_file(self, path.as_path())
            .map_err(|e| e.to_string())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        JavaScriptAnalyzer::analyze_directory(self, dir.as_path())
            .map_err(|e| e.to_string())
    }
}

fn internal_error<E: Display>(err: E) -> ParserError {
    let err_msg = err.to_string();
    tracing::error!(err_msg);
    ParserError {
        message: err_msg.into(),
    }
}

// 重构后的get_ast_parser函数，现在返回CodeAnalyzer
pub(crate) fn get_code_analyzer(language_id: crate::codegraph::treesitter::language_id::LanguageId) -> Result<Box<dyn CodeAnalyzer + 'static>, ParserError> {
    match language_id {
        crate::codegraph::treesitter::language_id::LanguageId::Rust => {
            // 暂时返回错误，因为RustAnalyzer还没有实现
            Err(ParserError {
                message: "Rust analyzer not yet implemented".to_string()
            })
        }
        crate::codegraph::treesitter::language_id::LanguageId::Python => {
            let analyzer = PythonAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::Java => {
            let analyzer = JavaAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::Cpp => {
            let analyzer = CppAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::TypeScript => {
            let analyzer = TypeScriptAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::JavaScript => {
            let analyzer = JavaScriptAnalyzer::new()
                .map_err(|e| ParserError { message: e.to_string() })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::TypeScriptReact => {
            let analyzer = TypeScriptAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        other => Err(ParserError {
            message: "Unsupported language id: ".to_string() + &other.to_string()
        }),
    }
}

// 保持向后兼容的get_ast_parser函数，但现在返回CodeAnalyzer
pub(crate) fn get_ast_parser(language_id: crate::codegraph::treesitter::language_id::LanguageId) -> Result<Box<dyn CodeAnalyzer + 'static>, ParserError> {
    get_code_analyzer(language_id)
}

pub fn get_ast_parser_by_filename(filename: &PathBuf) -> Result<(Box<dyn CodeAnalyzer + 'static>, crate::codegraph::treesitter::language_id::LanguageId), ParserError> {
    let suffix = filename.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let maybe_language_id = get_language_id_by_filename(filename);
    match maybe_language_id {
        Some(language_id) => {
            let analyzer = get_code_analyzer(language_id)?;
            Ok((analyzer, language_id))
        }
        None => Err(ParserError { message: format!("not supported {}", suffix) }),
    }
}

pub fn get_language_id_by_filename(filename: &PathBuf) -> Option<crate::codegraph::treesitter::language_id::LanguageId> {
    let suffix = filename.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match suffix.as_str() {
        "cpp" | "cc" | "cxx" | "c++" | "c" | "h" | "hpp" | "hxx" | "hh" => Some(crate::codegraph::treesitter::language_id::LanguageId::Cpp),
        "inl" | "inc" | "tpp" | "tpl" => Some(crate::codegraph::treesitter::language_id::LanguageId::Cpp),
        "py" | "py3" | "pyx" => Some(crate::codegraph::treesitter::language_id::LanguageId::Python),
        "java" => Some(crate::codegraph::treesitter::language_id::LanguageId::Java),
        "js" | "jsx" => Some(crate::codegraph::treesitter::language_id::LanguageId::JavaScript),
        "rs" => Some(crate::codegraph::treesitter::language_id::LanguageId::Rust),
        "ts" | "tsx" => Some(crate::codegraph::treesitter::language_id::LanguageId::TypeScript),
        _ => None
    }
} 