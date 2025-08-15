use std::collections::HashMap;
use std::path::PathBuf;
use tree_sitter::{Parser, Node, QueryCursor, Language, Point, StreamingIterator};
use tracing::{info, warn};

use crate::codegraph::treesitter::queries::cpp::{
    CppQueries, CppSnippet, CppSnippetType, CppFunctionCall, 
    CppScope, CppAnalysisResult
};
use crate::codegraph::treesitter::parsers::{ParserError, AstLanguageParser};
use crate::codegraph::treesitter::ast_instance_structs::AstSymbolInstanceArc;

extern "C" { fn tree_sitter_cpp() -> Language; }

/// C++解析器
pub struct CppParser {
    parser: Parser,
    language: Language,
    queries: CppQueries,
}

impl CppParser {
    pub fn new() -> Result<Self, ParserError> {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_cpp() };
        
        parser.set_language(&language)
            .map_err(|e| ParserError {
                message: format!("Failed to set C++ language: {}", e)
            })?;

        let queries = CppQueries::new(&language)
            .map_err(|e| ParserError {
                message: format!("Failed to create C++ queries: {}", e)
            })?;

        Ok(Self {
            parser,
            language,
            queries,
        })
    }

    /// 解析C++文件并提取代码片段和调用关系
    pub fn parse(&mut self, code: &str, path: &PathBuf) -> Vec<AstSymbolInstanceArc> {
        info!("Parsing C++ file: {}", path.display());
        
        let tree = match self.parser.parse(code, None) {
            Some(tree) => tree,
            None => {
                warn!("Failed to parse C++ file: {}", path.display());
                return Vec::new();
            }
        };

        let root_node = tree.root_node();
        let analysis_result = self.analyze_cpp_code(code, &root_node, path);
        
        // 转换为AstSymbolInstanceArc格式
        self.convert_to_ast_symbols(analysis_result, path)
    }

    /// 分析C++代码，提取函数定义、调用关系等
    fn analyze_cpp_code(&self, code: &str, root_node: &Node, path: &PathBuf) -> CppAnalysisResult {
        let mut result = CppAnalysisResult {
            snippets: Vec::new(),
            function_calls: Vec::new(),
            scopes: Vec::new(),
            includes: Vec::new(),
            namespaces: HashMap::new(),
            classes: HashMap::new(),
        };

        // 第一遍：收集函数定义和作用域
        self.collect_function_definitions(code, root_node, path, &mut result);
        
        // 第二遍：收集命名空间和类定义
        self.collect_namespaces_and_classes(code, root_node, path, &mut result);
        
        // 第三遍：收集包含文件
        self.collect_includes(code, root_node, &mut result);
        
        // 第四遍：收集所有函数调用
        self.collect_function_calls(code, root_node, path, &mut result);
        
        // 第五遍：建立调用关系和作用域归属
        self.establish_call_relationships(&mut result);

        result
    }

    /// 收集函数定义
    fn collect_function_definitions(
        &self,
        code: &str,
        root_node: &Node,
        path: &PathBuf,
        result: &mut CppAnalysisResult,
    ) {
        let mut query_cursor = QueryCursor::new();
        
        // 使用正确的Tree-sitter API
        let mut matches = query_cursor.matches(&self.queries.function_definition, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            let mut function_name = String::new();
            let mut parameters = Vec::new();
            let mut body_start = Point::new(0, 0);
            let mut body_end = Point::new(0, 0);
            let mut return_type = None;

            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.function_definition.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "function.name" => {
                        function_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    }
                    "function.params" => {
                        let params_text = node.utf8_text(code.as_bytes()).unwrap();
                        parameters = self.parse_parameters(params_text);
                    }
                    "function.body" => {
                        body_start = node.start_position();
                        body_end = node.end_position();
                    }
                    _ => {}
                }
            }

            if !function_name.is_empty() {
                let snippet = CppSnippet {
                    snippet_type: CppSnippetType::Function,
                    name: function_name.clone(),
                    content: self.extract_node_text(code, &match_.captures[0].node),
                    start_line: body_start.row,
                    end_line: body_end.row,
                    start_column: body_start.column,
                    end_column: body_end.column,
                    file_path: path.to_string_lossy().to_string(),
                    namespace: None, // 稍后填充
                    class_name: None, // 稍后填充
                    parameters,
                    return_type,
                };

                result.snippets.push(snippet);

                // 创建作用域
                let scope = CppScope {
                    name: function_name,
                    scope_type: CppSnippetType::Function,
                    start_line: body_start.row,
                    end_line: body_end.row,
                    start_column: body_start.column,
                    end_column: body_end.column,
                    parent_scope: None,
                    namespace: None,
                    class_name: None,
                };
                result.scopes.push(scope);
            }
        }
    }

    /// 收集命名空间和类定义
    fn collect_namespaces_and_classes(
        &self,
        code: &str,
        root_node: &Node,
        path: &PathBuf,
        result: &mut CppAnalysisResult,
    ) {
        // 收集命名空间
        let mut query_cursor = QueryCursor::new();
        let mut namespace_matches = query_cursor.matches(&self.queries.namespace, *root_node, code.as_bytes());
        while let Some(match_) = namespace_matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.namespace.capture_names()[capture.index as usize];
                
                if *capture_name == "namespace.name" {
                    let namespace_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    let snippet = CppSnippet {
                        snippet_type: CppSnippetType::Namespace,
                        name: namespace_name.clone(),
                        content: self.extract_node_text(code, &node),
                        start_line: node.start_position().row,
                        end_line: node.end_position().row,
                        start_column: node.start_position().column,
                        end_column: node.end_position().column,
                        file_path: path.to_string_lossy().to_string(),
                        namespace: None,
                        class_name: None,
                        parameters: Vec::new(),
                        return_type: None,
                    };
                    result.snippets.push(snippet);
                }
            }
        }

        // 收集类定义
        let mut class_matches = query_cursor.matches(&self.queries.class_definition, *root_node, code.as_bytes());
        while let Some(match_) = class_matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.class_definition.capture_names()[capture.index as usize];
                
                if *capture_name == "class.name" {
                    let class_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    let snippet = CppSnippet {
                        snippet_type: CppSnippetType::Class,
                        name: class_name.clone(),
                        content: self.extract_node_text(code, &node),
                        start_line: node.start_position().row,
                        end_line: node.end_position().row,
                        start_column: node.start_position().column,
                        end_column: node.end_position().column,
                        file_path: path.to_string_lossy().to_string(),
                        namespace: None,
                        class_name: None,
                        parameters: Vec::new(),
                        return_type: None,
                    };
                    result.snippets.push(snippet);
                }
            }
        }
    }

    /// 收集包含文件
    fn collect_includes(&self, code: &str, root_node: &Node, result: &mut CppAnalysisResult) {
        let mut query_cursor = QueryCursor::new();
        
        let mut include_matches = query_cursor.matches(&self.queries.include, *root_node, code.as_bytes());
        while let Some(match_) = include_matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.include.capture_names()[capture.index as usize];
                
                if *capture_name == "include.path" {
                    let path = node.utf8_text(code.as_bytes()).unwrap();
                    let clean_path = path.trim_matches(|c| c == '<' || c == '"').to_string();
                    result.includes.push(clean_path);
                }
            }
        }
    }

    /// 收集函数调用
    fn collect_function_calls(
        &self,
        code: &str,
        root_node: &Node,
        path: &PathBuf,
        result: &mut CppAnalysisResult,
    ) {
        let mut query_cursor = QueryCursor::new();
        
        let mut call_matches = query_cursor.matches(&self.queries.function_call, *root_node, code.as_bytes());
        while let Some(match_) = call_matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.function_call.capture_names()[capture.index as usize];
                
                if *capture_name == "function.called" {
                    let called_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    let call_location = node.start_position();
                    
                    let function_call = CppFunctionCall {
                        caller_name: String::new(), // 稍后填充
                        called_name,
                        caller_location: (call_location.row, call_location.column),
                        called_location: (0, 0), // 稍后解析
                        caller_file: path.to_string_lossy().to_string(),
                        called_file: None,
                        is_resolved: false,
                        namespace: None,
                        class_name: None,
                    };
                    
                    result.function_calls.push(function_call);
                }
            }
        }
    }

    /// 建立调用关系和作用域归属
    fn establish_call_relationships(&self, result: &mut CppAnalysisResult) {
        // 为每个函数调用找到其所属的作用域
        for call in &mut result.function_calls {
            let call_location = call.caller_location;
            
            // 找到包含此调用的函数作用域
            for scope in &result.scopes {
                if scope.scope_type == CppSnippetType::Function &&
                   call_location.0 >= scope.start_line &&
                   call_location.0 <= scope.end_line &&
                   call_location.1 >= scope.start_column &&
                   call_location.1 <= scope.end_column {
                    call.caller_name = scope.name.clone();
                    break;
                }
            }
        }

        // 为代码片段设置命名空间和类名
        for snippet in &mut result.snippets {
            let snippet_location = (snippet.start_line, snippet.start_column);
            
            // 查找包含此片段的命名空间
            for scope in &result.scopes {
                if scope.scope_type == CppSnippetType::Namespace &&
                   snippet_location.0 >= scope.start_line &&
                   snippet_location.0 <= scope.end_line {
                    snippet.namespace = Some(scope.name.clone());
                    break;
                }
            }
            
            // 查找包含此片段的类
            for scope in &result.scopes {
                if scope.scope_type == CppSnippetType::Class &&
                   snippet_location.0 >= scope.start_line &&
                   snippet_location.0 <= scope.end_line {
                    snippet.class_name = Some(scope.name.clone());
                    break;
                }
            }
        }
    }

    /// 解析函数参数
    fn parse_parameters(&self, params_text: &str) -> Vec<String> {
        // 简单的参数解析，可以根据需要扩展
        params_text
            .trim_matches(|c| c == '(' || c == ')')
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 提取节点的文本内容
    fn extract_node_text(&self, code: &str, node: &Node) -> String {
        node.utf8_text(code.as_bytes())
            .unwrap_or("")
            .to_string()
    }

    /// 转换为AstSymbolInstanceArc格式
    fn convert_to_ast_symbols(&self, result: CppAnalysisResult, _path: &PathBuf) -> Vec<AstSymbolInstanceArc> {
        let symbols = Vec::new();
        
        // 转换代码片段为AST符号
        for _snippet in result.snippets {
            // 这里需要根据AstSymbolInstanceArc的具体结构进行转换
            // 由于没有看到AstSymbolInstanceArc的完整定义，这里提供一个基本框架
            // 实际实现时需要根据具体的结构体定义进行调整
        }
        
        symbols
    }
}

impl AstLanguageParser for CppParser {
    fn parse(&mut self, code: &str, path: &PathBuf) -> Vec<AstSymbolInstanceArc> {
        self.parse(code, path)
    }
} 