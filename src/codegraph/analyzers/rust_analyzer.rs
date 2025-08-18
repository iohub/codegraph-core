use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tree_sitter::{Parser, Node, QueryCursor, Language, Point, StreamingIterator};
use tracing::{info, warn};
use uuid::Uuid;

use crate::codegraph::types::{FunctionInfo, ParameterInfo};
use crate::codegraph::treesitter::queries::rust::{
    RustQueries, RustSnippet, RustSnippetType, RustMethodCall, RustAnalysisResult
};

extern "C" { fn tree_sitter_rust() -> Language; }

/// Rust函数作用域信息
#[derive(Debug, Clone)]
struct FunctionScope {
    pub name: String,
    pub params: Vec<String>,
    pub body_start: Point,
    pub body_end: Point,
    pub module_name: Option<String>,
    pub struct_name: Option<String>,
    pub trait_name: Option<String>,
    pub function_id: Uuid,
    pub modifiers: Vec<String>,
    pub type_parameters: Vec<String>,
    pub return_type: Option<String>,
}

/// Rust函数调用信息
#[derive(Debug, Clone)]
struct FunctionCall {
    pub caller_name: String,
    pub called_name: String,
    pub location: Point,
    pub module_name: Option<String>,
    pub struct_name: Option<String>,
    pub trait_name: Option<String>,
    pub arguments: Vec<String>,
    pub type_arguments: Vec<String>,
}

/// Rust代码分析器
pub struct RustAnalyzer {
    parser: Parser,
    language: Language,
    queries: RustQueries,
    /// 函数名 -> 函数信息映射（用于解析调用关系）
    function_registry: HashMap<String, FunctionInfo>,
    /// 文件路径 -> 函数列表映射
    file_functions: HashMap<PathBuf, Vec<FunctionInfo>>,
}

impl RustAnalyzer {
    pub fn new() -> Result<Self, String> {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_rust() };
        
        parser.set_language(&language)
            .map_err(|e| format!("Failed to set Rust language: {}", e))?;

        let queries = RustQueries::new(&language)
            .map_err(|e| format!("Failed to create Rust queries: {}", e))?;

