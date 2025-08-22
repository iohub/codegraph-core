use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tree_sitter::{Parser, Node, QueryCursor, Language, Point, StreamingIterator};
use tracing::{info, warn};
use uuid::Uuid;

use crate::codegraph::types::{FunctionInfo, ParameterInfo};
use crate::codegraph::treesitter::queries::java::{
    JavaQueries, JavaSnippet, JavaSnippetType, JavaMethodCall, JavaScope, JavaAnalysisResult
};

extern "C" { fn tree_sitter_java() -> Language; }

/// Java方法作用域信息
#[derive(Debug, Clone)]
struct MethodScope {
    pub name: String,
    pub params: Vec<String>,
    pub body_start: Point,
    pub body_end: Point,
    pub package_name: Option<String>,
    pub class_name: Option<String>,
    pub interface_name: Option<String>,
    pub method_id: Uuid,
    pub modifiers: Vec<String>,
    pub type_parameters: Vec<String>,
    pub return_type: Option<String>,
}

/// Java方法调用信息
#[derive(Debug, Clone)]
struct MethodCall {
    pub caller_name: String,
    pub called_name: String,
    pub location: Point,
    pub package_name: Option<String>,
    pub class_name: Option<String>,
    pub interface_name: Option<String>,
    pub arguments: Vec<String>,
    pub type_arguments: Vec<String>,
}

/// Java代码分析器
pub struct JavaAnalyzer {
    parser: Parser,
    language: Language,
    queries: JavaQueries,
    /// 方法名 -> 方法信息映射（用于解析调用关系）
    method_registry: HashMap<String, FunctionInfo>,
    /// 文件路径 -> 方法列表映射
    file_methods: HashMap<PathBuf, Vec<FunctionInfo>>,
}

impl JavaAnalyzer {
    pub fn new() -> Result<Self, String> {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_java() };
        
        parser.set_language(&language)
            .map_err(|e| format!("Failed to set Java language: {}", e))?;

        let queries = JavaQueries::new(&language)
            .map_err(|e| format!("Failed to create Java queries: {}", e))?;

