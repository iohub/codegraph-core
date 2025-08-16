use std::fmt::Display;
use std::path::PathBuf;

use tracing::error;

use crate::codegraph::treesitter::ast_instance_structs::AstSymbolInstanceArc;
use crate::codegraph::treesitter::language_id::LanguageId;
use crate::codegraph::{PythonAnalyzer, CppAnalyzer, TypeScriptAnalyzer, JavaAnalyzer, JavaScriptAnalyzer};

pub(crate) mod rust;
// #[cfg(test)]
// mod tests;
mod utils;
mod java;
mod ts;
mod js;

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

impl CodeAnalyzer for JavaAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        JavaAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        JavaAnalyzer::analyze_directory(self, dir.as_path())
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
    error!(err_msg);
    ParserError {
        message: err_msg.into(),
    }
}

// 重构后的get_ast_parser函数，现在返回CodeAnalyzer
pub(crate) fn get_code_analyzer(language_id: LanguageId) -> Result<Box<dyn CodeAnalyzer + 'static>, ParserError> {
    match language_id {
        LanguageId::Rust => {
            // 暂时返回错误，因为RustAnalyzer还没有实现
            Err(ParserError {
                message: "Rust analyzer not yet implemented".to_string()
            })
        }
        LanguageId::Python => {
            let analyzer = PythonAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        LanguageId::Java => {
            let analyzer = JavaAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        LanguageId::Cpp => {
            let analyzer = CppAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        LanguageId::TypeScript => {
            let analyzer = TypeScriptAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        LanguageId::JavaScript => {
            let analyzer = JavaScriptAnalyzer::new()
                .map_err(|e| ParserError { message: e.to_string() })?;
            Ok(Box::new(analyzer))
        }
        LanguageId::TypeScriptReact => {
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
pub(crate) fn get_ast_parser(language_id: LanguageId) -> Result<Box<dyn CodeAnalyzer + 'static>, ParserError> {
    get_code_analyzer(language_id)
}

pub fn get_ast_parser_by_filename(filename: &PathBuf) -> Result<(Box<dyn CodeAnalyzer + 'static>, LanguageId), ParserError> {
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

pub fn get_language_id_by_filename(filename: &PathBuf) -> Option<LanguageId> {
    let suffix = filename.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match suffix.as_str() {
        "cpp" | "cc" | "cxx" | "c++" | "c" | "h" | "hpp" | "hxx" | "hh" => Some(LanguageId::Cpp),
        "inl" | "inc" | "tpp" | "tpl" => Some(LanguageId::Cpp),
        "py" | "py3" | "pyx" => Some(LanguageId::Python),
        "java" => Some(LanguageId::Java),
        "js" | "jsx" => Some(LanguageId::JavaScript),
        "rs" => Some(LanguageId::Rust),
        "ts" | "tsx" => Some(LanguageId::TypeScript),
        _ => None
    }
}