        Ok(Self {
            parser,
            language,
            queries,
            function_registry: HashMap::new(),
            file_functions: HashMap::new(),
        })
    }

    /// 分析目录下的所有Rust文件
    pub fn analyze_directory(&mut self, dir: &Path) -> Result<(), String> {
        info!("Starting Rust analysis for directory: {}", dir.display());
        
        let files = self.scan_rust_files(dir);
        info!("Found {} Rust files to analyze", files.len());
        
        for file_path in files {
            if let Err(e) = self.analyze_file(&file_path) {
                warn!("Failed to analyze file {}: {}", file_path.display(), e);
            }
        }
        
        info!("Rust analysis completed");
        Ok(())
    }

    /// 分析单个Rust文件
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<(), String> {
        info!("Analyzing Rust file: {}", file_path.display());
        
        let code = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
        
        let tree = self.parser.parse(&code, None)
            .ok_or_else(|| format!("Failed to parse file {}", file_path.display()))?;
        
        let root_node = tree.root_node();
        let analysis_result = self.analyze_rust_code(&code, &root_node, file_path)?;
        
        // 将分析结果转换为FunctionInfo和CallRelation
        self.process_analysis_result(analysis_result, file_path);
        
        Ok(())
    }

    /// 扫描目录下的Rust文件
    fn scan_rust_files(&self, dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self._scan_directory_recursive(dir, &mut files);
        files
    }

    fn _scan_directory_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 跳过常见的忽略目录
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "target" || name == "node_modules" || 
                           name == "__pycache__" || name == "venv" || name == "env" || name == ".venv" ||
                           name == "dist" || name == "build" || name == "coverage" {
                            continue;
                        }
                    }
                    self._scan_directory_recursive(&path, files);
                } else if self.is_rust_file(&path) {
                    files.push(path);
                }
            }
        }
    }

    /// 判断文件是否为Rust文件
    fn is_rust_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(), "rs")
        } else {
            false
        }
    }

    /// 分析Rust代码，提取函数定义、调用关系等
    fn analyze_rust_code(&self, code: &str, root_node: &Node, path: &Path) -> Result<RustAnalysisResult, String> {
        let mut result = RustAnalysisResult {
            snippets: Vec::new(),
            method_calls: Vec::new(),
            scopes: Vec::new(),
            imports: Vec::new(),
            modules: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            macros: HashMap::new(),
            types: HashMap::new(),
        };

        // 第一遍：收集模块和导入信息
        self.collect_module_and_import_info(code, root_node, path, &mut result);
        
        // 第二遍：收集函数定义和作用域
        let function_scopes = self.collect_function_definitions(code, root_node, path, &mut result)?;
        
        // 第三遍：收集结构体、枚举和trait定义
        self.collect_struct_enum_trait_definitions(code, root_node, path, &mut result);
        
        // 第四遍：收集所有函数调用
        let all_calls = self.collect_function_calls(code, root_node, path);
        
        // 第五遍：建立调用关系和作用域归属
        self.establish_call_relationships(&function_scopes, &all_calls, &mut result);

        Ok(result)
    }

    /// 收集模块和导入信息
    fn collect_module_and_import_info(&self, code: &str, root_node: &Node, path: &Path, result: &mut RustAnalysisResult) {
        let mut query_cursor = QueryCursor::new();
        
        // 收集模块声明
        let mut matches = query_cursor.matches(&self.queries.mod_definition, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.mod_definition.capture_names()[capture.index as usize];
                
                if *capture_name == "mod.name" {
                    let module_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    result.modules.insert(module_name.clone(), vec![path.to_string_lossy().to_string()]);
                }
            }
        }

        // 收集use声明
        let mut matches = query_cursor.matches(&self.queries.use_declaration, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.use_declaration.capture_names()[capture.index as usize];
                
                if *capture_name == "use.path" {
                    let import_path = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    result.imports.push(import_path);
                }
            }
        }
    }

    /// 收集函数定义
    fn collect_function_definitions(&self, code: &str, root_node: &Node, path: &Path, result: &mut RustAnalysisResult) -> Result<Vec<FunctionScope>, String> {
        let mut function_scopes = Vec::new();
        let mut query_cursor = QueryCursor::new();
        
        let mut matches = query_cursor.matches(&self.queries.function_definition, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            let mut function_name = String::new();
            let mut params = Vec::new();
            let mut body_start = Point::new(0, 0);
            let mut body_end = Point::new(0, 0);
            let mut modifiers = Vec::new();
            let mut type_parameters = Vec::new();
            let mut return_type = None;
            
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.function_definition.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "function.name" => {
                        function_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    }
                    "function.params" => {
                        params = self.extract_parameters(code, &node);
                    }
                    "function.body" => {
                        body_start = node.start_position();
                        body_end = node.end_position();
                    }
                    _ => {}
                }
            }
            
            if !function_name.is_empty() {
                let function_scope = FunctionScope {
                    name: function_name.clone(),
                    params: params.clone(),
                    body_start,
                    body_end,
                    module_name: None, // TODO: 从上下文推断
                    struct_name: None,  // TODO: 从上下文推断
                    trait_name: None,   // TODO: 从上下文推断
                    function_id: Uuid::new_v4(),
                    modifiers: modifiers.clone(),
                    type_parameters: type_parameters.clone(),
                    return_type: return_type.clone(),
                };
                
                function_scopes.push(function_scope.clone());
                
                // 添加到snippets
                let snippet = RustSnippet {
                    snippet_type: RustSnippetType::Function,
                    name: function_name.clone(),
                    start_line: body_start.row,
                    end_line: body_end.row,
                    file_path: path.to_string_lossy().to_string(),
                    content: String::new(), // TODO: 提取实际代码内容
                    scope: None,
                    modifiers,
                    generics: type_parameters,
                    parameters: params,
                    return_type,
                };
                result.snippets.push(snippet);
            }
        }
        
        Ok(function_scopes)
    }

    /// 收集结构体、枚举和trait定义
    fn collect_struct_enum_trait_definitions(&self, code: &str, root_node: &Node, path: &Path, result: &mut RustAnalysisResult) {
        let mut query_cursor = QueryCursor::new();
        
        // 收集结构体定义
        let mut matches = query_cursor.matches(&self.queries.struct_definition, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.struct_definition.capture_names()[capture.index as usize];
                
                if *capture_name == "struct.name" {
                    let struct_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    result.structs.insert(struct_name.clone(), vec![path.to_string_lossy().to_string()]);
                    
                    let snippet = RustSnippet {
                        snippet_type: RustSnippetType::Struct,
                        name: struct_name,
                        start_line: node.start_position().row,
                        end_line: node.end_position().row,
                        file_path: path.to_string_lossy().to_string(),
                        content: node.utf8_text(code.as_bytes()).unwrap().to_string(),
                        scope: None,
                        modifiers: Vec::new(),
                        generics: Vec::new(),
                        parameters: Vec::new(),
                        return_type: None,
                    };
                    result.snippets.push(snippet);
                }
            }
        }
        
        // 收集枚举定义
        let mut matches = query_cursor.matches(&self.queries.enum_definition, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.enum_definition.capture_names()[capture.index as usize];
                
                if *capture_name == "enum.name" {
                    let enum_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    result.enums.insert(enum_name.clone(), vec![path.to_string_lossy().to_string()]);
                    
                    let snippet = RustSnippet {
                        snippet_type: RustSnippetType::Enum,
                        name: enum_name,
                        start_line: node.start_position().row,
                        end_line: node.end_position().row,
                        file_path: path.to_string_lossy().to_string(),
                        content: node.utf8_text(code.as_bytes()).unwrap().to_string(),
                        scope: None,
                        modifiers: Vec::new(),
                        generics: Vec::new(),
                        parameters: Vec::new(),
                        return_type: None,
                    };
                    result.snippets.push(snippet);
                }
            }
        }
        
        // 收集trait定义
        let mut matches = query_cursor.matches(&self.queries.trait_definition, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.trait_definition.capture_names()[capture.index as usize];
                
                if *capture_name == "trait.name" {
                    let trait_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    result.traits.insert(trait_name.clone(), vec![path.to_string_lossy().to_string()]);
                    
                    let snippet = RustSnippet {
                        snippet_type: RustSnippetType::Trait,
                        name: trait_name,
                        start_line: node.start_position().row,
                        end_line: node.end_position().row,
                        file_path: path.to_string_lossy().to_string(),
                        content: node.utf8_text(code.as_bytes()).unwrap().to_string(),
                        scope: None,
                        modifiers: Vec::new(),
                        generics: Vec::new(),
                        parameters: Vec::new(),
                        return_type: None,
                    };
                    result.snippets.push(snippet);
                }
            }
        }
    }

    /// 收集函数调用
    fn collect_function_calls(&self, code: &str, root_node: &Node, path: &Path) -> Vec<FunctionCall> {
        let mut function_calls = Vec::new();
        let mut query_cursor = QueryCursor::new();
        
        let mut matches = query_cursor.matches(&self.queries.function_call, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            let mut called_name = String::new();
            let mut arguments = Vec::new();
            let mut type_arguments = Vec::new();
            let mut location = Point::new(0, 0);
            
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.function_call.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "function.called" | "method.called" => {
                        called_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                        location = node.start_position();
                    }
                    "function.args" | "method.args" => {
                        arguments = self.extract_arguments(code, &node);
                    }
                    "function.type_args" => {
                        type_arguments = self.extract_type_arguments(code, &node);
                    }
                    _ => {}
                }
            }
            
            if !called_name.is_empty() {
                let function_call = FunctionCall {
                    caller_name: String::new(), // TODO: 从上下文推断
                    called_name,
                    location,
                    module_name: None,
                    struct_name: None,
                    trait_name: None,
                    arguments,
                    type_arguments,
                };
                function_calls.push(function_call);
            }
        }
        
        function_calls
    }

    /// 建立调用关系和作用域归属
    fn establish_call_relationships(&self, function_scopes: &[FunctionScope], all_calls: &[FunctionCall], result: &mut RustAnalysisResult) {
        for call in all_calls {
            // 查找调用者所在的作用域
            let caller_scope = self.find_caller_scope(call, function_scopes);
            
            let method_call = RustMethodCall {
                caller_name: caller_scope.map(|s| s.name.clone()).unwrap_or_default(),
                called_name: call.called_name.clone(),
                location: (call.location.row, call.location.column),
                file_path: String::new(), // TODO: 从上下文获取
                arguments: call.arguments.clone(),
                type_arguments: call.type_arguments.clone(),
            };
            result.method_calls.push(method_call);
        }
    }

    /// 查找调用者所在的作用域
    fn find_caller_scope<'a>(&self, call: &FunctionCall, function_scopes: &'a [FunctionScope]) -> Option<&'a FunctionScope> {
        // 简单的实现：找到包含调用位置的作用域
        for scope in function_scopes {
            if call.location.row >= scope.body_start.row && call.location.row <= scope.body_end.row {
                return Some(scope);
            }
        }
        None
    }

    /// 提取参数信息
    fn extract_parameters(&self, code: &str, params_node: &Node) -> Vec<String> {
        let mut params = Vec::new();
        let mut cursor = params_node.walk();
        
        for child in params_node.children(&mut cursor) {
            if child.kind() == "parameter" {
                let param_text = child.utf8_text(code.as_bytes()).unwrap().to_string();
                params.push(param_text);
            }
        }
        
        params
    }

    /// 提取参数信息
    fn extract_arguments(&self, code: &str, args_node: &Node) -> Vec<String> {
        let mut args = Vec::new();
        let mut cursor = args_node.walk();
        
        for child in args_node.children(&mut cursor) {
            if child.kind() != "," {
                let arg_text = child.utf8_text(code.as_bytes()).unwrap().to_string();
                args.push(arg_text);
            }
        }
        
        args
    }

    /// 提取类型参数信息
    fn extract_type_arguments(&self, code: &str, type_args_node: &Node) -> Vec<String> {
        let mut type_args = Vec::new();
        let mut cursor = type_args_node.walk();
        
        for child in type_args_node.children(&mut cursor) {
            if child.kind() != "," && child.kind() != "<" && child.kind() != ">" {
                let type_arg_text = child.utf8_text(code.as_bytes()).unwrap().to_string();
                type_args.push(type_arg_text);
            }
        }
        
        type_args
    }

    /// 处理分析结果，转换为FunctionInfo
    fn process_analysis_result(&mut self, result: RustAnalysisResult, file_path: &Path) {
        let mut file_functions = Vec::new();
        
        for snippet in result.snippets {
            if let RustSnippetType::Function = snippet.snippet_type {
                let function_info = FunctionInfo {
                    id: Uuid::new_v4(),
                    name: snippet.name.clone(),
                    file_path: file_path.to_path_buf(),
                    line_start: snippet.start_line,
                    line_end: snippet.end_line,
                    namespace: snippet.scope.unwrap_or_default(),
                    language: "rust".to_string(),
                    signature: None, // TODO: 生成函数签名
                    return_type: snippet.return_type,
                    parameters: snippet.parameters.iter().map(|p| {
                        ParameterInfo {
                            name: p.clone(),
                            type_name: None, // TODO: 解析参数类型
                            default_value: None,
                        }
                    }).collect(),
                };
                
                self.function_registry.insert(snippet.name.clone(), function_info.clone());
                file_functions.push(function_info);
            }
        }
        
        self.file_functions.insert(file_path.to_path_buf(), file_functions);
    }
}

// 实现CodeAnalyzer trait的新方法
impl crate::codegraph::analyzers::CodeAnalyzer for RustAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        RustAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        RustAnalyzer::analyze_directory(self, dir.as_path())
    }
    
    fn extract_functions(&self, path: &PathBuf) -> Result<Vec<crate::codegraph::types::FunctionInfo>, String> {
        if let Some(functions) = self.file_functions.get(path) {
            Ok(functions.clone())
        } else {
            Ok(Vec::new())
        }
    }
    
    fn extract_classes(&self, _path: &PathBuf) -> Result<Vec<crate::codegraph::types::ClassInfo>, String> {
        // Rust doesn't have traditional classes, but we can extract structs and enums
        // For now, return empty vector
        Ok(Vec::new())
    }
    
    fn extract_call_relations(&self, path: &PathBuf) -> Result<Vec<crate::codegraph::types::CallRelation>, String> {
        // Extract call relations from the analysis result
        // For now, return empty vector
        Ok(Vec::new())
    }
} 