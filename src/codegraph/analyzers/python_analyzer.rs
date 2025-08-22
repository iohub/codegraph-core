use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tree_sitter::{Parser, Node, QueryCursor, Language, Point, StreamingIterator};
use tracing::{info, warn};
use uuid::Uuid;

use crate::codegraph::types::{FunctionInfo, ParameterInfo};
use crate::codegraph::treesitter::queries::python::{
    PythonQueries, PythonSnippet, PythonSnippetType, PythonFunctionCall, 
    PythonScope, PythonAnalysisResult
};

extern "C" { fn tree_sitter_python() -> Language; }

/// Python函数作用域信息
#[derive(Debug, Clone)]
struct FunctionScope {
    pub name: String,
    pub params: Vec<String>,
    pub body_start: Point,
    pub body_end: Point,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub function_id: Uuid,
    pub decorators: Vec<String>,
}

/// Python函数调用信息
#[derive(Debug, Clone)]
struct FunctionCall {
    pub caller_name: String,
    pub called_name: String,
    pub location: Point,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub arguments: Vec<String>,
    pub keyword_arguments: HashMap<String, String>,
}

/// Python代码分析器
pub struct PythonAnalyzer {
    parser: Parser,
    language: Language,
    queries: PythonQueries,
    /// 函数名 -> 函数信息映射（用于解析调用关系）
    function_registry: HashMap<String, FunctionInfo>,
    /// 文件路径 -> 函数列表映射
    file_functions: HashMap<PathBuf, Vec<FunctionInfo>>,
    /// 所有代码片段
    all_snippets: Vec<PythonSnippet>,
    /// 所有类信息
    all_classes: Vec<PythonSnippet>,
}

impl PythonAnalyzer {
    pub fn new() -> Result<Self, String> {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_python() };
        
        parser.set_language(&language)
            .map_err(|e| format!("Failed to set Python language: {}", e))?;

        let queries = PythonQueries::new(&language)
            .map_err(|e| format!("Failed to create Python queries: {}", e))?;

