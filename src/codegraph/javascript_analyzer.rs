use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tree_sitter::{Parser, Tree, Node, Language};
use crate::codegraph::treesitter::queries::javascript::{
    JavaScriptQueries, JavaScriptSnippet, JavaScriptSnippetType, 
    JavaScriptFunctionCall, JavaScriptScope, JavaScriptAnalysisResult
};
use crate::codegraph::types::{FunctionInfo, ClassInfo, EntityNode, EntityEdge, EntityEdgeType, EntityGraph};

extern "C" { fn tree_sitter_javascript() -> Language; }

/// JavaScript代码分析器
pub struct JavaScriptAnalyzer {
    queries: JavaScriptQueries,
    parser: Parser,
    language: Language,
    snippets: Vec<JavaScriptSnippet>,
    function_calls: Vec<JavaScriptFunctionCall>,
    scopes: Vec<JavaScriptScope>,
    imports: Vec<String>,
    exports: Vec<String>,
    modules: HashMap<String, Vec<String>>,
    classes: HashMap<String, Vec<String>>,
    objects: HashMap<String, Vec<String>>,
}

impl JavaScriptAnalyzer {
    /// 创建新的JavaScript分析器
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut parser = Parser::new();
        let js_language = unsafe { tree_sitter_javascript() };
        parser.set_language(&js_language)?;
        
        let queries = JavaScriptQueries::new(&js_language)?;
        
