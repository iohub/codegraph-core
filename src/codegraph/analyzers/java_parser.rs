use std::path::PathBuf;
use tree_sitter::{Parser, Tree, Node, QueryCursor, Language, StreamingIterator};
use tracing::{error, debug, info};

use crate::codegraph::analyzers::{ParserError, AstLanguageParser};
use crate::codegraph::treesitter::ast_instance_structs::AstSymbolInstanceArc;
use crate::codegraph::treesitter::queries::java::{JavaQueries, JavaSnippet, JavaSnippetType};

extern "C" { fn tree_sitter_java() -> Language; }

pub struct JavaParser {
    queries: JavaQueries,
}

impl JavaParser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let language = unsafe { tree_sitter_java() };
        let queries = JavaQueries::new(&language)?;
        Ok(Self { queries })
    }

    fn parse_java_code(&self, code: &str) -> Result<Tree, ParserError> {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_java() };
        parser.set_language(&language)
            .map_err(|e| ParserError { message: format!("Failed to set Java language: {}", e) })?;
        
        parser.parse(code, None)
            .ok_or_else(|| ParserError { message: "Failed to parse Java code".to_string() })
    }

    fn extract_snippets(&self, tree: &Tree, code: &str, path: &str) -> Vec<JavaSnippet> {
        let mut snippets = Vec::new();
        let root_node = tree.root_node();
        
        // 提取包信息
        let package_name = self.extract_package_name(&root_node, code);
        
        // 提取方法定义
        self.extract_method_definitions(&root_node, code, path, &package_name, &mut snippets);
        
        // 提取类定义
        self.extract_class_definitions(&root_node, code, path, &package_name, &mut snippets);
        
        // 提取接口定义
        self.extract_interface_definitions(&root_node, code, path, &package_name, &mut snippets);
        
        // 提取构造函数定义
        self.extract_constructor_definitions(&root_node, code, path, &package_name, &mut snippets);
        
        // 提取变量声明
        self.extract_variable_declarations(&root_node, code, path, &package_name, &mut snippets);
        
        // 提取枚举定义
        self.extract_enum_definitions(&root_node, code, path, &package_name, &mut snippets);
        
        // 提取注解
        self.extract_annotations(&root_node, code, path, &package_name, &mut snippets);
        
        snippets
    }

    fn extract_package_name(&self, root: &Node, code: &str) -> Option<String> {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.queries.package_declaration, *root, code.as_bytes());
        
        while let Some(m) = matches.next() {
            for capture in m.captures {
                let node = capture.node;
                let capture_name = &self.queries.package_declaration.capture_names()[capture.index as usize];
                
                if *capture_name == "package.name" {
                    let package_name = node.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                    if !package_name.is_empty() {
                        return Some(package_name);
                    }
                }
            }
        }
        None
    }

    fn extract_method_definitions(&self, root: &Node, code: &str, path: &str, package_name: &Option<String>, snippets: &mut Vec<JavaSnippet>) {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.queries.method_definition, *root, code.as_bytes());
        
        while let Some(m) = matches.next() {
            let mut name = String::new();
            let mut params = Vec::new();
            let mut return_type = None;
            let mut modifiers = Vec::new();
            let mut class_name = None;
            
            // 查找父类名
            if let Some(parent) = self.find_parent_class(root, 0, 0) {
                class_name = Some(parent);
            }
            
            for capture in m.captures {
                let node = capture.node;
                let capture_name = &self.queries.method_definition.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "method.name" => {
                        name = node.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                    }
                    "method.params" => {
                        params = self.extract_parameters(&node, code);
                    }
                    "method.body" => {
                        // 方法体内容
                    }
                    _ => {}
                }
            }
            
            if !name.is_empty() {
                let snippet = self.create_java_snippet(
                    JavaSnippetType::Method,
                    name,
                    code[0..code.len()].to_string(), // 简化处理
                    path,
                    package_name,
                    class_name,
                    params,
                    return_type,
                    modifiers,
                );
                snippets.push(snippet);
            }
        }
    }

    fn extract_class_definitions(&self, root: &Node, code: &str, path: &str, package_name: &Option<String>, snippets: &mut Vec<JavaSnippet>) {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.queries.class_definition, *root, code.as_bytes());
        
        while let Some(m) = matches.next() {
            let mut name = String::new();
            let mut modifiers: Vec<String> = Vec::new();
            
            for capture in m.captures {
                let node = capture.node;
                let capture_name = &self.queries.class_definition.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "class.name" => {
                        name = node.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                    }
                    "class.body" => {
                        // 类体内容
                    }
                    _ => {}
                }
            }
            
            if !name.is_empty() {
                let snippet = self.create_java_snippet(
                    JavaSnippetType::Class,
                    name,
                    code[0..code.len()].to_string(), // 简化处理
                    path,
                    package_name,
                    None,
                    Vec::new(),
                    None,
                    Vec::new(),
                );
                snippets.push(snippet);
            }
        }
    }

    fn extract_interface_definitions(&self, root: &Node, code: &str, path: &str, package_name: &Option<String>, snippets: &mut Vec<JavaSnippet>) {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.queries.interface_definition, *root, code.as_bytes());
        
        while let Some(m) = matches.next() {
            let mut name = String::new();
            
            for capture in m.captures {
                let node = capture.node;
                let capture_name = &self.queries.interface_definition.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "interface.name" => {
                        name = node.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                    }
                    _ => {}
                }
            }
            
            if !name.is_empty() {
                let snippet = self.create_java_snippet(
                    JavaSnippetType::Interface,
                    name,
                    code[0..code.len()].to_string(), // 简化处理
                    path,
                    package_name,
                    None,
                    Vec::new(),
                    None,
                    Vec::new(),
                );
                snippets.push(snippet);
            }
        }
    }

    fn extract_constructor_definitions(&self, root: &Node, code: &str, path: &str, package_name: &Option<String>, snippets: &mut Vec<JavaSnippet>) {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.queries.constructor_definition, *root, code.as_bytes());
        
        while let Some(m) = matches.next() {
            let mut name = String::new();
            let mut params = Vec::new();
            let mut class_name = None;
            
            // 查找父类名
            if let Some(parent) = self.find_parent_class(root, 0, 0) {
                class_name = Some(parent.clone());
                name = parent;
            }
            
            for capture in m.captures {
                let node = capture.node;
                let capture_name = &self.queries.constructor_definition.capture_names()[capture.index as usize];
                
                match *capture_name {
                    "constructor.params" => {
                        params = self.extract_parameters(&node, code);
                    }
                    _ => {}
                }
            }
            
            if !name.is_empty() {
                let snippet = self.create_java_snippet(
                    JavaSnippetType::Constructor,
                    name,
                    code[0..code.len()].to_string(), // 简化处理
                    path,
                    package_name,
                    class_name,
                    params,
                    None,
                    Vec::new(),
                );
                snippets.push(snippet);
            }
        }
    }

    fn extract_variable_declarations(&self, root: &Node, code: &str, path: &str, package_name: &Option<String>, snippets: &mut Vec<JavaSnippet>) {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.queries.variable_declaration, *root, code.as_bytes());
        
        while let Some(m) = matches.next() {
            let mut var_name = String::new();
            let mut class_name = None;
            
            // 查找父类名
            if let Some(parent) = self.find_parent_class(root, 0, 0) {
                class_name = Some(parent);
            }
            
            for capture in m.captures {
                let node = capture.node;
                let capture_name = &self.queries.variable_declaration.capture_names()[capture.index as usize];
                
                if *capture_name == "variable.name" {
                    var_name = node.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                }
            }
            
            if !var_name.is_empty() {
                let snippet = self.create_java_snippet(
                    JavaSnippetType::Variable,
                    var_name,
                    code[0..code.len()].to_string(), // 简化处理
                    path,
                    package_name,
                    class_name,
                    Vec::<String>::new(),
                    None,
                    Vec::new(),
                );
                snippets.push(snippet);
            }
        }
    }

    fn extract_enum_definitions(&self, root: &Node, code: &str, path: &str, package_name: &Option<String>, snippets: &mut Vec<JavaSnippet>) {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.queries.enum_definition, *root, code.as_bytes());
        
        while let Some(m) = matches.next() {
            let mut enum_name = String::new();
            
            for capture in m.captures {
                let node = capture.node;
                let capture_name = &self.queries.enum_definition.capture_names()[capture.index as usize];
                
                if *capture_name == "enum.name" {
                    enum_name = node.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                }
            }
            
            if !enum_name.is_empty() {
                let snippet = self.create_java_snippet(
                    JavaSnippetType::Enum,
                    enum_name.clone(),
                    code[0..code.len()].to_string(), // 简化处理
                    path,
                    package_name,
                    Some(enum_name),
                    Vec::<String>::new(),
                    None,
                    Vec::new(),
                );
                snippets.push(snippet);
            }
        }
    }

    fn extract_annotations(&self, root: &Node, code: &str, path: &str, package_name: &Option<String>, snippets: &mut Vec<JavaSnippet>) {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.queries.annotation, *root, code.as_bytes());
        
        while let Some(m) = matches.next() {
            let mut annotation_name = String::new();
            
            for capture in m.captures {
                let node = capture.node;
                let capture_name = &self.queries.annotation.capture_names()[capture.index as usize];
                
                if *capture_name == "annotation.name" {
                    annotation_name = node.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                }
            }
            
            if !annotation_name.is_empty() {
                let snippet = self.create_java_snippet(
                    JavaSnippetType::Annotation,
                    annotation_name,
                    code[0..code.len()].to_string(), // 简化处理
                    path,
                    package_name,
                    None,
                    Vec::<String>::new(),
                    None,
                    Vec::new(),
                );
                snippets.push(snippet);
            }
        }
    }

    fn find_parent_class(&self, _root: &Node, _line: usize, _column: usize) -> Option<String> {
        // 简化的父类查找逻辑
        None
    }

    fn extract_parameters(&self, params_node: &Node, code: &str) -> Vec<String> {
        let mut params = Vec::new();
        
        for i in 0..params_node.child_count() {
            if let Some(child) = params_node.child(i) {
                if child.kind() == "formal_parameter" {
                    for j in 0..child.child_count() {
                        if let Some(param_child) = child.child(j) {
                            if param_child.kind() == "identifier" {
                                let param_name = param_child.utf8_text(code.as_bytes()).unwrap_or("").to_string();
                                if !param_name.is_empty() {
                                    params.push(param_name);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        params
    }

    /// 创建带有默认值的JavaSnippet实例
    fn create_java_snippet(
        &self,
        snippet_type: JavaSnippetType,
        name: String,
        content: String,
        path: &str,
        package_name: &Option<String>,
        class_name: Option<String>,
        parameters: Vec<String>,
        return_type: Option<String>,
        modifiers: Vec<String>,
    ) -> JavaSnippet {
        JavaSnippet {
            snippet_type,
            name,
            content,
            start_line: 0,
            end_line: 0,
            start_column: 0,
            end_column: 0,
            file_path: path.to_string(),
            package_name: package_name.clone(),
            class_name,
            parameters,
            return_type,
            modifiers,
            type_parameters: None,
            superclass: None,
            interfaces: None,
            generic_arguments: None,
            bounds: None,
            exception_types: None,
            loop_type: None,
            condition_type: None,
        }
    }

    fn convert_to_ast_symbols(&self, _snippets: Vec<JavaSnippet>, _path: &PathBuf) -> Vec<AstSymbolInstanceArc> {
        // 将JavaSnippet转换为AstSymbolInstanceArc
        // 这里需要根据实际的AstSymbolInstanceArc结构进行转换
        let symbols = Vec::new();
        
        // TODO: 实现具体的转换逻辑
        // 这里需要根据AstSymbolInstanceArc的实际结构来创建对象
        
        symbols
    }
}

impl AstLanguageParser for JavaParser {
    fn parse(&mut self, code: &str, path: &PathBuf) -> Vec<AstSymbolInstanceArc> {
        info!("Parsing Java file: {:?}", path);
        
        match self.parse_java_code(code) {
            Ok(tree) => {
                let path_str = path.to_string_lossy();
                
                // 提取代码片段
                let snippets = self.extract_snippets(&tree, code, &path_str);
                info!("Extracted {} Java snippets from file: {}", snippets.len(), path_str);
                
                // 转换为AstSymbolInstanceArc
                let symbols = self.convert_to_ast_symbols(snippets, path);
                info!("Converted to {} AST symbols", symbols.len());
                
                symbols
            }
            Err(e) => {
                error!("Failed to parse Java code in file {:?}: {}", path, e.message);
                Vec::new()
            }
        }
    }
} 