        Ok(Self {
            parser,
            language,
            queries,
            function_registry: HashMap::new(),
            file_functions: HashMap::new(),
            all_snippets: Vec::new(),
            all_classes: Vec::new(),
        })
    }

    /// 分析目录下的所有Python文件
    pub fn analyze_directory(&mut self, dir: &Path) -> Result<(), String> {
        info!("Starting Python analysis for directory: {}", dir.display());
        
        let files = self.scan_python_files(dir);
        info!("Found {} Python files to analyze", files.len());
        
        for file_path in files {
            if let Err(e) = self.analyze_file(&file_path) {
                warn!("Failed to analyze file {}: {}", file_path.display(), e);
            }
        }
        
        info!("Python analysis completed");
        Ok(())
    }

    /// 分析单个Python文件
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<(), String> {
        info!("Analyzing Python file: {}", file_path.display());
        
        let code = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
        
        let tree = self.parser.parse(&code, None)
            .ok_or_else(|| format!("Failed to parse file {}", file_path.display()))?;
        
        let root_node = tree.root_node();
        let analysis_result = self.analyze_python_code(&code, &root_node, file_path)?;
        
        // 将分析结果转换为FunctionInfo和CallRelation
        self.process_analysis_result(analysis_result, file_path);
        
        Ok(())
    }

    /// 扫描目录下的Python文件
    fn scan_python_files(&self, dir: &Path) -> Vec<PathBuf> {
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
                           name == "__pycache__" || name == "venv" || name == "env" || name == ".venv" {
                            continue;
                        }
                    }
                    self._scan_directory_recursive(&path, files);
                } else if self.is_python_file(&path) {
                    files.push(path);
                }
            }
        }
    }

    /// 判断文件是否为Python文件
    fn is_python_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(),
                "py" | "pyw" | "pyi" | "pyx" | "pxd" | "pyx.in" | "pxd.in"
            )
        } else {
            // 检查文件名是否为常见的Python文件
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                name == "setup.py" || name == "requirements.txt" || name == "Pipfile" || 
                name == "pyproject.toml" || name == "Makefile" || name == "Dockerfile"
            } else {
                false
            }
        }
    }

    /// 分析Python代码，提取函数定义、调用关系等
    fn analyze_python_code(&self, code: &str, root_node: &Node, path: &Path) -> Result<PythonAnalysisResult, String> {
        let mut result = PythonAnalysisResult {
            snippets: Vec::new(),
            function_calls: Vec::new(),
            scopes: Vec::new(),
            imports: Vec::new(),
            modules: HashMap::new(),
            classes: HashMap::new(),
        };

        // 第一遍：收集函数定义和作用域
        let function_scopes = self.collect_function_definitions(code, root_node, path, &mut result)?;
        
        // 第二遍：收集类定义
        self.collect_class_definitions(code, root_node, path, &mut result);
        
        // 第三遍：收集导入语句
        self.collect_imports(code, root_node, &mut result);
        
        // 第四遍：收集所有函数调用
        let all_calls = self.collect_function_calls(code, root_node, path);
        
        // 第五遍：建立调用关系和作用域归属
        self.establish_call_relationships(&function_scopes, &all_calls, &mut result);

        Ok(result)
    }

    /// 收集函数定义和作用域
    fn collect_function_definitions(
        &self,
        code: &str,
        root_node: &Node,
        path: &Path,
        result: &mut PythonAnalysisResult,
    ) -> Result<Vec<FunctionScope>, String> {
        let mut query_cursor = QueryCursor::new();
        let mut function_scopes = Vec::new();
        
        let mut matches = query_cursor.matches(&self.queries.function_definition, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            let mut function_name = String::new();
            let mut parameters = Vec::new();
            let mut body_start = Point::new(0, 0);
            let mut body_end = Point::new(0, 0);
            let mut return_type = None;
            let mut decorators = Vec::new();

            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.function_definition.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "function.name" => {
                        function_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    }
                    "function.params" => {
                        let params_text = node.utf8_text(code.as_bytes()).unwrap();
                        parameters = self.parse_python_parameters(params_text);
                    }
                    "function.body" => {
                        body_start = node.start_position();
                        body_end = node.end_position();
                    }
                    "function.return_type" => {
                        return_type = Some(node.utf8_text(code.as_bytes()).unwrap().to_string());
                    }
                    _ => {}
                }
            }

            // 收集装饰器
            if let Some(parent) = root_node.parent() {
                if parent.kind() == "decorated_definition" {
                    decorators = self.collect_decorators(code, &parent);
                }
            }

            if !function_name.is_empty() {
                let function_id = Uuid::new_v4();
                
                let function_scope = FunctionScope {
                    name: function_name.clone(),
                    params: parameters.clone(),
                    body_start,
                    body_end,
                    module_name: None, // 稍后填充
                    class_name: None, // 稍后填充
                    function_id,
                    decorators: decorators.clone(),
                };
                
                function_scopes.push(function_scope);

                let snippet = PythonSnippet {
                    snippet_type: PythonSnippetType::Function,
                    name: function_name.clone(),
                    content: self.extract_node_text(code, &match_.captures[0].node),
                    start_line: body_start.row,
                    end_line: body_end.row,
                    start_column: body_start.column,
                    end_column: body_end.column,
                    file_path: path.to_string_lossy().to_string(),
                    module_name: None,
                    class_name: None,
                    parameters,
                    return_type,
                    decorators: decorators.clone(),
                };

                result.snippets.push(snippet);

                // 创建作用域
                let scope = PythonScope {
                    name: function_name,
                    scope_type: PythonSnippetType::Function,
                    start_line: body_start.row,
                    end_line: body_end.row,
                    start_column: body_start.column,
                    end_column: body_end.column,
                    parent_scope: None,
                    module_name: None,
                    class_name: None,
                };
                result.scopes.push(scope);
            }
        }
        
        Ok(function_scopes)
    }

    /// 收集类定义
    fn collect_class_definitions(
        &self,
        code: &str,
        root_node: &Node,
        path: &Path,
        result: &mut PythonAnalysisResult,
    ) {
        let mut query_cursor = QueryCursor::new();
        let mut matches = query_cursor.matches(&self.queries.class_definition, *root_node, code.as_bytes());
        
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.class_definition.capture_names()[capture.index as usize];
                
                if *capture_name == "class.name" {
                    let class_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    let snippet = PythonSnippet {
                        snippet_type: PythonSnippetType::Class,
                        name: class_name.clone(),
                        content: self.extract_node_text(code, &node),
                        start_line: node.start_position().row,
                        end_line: node.end_position().row,
                        start_column: node.start_position().column,
                        end_column: node.end_position().column,
                        file_path: path.to_string_lossy().to_string(),
                        module_name: None,
                        class_name: None,
                        parameters: Vec::new(),
                        return_type: None,
                        decorators: Vec::new(),
                    };
                    result.snippets.push(snippet);
                }
            }
        }
    }

    /// 收集导入语句
    fn collect_imports(&self, code: &str, root_node: &Node, result: &mut PythonAnalysisResult) {
        let mut query_cursor = QueryCursor::new();
        
        let mut matches = query_cursor.matches(&self.queries.import_statement, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.import_statement.capture_names()[capture.index as usize];
                
                if *capture_name == "import.module" || *capture_name == "import.name" {
                    let module = node.utf8_text(code.as_bytes()).unwrap();
                    let clean_module = module.trim_matches(|c| c == '"' || c == '\'').to_string();
                    result.imports.push(clean_module);
                }
            }
        }
    }

    /// 收集函数调用
    fn collect_function_calls(&self, code: &str, root_node: &Node, _path: &Path) -> Vec<FunctionCall> {
        let mut query_cursor = QueryCursor::new();
        let mut all_calls = Vec::new();
        
        let mut matches = query_cursor.matches(&self.queries.function_call, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            let mut called_name = String::new();
            let mut arguments = Vec::new();
            let mut keyword_arguments = HashMap::new();
            let mut call_location = Point::new(0, 0);
            
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.function_call.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "function.called" | "method.name" => {
                        called_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                        call_location = node.start_position();
                    }
                    "function.args" | "method.args" => {
                        arguments = self.parse_function_arguments(code, &node);
                    }
                    "kwarg.name" => {
                        let name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                        // 这里需要找到对应的值，简化处理
                        keyword_arguments.insert(name, String::new());
                    }
                    _ => {}
                }
            }

            if !called_name.is_empty() {
                let function_call = FunctionCall {
                    caller_name: String::new(), // 稍后填充
                    called_name,
                    location: call_location,
                    module_name: None,
                    class_name: None,
                    arguments,
                    keyword_arguments,
                };
                
                all_calls.push(function_call);
            }
        }
        
        all_calls
    }

    /// 建立调用关系和作用域归属
    fn establish_call_relationships(
        &self,
        function_scopes: &[FunctionScope],
        all_calls: &[FunctionCall],
        result: &mut PythonAnalysisResult,
    ) {
        let mut function_call_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut global_calls = Vec::new();

        // 为每个函数调用找到其所属的作用域
        for call in all_calls {
            let mut is_in_function = false;
            
            for scope in function_scopes {
                // 检查调用是否在此函数体内
                if call.location.row >= scope.body_start.row &&
                   call.location.row <= scope.body_end.row &&
                   call.location.column >= scope.body_start.column &&
                   call.location.column <= scope.body_end.column {
                    
                    function_call_map
                        .entry(scope.name.clone())
                        .or_insert_with(Vec::new)
                        .push(call.called_name.clone());
                    
                    is_in_function = true;
                    break;
                }
            }
            
            if !is_in_function {
                global_calls.push(call.called_name.clone());
            }
        }

        // 为代码片段设置模块名和类名
        for snippet in &mut result.snippets {
            let snippet_location = (snippet.start_line, snippet.start_column);
            
            // 查找包含此片段的类
            for scope in &result.scopes {
                if scope.scope_type == PythonSnippetType::Class &&
                   snippet_location.0 >= scope.start_line &&
                   snippet_location.0 <= scope.end_line {
                    snippet.class_name = Some(scope.name.clone());
                    break;
                }
            }
        }

        // 创建PythonFunctionCall对象
        for call in all_calls {
            let mut caller_name = String::new();
            
            // 找到包含此调用的函数作用域
            for scope in function_scopes {
                if call.location.row >= scope.body_start.row &&
                   call.location.row <= scope.body_end.row &&
                   call.location.column >= scope.body_start.column &&
                   call.location.column <= scope.body_end.column {
                    caller_name = scope.name.clone();
                    break;
                }
            }

            let function_call = PythonFunctionCall {
                caller_name,
                called_name: call.called_name.clone(),
                caller_location: (call.location.row, call.location.column),
                called_location: (0, 0), // 稍后解析
                caller_file: String::new(), // 稍后填充
                called_file: None,
                is_resolved: false,
                module_name: call.module_name.clone(),
                class_name: call.class_name.clone(),
                arguments: call.arguments.clone(),
                keyword_arguments: call.keyword_arguments.clone(),
            };
            
            result.function_calls.push(function_call);
        }
    }

    /// 处理分析结果，转换为FunctionInfo和CallRelation
    fn process_analysis_result(&mut self, result: PythonAnalysisResult, file_path: &Path) {
        let mut file_functions = Vec::new();
        
        // 存储所有代码片段
        self.all_snippets.extend(result.snippets.clone());
        
        // 转换代码片段为FunctionInfo
        for snippet in result.snippets {
            if snippet.snippet_type == PythonSnippetType::Function {
                let function_info = FunctionInfo {
                    id: Uuid::new_v4(),
                    name: snippet.name.clone(),
                    file_path: file_path.to_path_buf(),
                    line_start: snippet.start_line,
                    line_end: snippet.end_line,
                    namespace: snippet.module_name.unwrap_or_default(),
                    language: "python".to_string(),
                    signature: Some(snippet.content.clone()),
                    return_type: snippet.return_type,
                    parameters: snippet.parameters.iter().map(|p| ParameterInfo {
                        name: p.clone(),
                        type_name: None,
                        default_value: None,
                    }).collect(),
                };
                
                file_functions.push(function_info.clone());
                self.function_registry.insert(snippet.name, function_info);
            } else if snippet.snippet_type == PythonSnippetType::Class {
                self.all_classes.push(snippet);
            }
        }
        
        self.file_functions.insert(file_path.to_path_buf(), file_functions);
    }

    /// 解析Python函数参数
    fn parse_python_parameters(&self, params_text: &str) -> Vec<String> {
        params_text
            .trim_matches(|c| c == '(' || c == ')')
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 解析函数调用参数
    fn parse_function_arguments(&self, code: &str, args_node: &Node) -> Vec<String> {
        let mut arguments = Vec::new();
        let mut cursor = args_node.walk();
        
        for child in args_node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "string" || child.kind() == "number" {
                let arg_text = child.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                if !arg_text.is_empty() {
                    arguments.push(arg_text);
                }
            }
        }
        
        arguments
    }

    /// 收集装饰器
    fn collect_decorators(&self, code: &str, decorated_node: &Node) -> Vec<String> {
        let mut decorators = Vec::new();
        let mut cursor = decorated_node.walk();
        
        for child in decorated_node.children(&mut cursor) {
            if child.kind() == "decorator" {
                let decorator_text = child.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                if !decorator_text.is_empty() {
                    decorators.push(decorator_text);
                }
            }
        }
        
        decorators
    }

    /// 提取节点的文本内容
    fn extract_node_text(&self, code: &str, node: &Node) -> String {
        node.utf8_text(code.as_bytes())
            .unwrap_or("")
            .to_string()
    }

    /// 获取所有函数信息
    pub fn get_all_functions(&self) -> Vec<&FunctionInfo> {
        self.function_registry.values().collect()
    }

    /// 根据函数名查找函数
    pub fn find_functions_by_name(&self, name: &str) -> Vec<&FunctionInfo> {
        self.function_registry.values()
            .filter(|f| f.name == name)
            .collect()
    }

    /// 获取文件的函数列表
    pub fn get_file_functions(&self, file_path: &Path) -> Option<&Vec<FunctionInfo>> {
        self.file_functions.get(file_path)
    }

    /// 生成分析报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Python Code Analysis Report ===\n\n");
        
        // 统计信息
        report.push_str(&format!("Total Snippets: {}\n", self.all_snippets.len()));
        report.push_str(&format!("Total Functions: {}\n", self.function_registry.len()));
        report.push_str(&format!("Total Files: {}\n", self.file_functions.len()));
        
        // 函数列表
        report.push_str("\n=== Functions ===\n");
        for function in self.function_registry.values() {
            report.push_str(&format!("  {} ({}:{}-{})\n", 
                function.name, function.file_path.display(), 
                function.line_start, function.line_end));
        }
        
        // 类列表
        report.push_str("\n=== Classes ===\n");
        for class in &self.all_classes {
            report.push_str(&format!("  {} ({}:{}-{})\n", 
                class.name, class.file_path, 
                class.start_line, class.end_line));
        }
        
        // 文件分布
        report.push_str("\nFunctions by File:\n");
        for (file_path, functions) in &self.file_functions {
            report.push_str(&format!("  {}: {} functions\n", 
                file_path.display(), functions.len()));
        }
        
        report
    }
} 