        Ok(Self {
            queries,
            parser,
            language: js_language,
            snippets: Vec::new(),
            function_calls: Vec::new(),
            scopes: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            modules: HashMap::new(),
            classes: HashMap::new(),
            objects: HashMap::new(),
        })
    }

    /// 分析单个JavaScript文件
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let tree = self.parser.parse(&content, None)
            .ok_or("Failed to parse JavaScript file")?;
        
        self.analyze_tree(&tree, &content, file_path)?;
        Ok(())
    }

    /// 分析目录中的所有JavaScript文件
    pub fn analyze_directory(&mut self, dir_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.analyze_directory_recursive(dir_path)?;
        Ok(())
    }

    /// 递归分析目录
    fn analyze_directory_recursive(&mut self, dir_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if dir_path.is_file() {
            if let Some(ext) = dir_path.extension() {
                if ext == "js" || ext == "jsx" || ext == "mjs" {
                    self.analyze_file(dir_path)?;
                }
            }
            return Ok(());
        }

        if dir_path.is_dir() {
            for entry in fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                
                // 跳过node_modules和.git目录
                if path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name == "node_modules" || name == ".git")
                    .unwrap_or(false) {
                    continue;
                }
                
                self.analyze_directory_recursive(&path)?;
            }
        }
        
        Ok(())
    }

    /// 分析语法树
    fn analyze_tree(&mut self, tree: &Tree, content: &str, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let root_node = tree.root_node();
        let file_path_str = file_path.to_string_lossy().to_string();
        
        // 分析函数定义
        self.analyze_functions(&root_node, content, &file_path_str)?;
        
        // 分析类定义
        self.analyze_classes(&root_node, content, &file_path_str)?;
        
        // 分析对象定义
        self.analyze_objects(&root_node, content, &file_path_str)?;
        
        // 分析导入导出
        self.analyze_imports_exports(&root_node, content, &file_path_str)?;
        
        // 分析函数调用
        self.analyze_function_calls(&root_node, content, &file_path_str)?;
        
        // 分析作用域
        self.analyze_scopes(&root_node, content, &file_path_str)?;
        
        Ok(())
    }

    /// 分析函数定义
    fn analyze_functions(&mut self, node: &Node, content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor = node.walk();
        
        loop {
            let node = cursor.node();
            
            // 函数声明
            if node.kind() == "function_declaration" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(content.as_bytes())?.to_string();
                    let snippet = self.create_function_snippet(&node, content, file_path, &name, JavaScriptSnippetType::Function)?;
                    self.snippets.push(snippet);
                }
            }
            
            // 箭头函数
            if node.kind() == "arrow_function" {
                let name = self.extract_arrow_function_name(&node, content)?;
                let snippet = self.create_function_snippet(&node, content, file_path, &name, JavaScriptSnippetType::ArrowFunction)?;
                self.snippets.push(snippet);
            }
            
            // 函数表达式
            if node.kind() == "function" {
                let name = self.extract_function_expression_name(&node, content)?;
                let snippet = self.create_function_snippet(&node, content, file_path, &name, JavaScriptSnippetType::FunctionExpression)?;
                self.snippets.push(snippet);
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        
        Ok(())
    }

    /// 分析类定义
    fn analyze_classes(&mut self, node: &Node, content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor = node.walk();
        
        loop {
            let node = cursor.node();
            
            if node.kind() == "class_declaration" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(content.as_bytes())?.to_string();
                    let snippet = self.create_class_snippet(&node, content, file_path, &name)?;
                    self.snippets.push(snippet);
                    
                    // 分析类方法
                    self.analyze_class_methods(&node, content, file_path, &name)?;
                }
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        
        Ok(())
    }

    /// 分析对象定义
    fn analyze_objects(&mut self, node: &Node, content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor = node.walk();
        
        loop {
            let node = cursor.node();
            
            if node.kind() == "object" {
                let name = self.extract_object_name(&node, content)?;
                let snippet = self.create_object_snippet(&node, content, file_path, &name)?;
                self.snippets.push(snippet);
                
                // 分析对象方法
                self.analyze_object_methods(&node, content, file_path, &name)?;
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        
        Ok(())
    }

    /// 分析导入导出
    fn analyze_imports_exports(&mut self, node: &Node, content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor = node.walk();
        
        loop {
            let node = cursor.node();
            
            if node.kind() == "import_statement" {
                let import_info = self.extract_import_info(&node, content)?;
                self.imports.push(import_info);
            }
            
            if node.kind() == "export_statement" {
                let export_info = self.extract_export_info(&node, content)?;
                self.exports.push(export_info);
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        
        Ok(())
    }

    /// 分析函数调用
    fn analyze_function_calls(&mut self, node: &Node, content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor = node.walk();
        
        loop {
            let node = cursor.node();
            
            if node.kind() == "call_expression" {
                let function_call = self.extract_function_call(&node, content, file_path)?;
                self.function_calls.push(function_call);
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        
        Ok(())
    }

    /// 分析作用域
    fn analyze_scopes(&mut self, node: &Node, content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor = node.walk();
        
        loop {
            let node = cursor.node();
            
            if node.kind() == "program" {
                let scope = self.create_scope(&node, content, file_path, "global", JavaScriptSnippetType::Module)?;
                self.scopes.push(scope);
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        
        Ok(())
    }

    /// 创建函数片段
    fn create_function_snippet(
        &self, 
        node: &Node, 
        content: &str, 
        file_path: &str, 
        name: &str, 
        snippet_type: JavaScriptSnippetType
    ) -> Result<JavaScriptSnippet, Box<dyn std::error::Error>> {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        let start_column = node.start_position().column;
        let end_column = node.end_position().column;
        
        let content_text = &content[node.start_byte()..node.end_byte()];
        let parameters = self.extract_parameters(node, content)?;
        
        Ok(JavaScriptSnippet {
            snippet_type,
            name: name.to_string(),
            content: content_text.to_string(),
            start_line,
            end_line,
            start_column,
            end_column,
            file_path: file_path.to_string(),
            module_name: None,
            class_name: None,
            object_name: None,
            parameters,
            return_type: None,
            decorators: Vec::new(),
            extends: Vec::new(),
            implements: Vec::new(),
            is_async: false,
            is_generator: false,
        })
    }

    /// 创建类片段
    fn create_class_snippet(
        &self, 
        node: &Node, 
        content: &str, 
        file_path: &str, 
        name: &str
    ) -> Result<JavaScriptSnippet, Box<dyn std::error::Error>> {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        let start_column = node.start_position().column;
        let end_column = node.end_position().column;
        
        let content_text = &content[node.start_byte()..node.end_byte()];
        let extends = self.extract_extends(node, content)?;
        
        Ok(JavaScriptSnippet {
            snippet_type: JavaScriptSnippetType::Class,
            name: name.to_string(),
            content: content_text.to_string(),
            start_line,
            end_line,
            start_column,
            end_column,
            file_path: file_path.to_string(),
            module_name: None,
            class_name: Some(name.to_string()),
            object_name: None,
            parameters: Vec::new(),
            return_type: None,
            decorators: Vec::new(),
            extends,
            implements: Vec::new(),
            is_async: false,
            is_generator: false,
        })
    }

    /// 创建对象片段
    fn create_object_snippet(
        &self, 
        node: &Node, 
        content: &str, 
        file_path: &str, 
        name: &str
    ) -> Result<JavaScriptSnippet, Box<dyn std::error::Error>> {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        let start_column = node.start_position().column;
        let end_column = node.end_position().column;
        
        let content_text = &content[node.start_byte()..node.end_byte()];
        
        Ok(JavaScriptSnippet {
            snippet_type: JavaScriptSnippetType::Object,
            name: name.to_string(),
            content: content_text.to_string(),
            start_line,
            end_line,
            start_column,
            end_column,
            file_path: file_path.to_string(),
            module_name: None,
            class_name: None,
            object_name: Some(name.to_string()),
            parameters: Vec::new(),
            return_type: None,
            decorators: Vec::new(),
            extends: Vec::new(),
            implements: Vec::new(),
            is_async: false,
            is_generator: false,
        })
    }

    /// 创建作用域
    fn create_scope(
        &self, 
        node: &Node, 
        content: &str, 
        file_path: &str, 
        name: &str, 
        scope_type: JavaScriptSnippetType
    ) -> Result<JavaScriptScope, Box<dyn std::error::Error>> {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        let start_column = node.start_position().column;
        let end_column = node.end_position().column;
        
        Ok(JavaScriptScope {
            name: name.to_string(),
            scope_type,
            start_line,
            end_line,
            start_column,
            end_column,
            parent_scope: None,
            module_name: None,
            class_name: None,
            object_name: None,
        })
    }

    /// 提取参数
    fn extract_parameters(&self, node: &Node, content: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut parameters = Vec::new();
        
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            
            loop {
                let param_node = cursor.node();
                
                if param_node.kind() == "identifier" {
                    let param_name = param_node.utf8_text(content.as_bytes())?;
                    parameters.push(param_name.to_string());
                }
                
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        Ok(parameters)
    }

    /// 提取继承信息
    fn extract_extends(&self, node: &Node, content: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut extends = Vec::new();
        
        if let Some(heritage_node) = node.child_by_field_name("heritage") {
            let mut cursor = heritage_node.walk();
            
            loop {
                let heritage_item = cursor.node();
                
                if heritage_item.kind() == "extends_clause" {
                    if let Some(type_ref) = heritage_item.child_by_field_name("type") {
                        let type_name = type_ref.utf8_text(content.as_bytes())?;
                        extends.push(type_name.to_string());
                    }
                }
                
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        Ok(extends)
    }

    /// 提取箭头函数名称
    fn extract_arrow_function_name(&self, node: &Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 尝试从父节点获取名称
        if let Some(parent) = node.parent() {
            if parent.kind() == "variable_declarator" {
                if let Some(name_node) = parent.child_by_field_name("name") {
                    return Ok(name_node.utf8_text(content.as_bytes())?.to_string());
                }
            }
        }
        
        Ok("anonymous_arrow_function".to_string())
    }

    /// 提取函数表达式名称
    fn extract_function_expression_name(&self, node: &Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 尝试从父节点获取名称
        if let Some(parent) = node.parent() {
            if parent.kind() == "variable_declarator" {
                if let Some(name_node) = parent.child_by_field_name("name") {
                    return Ok(name_node.utf8_text(content.as_bytes())?.to_string());
                }
            }
        }
        
        Ok("anonymous_function".to_string())
    }

    /// 提取对象名称
    fn extract_object_name(&self, node: &Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 尝试从父节点获取名称
        if let Some(parent) = node.parent() {
            if parent.kind() == "variable_declarator" {
                if let Some(name_node) = parent.child_by_field_name("name") {
                    return Ok(name_node.utf8_text(content.as_bytes())?.to_string());
                }
            }
        }
        
        Ok("anonymous_object".to_string())
    }

    /// 提取导入信息
    fn extract_import_info(&self, node: &Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut import_info = String::new();
        
        if let Some(source_node) = node.child_by_field_name("source") {
            let source = source_node.utf8_text(content.as_bytes())?;
            import_info.push_str(&format!("import from {}", source));
        }
        
        Ok(import_info)
    }

    /// 提取导出信息
    fn extract_export_info(&self, node: &Node, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut export_info = String::new();
        
        if let Some(declaration_node) = node.child_by_field_name("declaration") {
            if let Some(name_node) = declaration_node.child_by_field_name("name") {
                let name = name_node.utf8_text(content.as_bytes())?;
                export_info.push_str(&format!("export {}", name));
            }
        }
        
        Ok(export_info)
    }

    /// 提取函数调用信息
    fn extract_function_call(&self, node: &Node, content: &str, file_path: &str) -> Result<JavaScriptFunctionCall, Box<dyn std::error::Error>> {
        let mut caller_name = String::new();
        let mut called_name = String::new();
        
        if let Some(function_node) = node.child_by_field_name("function") {
            called_name = function_node.utf8_text(content.as_bytes())?.to_string();
        }
        
        if let Some(arguments_node) = node.child_by_field_name("arguments") {
            // 这里可以进一步解析参数
        }
        
        let start_line = node.start_position().row;
        let start_column = node.start_position().column;
        
        Ok(JavaScriptFunctionCall {
            caller_name,
            called_name,
            caller_location: (start_line, start_column),
            called_location: (start_line, start_column),
            caller_file: file_path.to_string(),
            called_file: None,
            is_resolved: false,
            module_name: None,
            class_name: None,
            object_name: None,
            arguments: Vec::new(),
            is_method_call: false,
            is_constructor_call: false,
        })
    }

    /// 分析类方法
    fn analyze_class_methods(&mut self, class_node: &Node, content: &str, file_path: &str, class_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor = class_node.walk();
        
        loop {
            let node = cursor.node();
            
            if node.kind() == "method_definition" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = name_node.utf8_text(content.as_bytes())?.to_string();
                    let snippet = self.create_method_snippet(&node, content, file_path, &name, class_name, JavaScriptSnippetType::ClassMethod)?;
                    self.snippets.push(snippet);
                }
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        
        Ok(())
    }

    /// 分析对象方法
    fn analyze_object_methods(&mut self, object_node: &Node, content: &str, file_path: &str, object_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cursor = object_node.walk();
        
        loop {
            let node = cursor.node();
            
            if node.kind() == "pair" {
                if let Some(value_node) = node.child_by_field_name("value") {
                    if value_node.kind() == "function" {
                        if let Some(key_node) = node.child_by_field_name("key") {
                            let name = key_node.utf8_text(content.as_bytes())?.to_string();
                            let snippet = self.create_method_snippet(&value_node, content, file_path, &name, object_name, JavaScriptSnippetType::ObjectMethod)?;
                            self.snippets.push(snippet);
                        }
                    }
                }
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        
        Ok(())
    }

    /// 创建方法片段
    fn create_method_snippet(
        &self, 
        node: &Node, 
        content: &str, 
        file_path: &str, 
        name: &str, 
        parent_name: &str, 
        snippet_type: JavaScriptSnippetType
    ) -> Result<JavaScriptSnippet, Box<dyn std::error::Error>> {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        let start_column = node.start_position().column;
        let end_column = node.end_position().column;
        
        let content_text = &content[node.start_byte()..node.end_byte()];
        let parameters = self.extract_parameters(node, content)?;
        
        let (class_name, object_name) = match snippet_type {
            JavaScriptSnippetType::ClassMethod => (Some(parent_name.to_string()), None),
            JavaScriptSnippetType::ObjectMethod => (None, Some(parent_name.to_string())),
            _ => (None, None),
        };
        
        Ok(JavaScriptSnippet {
            snippet_type,
            name: name.to_string(),
            content: content_text.to_string(),
            start_line,
            end_line,
            start_column,
            end_column,
            file_path: file_path.to_string(),
            module_name: None,
            class_name,
            object_name,
            parameters,
            return_type: None,
            decorators: Vec::new(),
            extends: Vec::new(),
            implements: Vec::new(),
            is_async: false,
            is_generator: false,
        })
    }

    /// 获取所有函数信息
    pub fn get_all_functions(&self) -> Vec<&JavaScriptSnippet> {
        self.snippets.iter()
            .filter(|s| matches!(s.snippet_type, 
                JavaScriptSnippetType::Function | 
                JavaScriptSnippetType::ArrowFunction | 
                JavaScriptSnippetType::FunctionExpression |
                JavaScriptSnippetType::Method |
                JavaScriptSnippetType::ClassMethod |
                JavaScriptSnippetType::ObjectMethod
            ))
            .collect()
    }

    /// 获取所有类信息
    pub fn get_all_classes(&self) -> Vec<&JavaScriptSnippet> {
        self.snippets.iter()
            .filter(|s| s.snippet_type == JavaScriptSnippetType::Class)
            .collect()
    }

    /// 获取所有对象信息
    pub fn get_all_objects(&self) -> Vec<&JavaScriptSnippet> {
        self.snippets.iter()
            .filter(|s| s.snippet_type == JavaScriptSnippetType::Object)
            .collect()
    }

    /// 生成分析报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== JavaScript Code Analysis Report ===\n\n");
        
        report.push_str(&format!("Total Snippets: {}\n", self.snippets.len()));
        report.push_str(&format!("Total Function Calls: {}\n", self.function_calls.len()));
        report.push_str(&format!("Total Scopes: {}\n", self.scopes.len()));
        report.push_str(&format!("Total Imports: {}\n", self.imports.len()));
        report.push_str(&format!("Total Exports: {}\n", self.exports.len()));
        
        report.push_str("\n=== Functions ===\n");
        for function in self.get_all_functions() {
            report.push_str(&format!("  - {} ({}:{}-{})\n", 
                function.name, 
                function.file_path, 
                function.start_line, 
                function.end_line));
        }
        
        report.push_str("\n=== Classes ===\n");
        for class in self.get_all_classes() {
            report.push_str(&format!("  - {} ({}:{}-{})\n", 
                class.name, 
                class.file_path, 
                class.start_line, 
                class.end_line));
        }
        
        report.push_str("\n=== Objects ===\n");
        for object in self.get_all_objects() {
            report.push_str(&format!("  - {} ({}:{}-{})\n", 
                object.name, 
                object.file_path, 
                object.start_line, 
                object.end_line));
        }
        
        report
    }

    /// 获取分析结果
    pub fn get_analysis_result(&self) -> JavaScriptAnalysisResult {
        JavaScriptAnalysisResult {
            snippets: self.snippets.clone(),
            function_calls: self.function_calls.clone(),
            scopes: self.scopes.clone(),
            imports: self.imports.clone(),
            exports: self.exports.clone(),
            modules: self.modules.clone(),
            classes: self.classes.clone(),
            objects: self.objects.clone(),
        }
    }
} 