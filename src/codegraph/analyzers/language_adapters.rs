use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use anyhow::Result;

use crate::codegraph::treesitter::language_id::LanguageId;
use crate::codegraph::types::{FunctionInfo, CallRelation, ParameterInfo};
use crate::codegraph::analyzers::{
    LanguageAnalyzer, ParsedUnit, Snippet, CodeAnalyzer,
    RustAnalyzer, JavaAnalyzer, PythonAnalyzer, CppAnalyzer, TypeScriptAnalyzer, JavaScriptAnalyzer,
};

// Rust分析器适配器
pub struct RustLanguageAnalyzer;

impl RustLanguageAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl LanguageAnalyzer for RustLanguageAnalyzer {
    fn language(&self) -> LanguageId {
        LanguageId::Rust
    }
    
    fn parse_file(&self, path: &Path) -> Result<ParsedUnit> {
        let content = fs::read_to_string(path)?;
        
        let ast_nodes = Vec::new(); // TODO: 从analyzer中获取AST节点
        
        Ok(ParsedUnit {
            file_path: path.to_path_buf(),
            language: LanguageId::Rust,
            content,
            ast_nodes,
        })
    }
    
    fn extract_functions(&self, unit: &ParsedUnit) -> Vec<FunctionInfo> {
        // 使用现有的Rust分析器提取函数
        let mut analyzer = match RustAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取函数信息
        match analyzer.extract_functions(&unit.file_path) {
            Ok(functions) => functions,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_calls(&self, unit: &ParsedUnit) -> Vec<CallRelation> {
        // 使用现有的Rust分析器提取调用关系
        let mut analyzer = match RustAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取调用关系
        match analyzer.extract_call_relations(&unit.file_path) {
            Ok(calls) => calls,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_snippets(&self, unit: &ParsedUnit) -> Vec<Snippet> {
        let functions = self.extract_functions(unit);
        let mut snippets = Vec::new();
        
        for function in functions {
            // 从文件内容中提取代码片段
            let content = &unit.content;
            let lines: Vec<&str> = content.lines().collect();
            
            let start_line = function.line_start.saturating_sub(1); // 转换为0索引
            let end_line = function.line_end.saturating_sub(1);
            
            let mut snippet_content = String::new();
            if start_line < lines.len() && end_line < lines.len() {
                for i in start_line..=end_line {
                    snippet_content.push_str(lines[i]);
                    snippet_content.push('\n');
                }
            }
            
            let snippet = Snippet {
                id: Uuid::new_v4(),
                file_path: unit.file_path.clone(),
                language: "rust".to_string(),
                range: (function.line_start, 0, function.line_end, 0),
                function_id: Some(function.id),
                preview: Some(snippet_content.trim().to_string()),
            };
            snippets.push(snippet);
        }
        
        snippets
    }
}

// Java分析器适配器
pub struct JavaLanguageAnalyzer;

impl JavaLanguageAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl LanguageAnalyzer for JavaLanguageAnalyzer {
    fn language(&self) -> LanguageId {
        LanguageId::Java
    }
    
    fn parse_file(&self, path: &Path) -> Result<ParsedUnit> {
        let content = fs::read_to_string(path)?;
        
        let ast_nodes = Vec::new(); // TODO: 从analyzer中获取AST节点
        
        Ok(ParsedUnit {
            file_path: path.to_path_buf(),
            language: LanguageId::Java,
            content,
            ast_nodes,
        })
    }
    
    fn extract_functions(&self, unit: &ParsedUnit) -> Vec<FunctionInfo> {
        // 使用现有的Java分析器提取函数
        let mut analyzer = match JavaAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取函数信息
        match analyzer.extract_functions(&unit.file_path) {
            Ok(functions) => functions,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_calls(&self, unit: &ParsedUnit) -> Vec<CallRelation> {
        // 使用现有的Java分析器提取调用关系
        let mut analyzer = match JavaAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取调用关系
        match analyzer.extract_call_relations(&unit.file_path) {
            Ok(calls) => calls,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_snippets(&self, unit: &ParsedUnit) -> Vec<Snippet> {
        let functions = self.extract_functions(unit);
        let mut snippets = Vec::new();
        
        for function in functions {
            // 从文件内容中提取代码片段
            let content = &unit.content;
            let lines: Vec<&str> = content.lines().collect();
            
            let start_line = function.line_start.saturating_sub(1); // 转换为0索引
            let end_line = function.line_end.saturating_sub(1);
            
            let mut snippet_content = String::new();
            if start_line < lines.len() && end_line < lines.len() {
                for i in start_line..=end_line {
                    snippet_content.push_str(lines[i]);
                    snippet_content.push('\n');
                }
            }
            
            let snippet = Snippet {
                id: Uuid::new_v4(),
                file_path: unit.file_path.clone(),
                language: "java".to_string(),
                range: (function.line_start, 0, function.line_end, 0),
                function_id: Some(function.id),
                preview: Some(snippet_content.trim().to_string()),
            };
            snippets.push(snippet);
        }
        
        snippets
    }
}

// Python分析器适配器
pub struct PythonLanguageAnalyzer;

impl PythonLanguageAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl LanguageAnalyzer for PythonLanguageAnalyzer {
    fn language(&self) -> LanguageId {
        LanguageId::Python
    }
    
    fn parse_file(&self, path: &Path) -> Result<ParsedUnit> {
        let content = fs::read_to_string(path)?;
        
        let ast_nodes = Vec::new(); // TODO: 从analyzer中获取AST节点
        
        Ok(ParsedUnit {
            file_path: path.to_path_buf(),
            language: LanguageId::Python,
            content,
            ast_nodes,
        })
    }
    
    fn extract_functions(&self, unit: &ParsedUnit) -> Vec<FunctionInfo> {
        // 使用现有的Python分析器提取函数
        let mut analyzer = match PythonAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取函数信息
        match analyzer.extract_functions(&unit.file_path) {
            Ok(functions) => functions,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_calls(&self, unit: &ParsedUnit) -> Vec<CallRelation> {
        // 使用现有的Python分析器提取调用关系
        let mut analyzer = match PythonAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取调用关系
        match analyzer.extract_call_relations(&unit.file_path) {
            Ok(calls) => calls,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_snippets(&self, unit: &ParsedUnit) -> Vec<Snippet> {
        let functions = self.extract_functions(unit);
        let mut snippets = Vec::new();
        
        for function in functions {
            // 从文件内容中提取代码片段
            let content = &unit.content;
            let lines: Vec<&str> = content.lines().collect();
            
            let start_line = function.line_start.saturating_sub(1); // 转换为0索引
            let end_line = function.line_end.saturating_sub(1);
            
            let mut snippet_content = String::new();
            if start_line < lines.len() && end_line < lines.len() {
                for i in start_line..=end_line {
                    snippet_content.push_str(lines[i]);
                    snippet_content.push('\n');
                }
            }
            
            let snippet = Snippet {
                id: Uuid::new_v4(),
                file_path: unit.file_path.clone(),
                language: "python".to_string(),
                range: (function.line_start, 0, function.line_end, 0),
                function_id: Some(function.id),
                preview: Some(snippet_content.trim().to_string()),
            };
            snippets.push(snippet);
        }
        
        snippets
    }
}

// C++分析器适配器
pub struct CppLanguageAnalyzer;

impl CppLanguageAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl LanguageAnalyzer for CppLanguageAnalyzer {
    fn language(&self) -> LanguageId {
        LanguageId::Cpp
    }
    
    fn parse_file(&self, path: &Path) -> Result<ParsedUnit> {
        let content = fs::read_to_string(path)?;
        
        let ast_nodes = Vec::new(); // TODO: 从analyzer中获取AST节点
        
        Ok(ParsedUnit {
            file_path: path.to_path_buf(),
            language: LanguageId::Cpp,
            content,
            ast_nodes,
        })
    }
    
    fn extract_functions(&self, unit: &ParsedUnit) -> Vec<FunctionInfo> {
        // 使用现有的C++分析器提取函数
        let mut analyzer = match CppAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取函数信息
        match analyzer.extract_functions(&unit.file_path) {
            Ok(functions) => functions,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_calls(&self, unit: &ParsedUnit) -> Vec<CallRelation> {
        // 使用现有的C++分析器提取调用关系
        let mut analyzer = match CppAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取调用关系
        match analyzer.extract_call_relations(&unit.file_path) {
            Ok(calls) => calls,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_snippets(&self, unit: &ParsedUnit) -> Vec<Snippet> {
        let functions = self.extract_functions(unit);
        let mut snippets = Vec::new();
        
        for function in functions {
            // 从文件内容中提取代码片段
            let content = &unit.content;
            let lines: Vec<&str> = content.lines().collect();
            
            let start_line = function.line_start.saturating_sub(1); // 转换为0索引
            let end_line = function.line_end.saturating_sub(1);
            
            let mut snippet_content = String::new();
            if start_line < lines.len() && end_line < lines.len() {
                for i in start_line..=end_line {
                    snippet_content.push_str(lines[i]);
                    snippet_content.push('\n');
                }
            }
            
            let snippet = Snippet {
                id: Uuid::new_v4(),
                file_path: unit.file_path.clone(),
                language: "cpp".to_string(),
                range: (function.line_start, 0, function.line_end, 0),
                function_id: Some(function.id),
                preview: Some(snippet_content.trim().to_string()),
            };
            snippets.push(snippet);
        }
        
        snippets
    }
}

// TypeScript分析器适配器
pub struct TypeScriptLanguageAnalyzer;

impl TypeScriptLanguageAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl LanguageAnalyzer for TypeScriptLanguageAnalyzer {
    fn language(&self) -> LanguageId {
        LanguageId::TypeScript
    }
    
    fn parse_file(&self, path: &Path) -> Result<ParsedUnit> {
        let content = fs::read_to_string(path)?;
        
        let ast_nodes = Vec::new(); // TODO: 从analyzer中获取AST节点
        
        Ok(ParsedUnit {
            file_path: path.to_path_buf(),
            language: LanguageId::TypeScript,
            content,
            ast_nodes,
        })
    }
    
    fn extract_functions(&self, unit: &ParsedUnit) -> Vec<FunctionInfo> {
        // 使用现有的TypeScript分析器提取函数
        let mut analyzer = match TypeScriptAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取函数信息
        match analyzer.extract_functions(&unit.file_path) {
            Ok(functions) => functions,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_calls(&self, unit: &ParsedUnit) -> Vec<CallRelation> {
        // 使用现有的TypeScript分析器提取调用关系
        let mut analyzer = match TypeScriptAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取调用关系
        match analyzer.extract_call_relations(&unit.file_path) {
            Ok(calls) => calls,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_snippets(&self, unit: &ParsedUnit) -> Vec<Snippet> {
        let functions = self.extract_functions(unit);
        let mut snippets = Vec::new();
        
        for function in functions {
            // 从文件内容中提取代码片段
            let content = &unit.content;
            let lines: Vec<&str> = content.lines().collect();
            
            let start_line = function.line_start.saturating_sub(1); // 转换为0索引
            let end_line = function.line_end.saturating_sub(1);
            
            let mut snippet_content = String::new();
            if start_line < lines.len() && end_line < lines.len() {
                for i in start_line..=end_line {
                    snippet_content.push_str(lines[i]);
                    snippet_content.push('\n');
                }
            }
            
            let snippet = Snippet {
                id: Uuid::new_v4(),
                file_path: unit.file_path.clone(),
                language: "typescript".to_string(),
                range: (function.line_start, 0, function.line_end, 0),
                function_id: Some(function.id),
                preview: Some(snippet_content.trim().to_string()),
            };
            snippets.push(snippet);
        }
        
        snippets
    }
}

// JavaScript分析器适配器
pub struct JavaScriptLanguageAnalyzer;

impl JavaScriptLanguageAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl LanguageAnalyzer for JavaScriptLanguageAnalyzer {
    fn language(&self) -> LanguageId {
        LanguageId::JavaScript
    }
    
    fn parse_file(&self, path: &Path) -> Result<ParsedUnit> {
        let content = fs::read_to_string(path)?;
        
        let ast_nodes = Vec::new(); // TODO: 从analyzer中获取AST节点
        
        Ok(ParsedUnit {
            file_path: path.to_path_buf(),
            language: LanguageId::JavaScript,
            content,
            ast_nodes,
        })
    }
    
    fn extract_functions(&self, unit: &ParsedUnit) -> Vec<FunctionInfo> {
        // 使用现有的JavaScript分析器提取函数
        let mut analyzer = match JavaScriptAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取函数信息
        match analyzer.extract_functions(&unit.file_path) {
            Ok(functions) => functions,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_calls(&self, unit: &ParsedUnit) -> Vec<CallRelation> {
        // 使用现有的JavaScript分析器提取调用关系
        let mut analyzer = match JavaScriptAnalyzer::new() {
            Ok(analyzer) => analyzer,
            Err(_) => return Vec::new(),
        };
        
        // 分析文件
        if let Err(_) = analyzer.analyze_file(&unit.file_path) {
            return Vec::new();
        }
        
        // 提取调用关系
        match analyzer.extract_call_relations(&unit.file_path) {
            Ok(calls) => calls,
            Err(_) => Vec::new(),
        }
    }
    
    fn extract_snippets(&self, unit: &ParsedUnit) -> Vec<Snippet> {
        let functions = self.extract_functions(unit);
        let mut snippets = Vec::new();
        
        for function in functions {
            // 从文件内容中提取代码片段
            let content = &unit.content;
            let lines: Vec<&str> = content.lines().collect();
            
            let start_line = function.line_start.saturating_sub(1); // 转换为0索引
            let end_line = function.line_end.saturating_sub(1);
            
            let mut snippet_content = String::new();
            if start_line < lines.len() && end_line < lines.len() {
                for i in start_line..=end_line {
                    snippet_content.push_str(lines[i]);
                    snippet_content.push('\n');
                }
            }
            
            let snippet = Snippet {
                id: Uuid::new_v4(),
                file_path: unit.file_path.clone(),
                language: "javascript".to_string(),
                range: (function.line_start, 0, function.line_end, 0),
                function_id: Some(function.id),
                preview: Some(snippet_content.trim().to_string()),
            };
            snippets.push(snippet);
        }
        
        snippets
    }
} 