        Ok(Self {
            parser,
            language,
            queries,
            method_registry: HashMap::new(),
            file_methods: HashMap::new(),
        })
    }

    /// 分析目录下的所有Java文件
    pub fn analyze_directory(&mut self, dir: &Path) -> Result<(), String> {
        info!("Starting Java analysis for directory: {}", dir.display());
        
        let files = self.scan_java_files(dir);
        info!("Found {} Java files to analyze", files.len());
        
        for file_path in files {
            if let Err(e) = self.analyze_file(&file_path) {
                warn!("Failed to analyze file {}: {}", file_path.display(), e);
            }
        }
        
        info!("Java analysis completed");
        Ok(())
    }

    /// 分析单个Java文件
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<(), String> {
        info!("Analyzing Java file: {}", file_path.display());
        
        let code = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;
        
        let tree = self.parser.parse(&code, None)
            .ok_or_else(|| format!("Failed to parse file {}", file_path.display()))?;
        
        let root_node = tree.root_node();
        let analysis_result = self.analyze_java_code(&code, &root_node, file_path)?;
        
        // 将分析结果转换为FunctionInfo和CallRelation
        self.process_analysis_result(analysis_result, file_path);
        
        Ok(())
    }

    /// 扫描目录下的Java文件
    fn scan_java_files(&self, dir: &Path) -> Vec<PathBuf> {
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
                } else if self.is_java_file(&path) {
                    files.push(path);
                }
            }
        }
    }

    /// 判断文件是否为Java文件
    fn is_java_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(), "java")
        } else {
            false
        }
    }

    /// 分析Java代码，提取方法定义、调用关系等
    fn analyze_java_code(&self, code: &str, root_node: &Node, path: &Path) -> Result<JavaAnalysisResult, String> {
        let mut result = JavaAnalysisResult {
            snippets: Vec::new(),
            method_calls: Vec::new(),
            scopes: Vec::new(),
            imports: Vec::new(),
            packages: HashMap::new(),
            classes: HashMap::new(),
            interfaces: HashMap::new(),
        };

        // 第一遍：收集包和导入信息
        self.collect_package_and_import_info(code, root_node, path, &mut result);
        
        // 第二遍：收集方法定义和作用域
        let method_scopes = self.collect_method_definitions(code, root_node, path, &mut result)?;
        
        // 第三遍：收集类、接口和类型定义
        self.collect_class_interface_type_definitions(code, root_node, path, &mut result);
        
        // 第四遍：收集所有方法调用
        let all_calls = self.collect_method_calls(code, root_node, path);
        
        // 第五遍：建立调用关系和作用域归属
        self.establish_call_relationships(&method_scopes, &all_calls, &mut result);

        Ok(result)
    }

    /// 收集包和导入信息
    fn collect_package_and_import_info(&self, code: &str, root_node: &Node, path: &Path, result: &mut JavaAnalysisResult) {
        let mut query_cursor = QueryCursor::new();
        
        // 收集包声明
        let mut matches = query_cursor.matches(&self.queries.package_declaration, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.package_declaration.capture_names()[capture.index as usize];
                
                if *capture_name == "package.name" {
                    let package_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    result.packages.insert(package_name.clone(), vec![path.to_string_lossy().to_string()]);
                }
            }
        }

        // 收集导入语句
        let mut matches = query_cursor.matches(&self.queries.import_declaration, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.import_declaration.capture_names()[capture.index as usize];
                
                if *capture_name == "import.name" {
                    let import_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    result.imports.push(import_name);
                }
            }
        }
    }

    /// 收集方法定义和作用域
    fn collect_method_definitions(
        &self,
        code: &str,
        root_node: &Node,
        path: &Path,
        result: &mut JavaAnalysisResult,
    ) -> Result<Vec<MethodScope>, String> {
        let mut query_cursor = QueryCursor::new();
        let mut method_scopes = Vec::new();
        
        info!("Starting method definition collection...");
        let mut matches = query_cursor.matches(&self.queries.method_definition, *root_node, code.as_bytes());
        let mut match_count = 0;
        
        while let Some(match_) = matches.next() {
            match_count += 1;
            info!("Found method declaration match #{}", match_count);
            
            let method_node = if !match_.captures.is_empty() {
                match_.captures[0].node
            } else {
                // 如果没有捕获组，找到实际的method_declaration节点
                let mut method_node = None;
                for capture in match_.captures.iter() {
                    if capture.node.kind() == "method_declaration" {
                        method_node = Some(capture.node);
                        break;
                    }
                }
                method_node.unwrap_or(*root_node)
            };
            
            info!("Method node type: {}", method_node.kind());
            
            if method_node.kind() != "method_declaration" {
                info!("Skipping non-method_declaration node: {}", method_node.kind());
                continue;
            }
            
            let mut method_name = String::new();
            let mut parameters = Vec::new();
            let mut body_start = Point::new(0, 0);
            let mut body_end = Point::new(0, 0);
            let mut return_type = None;
            let mut modifiers = Vec::new();
            let mut type_parameters = Vec::new();
            let mut package_name = None;
            let mut class_name = None;

            // 获取包名
            if let Some(pkg) = result.packages.keys().next() {
                package_name = Some(pkg.clone());
            }

            // 查找父类名
            if let Some(parent) = self.find_parent_class(root_node, method_node.start_position().row, method_node.start_position().column) {
                class_name = Some(parent);
            }

            // 遍历方法声明节点的子节点来找到方法名
            let mut cursor = method_node.walk();
            if cursor.goto_first_child() {
                loop {
                    let node = cursor.node();
                    info!("Child node: {} - '{}'", node.kind(), node.utf8_text(code.as_bytes()).unwrap_or(""));
                    match node.kind() {
                        "identifier" => {
                            // 这是方法名
                            method_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                            info!("Found method name: {}", method_name);
                            break;
                        }
                        _ => {}
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            // 如果找到了方法名，继续提取其他信息
            if !method_name.is_empty() {
                info!("Processing method: {}", method_name);
                
                // 重新遍历来找到参数和方法体
                let mut cursor = method_node.walk();
                if cursor.goto_first_child() {
                    loop {
                        let node = cursor.node();
                        match node.kind() {
                            "formal_parameters" => {
                                // 提取参数信息
                                let params_text = node.utf8_text(code.as_bytes()).unwrap();
                                parameters = self.parse_java_parameters(params_text);
                                info!("Found parameters: {:?}", parameters);
                            }
                            "block" => {
                                // 这是方法体
                                body_start = node.start_position();
                                body_end = node.end_position();
                                info!("Found method body at lines {}-{}", body_start.row, body_end.row);
                            }
                            _ => {}
                        }
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }

                let method_id = Uuid::new_v4();
                
                let method_scope = MethodScope {
                    name: method_name.clone(),
                    params: parameters.clone(),
                    body_start,
                    body_end,
                    package_name: package_name.clone(),
                    class_name: class_name.clone(),
                    interface_name: None,
                    method_id,
                    modifiers: modifiers.clone(),
                    type_parameters: type_parameters.clone(),
                    return_type: return_type.clone(),
                };
                
                method_scopes.push(method_scope);

                let snippet = JavaSnippet {
                    snippet_type: JavaSnippetType::Method,
                    name: method_name.clone(),
                    content: self.extract_node_text(code, &method_node),
                    start_line: body_start.row,
                    end_line: body_end.row,
                    start_column: body_start.column,
                    end_column: body_end.column,
                    file_path: path.to_string_lossy().to_string(),
                    package_name: package_name.clone(),
                    class_name: class_name.clone(),
                    parameters,
                    return_type,
                };

                result.snippets.push(snippet);

                // 创建作用域
                let scope = JavaScope {
                    name: method_name,
                    scope_type: JavaSnippetType::Method,
                    start_line: body_start.row,
                    end_line: body_end.row,
                    start_column: body_start.column,
                    end_column: body_end.column,
                    parent_scope: None,
                    package_name,
                    class_name,
                };
                result.scopes.push(scope);
            } else {
                info!("No method name found in this match");
            }
        }
        
        info!("Total method matches found: {}", match_count);
        info!("Total methods processed: {}", method_scopes.len());
        
        Ok(method_scopes)
    }

    /// 收集类、接口和类型定义
    fn collect_class_interface_type_definitions(
        &self,
        code: &str,
        root_node: &Node,
        path: &Path,
        result: &mut JavaAnalysisResult,
    ) {
        let mut query_cursor = QueryCursor::new();
        
        // 收集类定义
        let mut matches = query_cursor.matches(&self.queries.class_definition, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.class_definition.capture_names()[capture.index as usize];
                
                if *capture_name == "class.name" {
                    let class_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    let package_name = result.packages.keys().next().cloned();
                    
                    let snippet = JavaSnippet {
                        snippet_type: JavaSnippetType::Class,
                        name: class_name.clone(),
                        content: self.extract_node_text(code, &node),
                        start_line: node.start_position().row,
                        end_line: node.end_position().row,
                        start_column: node.start_position().column,
                        end_column: node.end_position().column,
                        file_path: path.to_string_lossy().to_string(),
                        package_name,
                        class_name: Some(class_name.clone()),
                        parameters: Vec::new(),
                        return_type: None,
                    };
                    result.snippets.push(snippet);
                    result.classes.insert(class_name, vec![path.to_string_lossy().to_string()]);
                }
            }
        }

        // 收集接口定义
        let mut matches = query_cursor.matches(&self.queries.interface_definition, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.interface_definition.capture_names()[capture.index as usize];
                
                if *capture_name == "interface.name" {
                    let interface_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                    let package_name = result.packages.keys().next().cloned();
                    
                    let snippet = JavaSnippet {
                        snippet_type: JavaSnippetType::Interface,
                        name: interface_name.clone(),
                        content: self.extract_node_text(code, &node),
                        start_line: node.start_position().row,
                        end_line: node.end_position().row,
                        start_column: node.start_position().column,
                        end_column: node.end_position().column,
                        file_path: path.to_string_lossy().to_string(),
                        package_name,
                        class_name: Some(interface_name.clone()),
                        parameters: Vec::new(),
                        return_type: None,
                    };
                    result.snippets.push(snippet);
                    result.interfaces.insert(interface_name, vec![path.to_string_lossy().to_string()]);
                }
            }
        }


    }

    /// 收集方法调用
    fn collect_method_calls(&self, code: &str, root_node: &Node, _path: &Path) -> Vec<MethodCall> {
        let mut query_cursor = QueryCursor::new();
        let mut all_calls = Vec::new();
        
        let mut matches = query_cursor.matches(&self.queries.method_call, *root_node, code.as_bytes());
        while let Some(match_) = matches.next() {
            let mut called_name = String::new();
            let mut arguments = Vec::new();
            let type_arguments = Vec::new();
            let mut call_location = Point::new(0, 0);
            
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.queries.method_call.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "method.called" => {
                        called_name = node.utf8_text(code.as_bytes()).unwrap().to_string();
                        call_location = node.start_position();
                    }
                    "method.args" => {
                        arguments = self.parse_function_arguments(code, &node);
                    }
                    _ => {}
                }
            }

            if !called_name.is_empty() {
                let method_call = MethodCall {
                    caller_name: String::new(), // 稍后填充
                    called_name,
                    location: call_location,
                    package_name: None,
                    class_name: None,
                    interface_name: None,
                    arguments,
                    type_arguments,
                };
                
                all_calls.push(method_call);
            }
        }
        
        all_calls
    }

    /// 建立调用关系和作用域归属
    fn establish_call_relationships(
        &self,
        method_scopes: &[MethodScope],
        all_calls: &[MethodCall],
        result: &mut JavaAnalysisResult,
    ) {
        let mut method_call_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut global_calls = Vec::new();

        // 为每个方法调用找到其所属的作用域
        for call in all_calls {
            let mut is_in_method = false;
            
            for scope in method_scopes {
                // 检查调用是否在此方法体内
                if call.location.row >= scope.body_start.row &&
                   call.location.row <= scope.body_end.row &&
                   call.location.column >= scope.body_start.column &&
                   call.location.column <= scope.body_end.column {
                    
                    method_call_map
                        .entry(scope.name.clone())
                        .or_insert_with(Vec::new)
                        .push(call.called_name.clone());
                    
                    is_in_method = true;
                    break;
                }
            }
            
            if !is_in_method {
                global_calls.push(call.called_name.clone());
            }
        }

        // 为代码片段设置包名、类名和接口名
        for snippet in &mut result.snippets {
            let snippet_location = (snippet.start_line, snippet.start_column);
            
            // 查找包含此片段的类
            for scope in &result.scopes {
                if scope.scope_type == JavaSnippetType::Class &&
                   snippet_location.0 >= scope.start_line &&
                   snippet_location.0 <= scope.end_line {
                    snippet.class_name = Some(scope.name.clone());
                    break;
                }
            }
            
            // 查找包含此片段的接口
            for scope in &result.scopes {
                if scope.scope_type == JavaSnippetType::Interface &&
                   snippet_location.0 >= scope.start_line &&
                   snippet_location.0 <= scope.end_line {
                    snippet.class_name = Some(scope.name.clone());
                    break;
                }
            }
        }

        // 创建JavaMethodCall对象
        for call in all_calls {
            let mut caller_name = String::new();
            
            // 找到包含此调用的方法作用域
            for scope in method_scopes {
                if call.location.row >= scope.body_start.row &&
                   call.location.row <= scope.body_end.row &&
                   call.location.column >= scope.body_start.column &&
                   call.location.column <= scope.body_end.column {
                    caller_name = scope.name.clone();
                    break;
                }
            }

            let method_call = JavaMethodCall {
                caller_name,
                called_name: call.called_name.clone(),
                caller_location: (call.location.row, call.location.column),
                called_location: (0, 0), // 稍后解析
                caller_file: String::new(), // 稍后填充
                called_file: None,
                is_resolved: false,
                package_name: call.package_name.clone(),
                class_name: call.class_name.clone(),
            };
            
            result.method_calls.push(method_call);
        }
    }

    /// 处理分析结果，转换为FunctionInfo和CallRelation
    fn process_analysis_result(&mut self, result: JavaAnalysisResult, file_path: &Path) {
        let mut file_methods = Vec::new();
        
        // 转换代码片段为FunctionInfo
        for snippet in result.snippets {
            if snippet.snippet_type == JavaSnippetType::Method {
                let function_info = FunctionInfo {
                    id: Uuid::new_v4(),
                    name: snippet.name.clone(),
                    file_path: file_path.to_path_buf(),
                    line_start: snippet.start_line,
                    line_end: snippet.end_line,
                    namespace: snippet.package_name.unwrap_or_default(),
                    language: "java".to_string(),
                    signature: Some(snippet.content.clone()),
                    return_type: snippet.return_type,
                    parameters: snippet.parameters.iter().map(|p| ParameterInfo {
                        name: p.clone(),
                        type_name: None,
                        default_value: None,
                    }).collect(),
                };
                
                file_methods.push(function_info.clone());
                self.method_registry.insert(snippet.name, function_info);
            }
        }
        
        self.file_methods.insert(file_path.to_path_buf(), file_methods);
    }

    /// 解析Java方法参数
    fn parse_java_parameters(&self, params_text: &str) -> Vec<String> {
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

    /// 查找父类
    fn find_parent_class(&self, _root: &Node, _line: usize, _column: usize) -> Option<String> {
        // 简单的父类查找逻辑
        // 这里可以实现更复杂的查找算法
        None
    }

    /// 提取节点的文本内容
    fn extract_node_text(&self, code: &str, node: &Node) -> String {
        node.utf8_text(code.as_bytes())
            .unwrap_or("")
            .to_string()
    }

    /// 获取所有方法信息
    pub fn get_all_methods(&self) -> Vec<&FunctionInfo> {
        self.method_registry.values().collect()
    }

    /// 根据方法名查找方法
    pub fn find_methods_by_name(&self, name: &str) -> Vec<&FunctionInfo> {
        self.method_registry.values()
            .filter(|f| f.name == name)
            .collect()
    }

    /// 获取文件的方法列表
    pub fn get_file_methods(&self, file_path: &Path) -> Option<&Vec<FunctionInfo>> {
        self.file_methods.get(file_path)
    }

    /// 生成分析报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Java Code Analysis Report ===\n\n");
        
        // 统计信息
        report.push_str(&format!("Total Methods: {}\n", self.method_registry.len()));
        report.push_str(&format!("Total Files: {}\n", self.file_methods.len()));
        
        // 文件分布
        report.push_str("\nMethods by File:\n");
        for (file_path, methods) in &self.file_methods {
            report.push_str(&format!("  {}: {} methods\n", 
                file_path.display(), methods.len()));
        }
        
        // 方法列表
        report.push_str("\nMethod List:\n");
        for method in self.method_registry.values() {
            report.push_str(&format!("  {} ({}:{}-{})\n", 
                method.name, method.file_path.display(), 
                method.line_start, method.line_end));
        }
        
        report
    }
} 