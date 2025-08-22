use std::collections::HashMap;
use tree_sitter::{Language, Query, QueryError};

/// Java查询集合
pub struct JavaQueries {
    pub method_definition: Query,
    pub method_call: Query,
    pub class_definition: Query,
    pub interface_definition: Query,
    pub package_declaration: Query,
    pub import_declaration: Query,
}

impl JavaQueries {
    pub fn new(language: &Language) -> Result<Self, QueryError> {
        // 方法定义查询
        let method_definition = Query::new(
            language,
            r#"
            ; 方法声明
            (method_declaration
              (_method_header
                type: (_unannotated_type) @method.return_type
                (_method_declarator
                  name: (identifier) @method.name
                  parameters: (formal_parameters) @method.params
                )
              )
              body: (block) @method.body
            ) @method.def
            "#,
        )?;

        // 方法调用查询
        let method_call = Query::new(
            language,
            r#"
            ; 方法调用
            (method_invocation
              (identifier) @method.called
              arguments: (argument_list) @method.args
            ) @method.call

            ; 带对象的方法调用
            (method_invocation
              (primary_expression) @method.object
              (identifier) @method.called
              arguments: (argument_list) @method.args
            ) @method.call
            "#,
        )?;

        // 类定义查询
        let class_definition = Query::new(
            language,
            r#"
            ; 类声明
            (class_declaration
              (identifier) @class.name
              body: (class_body) @class.body
            ) @class.def

            ; 记录声明
            (record_declaration
              (identifier) @class.name
              body: (class_body) @class.body
            ) @class.def
            "#,
        )?;

        // 接口定义查询
        let interface_definition = Query::new(
            language,
            r#"
            ; 接口声明
            (interface_declaration
              (identifier) @interface.name
              body: (interface_body) @interface.body
            ) @interface.def

            ; 注解类型声明
            (annotation_type_declaration
              (identifier) @interface.name
              body: (annotation_type_body) @interface.body
            ) @interface.def
            "#,
        )?;

        // 包声明查询
        let package_declaration = Query::new(
            language,
            r#"
            ; 包声明
            (package_declaration
              (_name) @package.name
            ) @package.decl
            "#,
        )?;

        // 导入声明查询
        let import_declaration = Query::new(
            language,
            r#"
            ; 导入声明
            (import_declaration
              (_name) @import.name
            ) @import.decl
            "#,
        )?;

        Ok(Self {
            method_definition,
            method_call,
            class_definition,
            interface_definition,
            package_declaration,
            import_declaration,
        })
    }
}

/// Java代码片段类型
#[derive(Debug, Clone, PartialEq)]
pub enum JavaSnippetType {
    Method,
    Class,
    Interface,
    Package,
    Import,
}

/// Java代码片段信息
#[derive(Debug, Clone)]
pub struct JavaSnippet {
    pub snippet_type: JavaSnippetType,
    pub name: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub file_path: String,
    pub package_name: Option<String>,
    pub class_name: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
}

/// Java方法调用信息
#[derive(Debug, Clone)]
pub struct JavaMethodCall {
    pub caller_name: String,
    pub called_name: String,
    pub caller_location: (usize, usize), // (line, column)
    pub called_location: (usize, usize),
    pub caller_file: String,
    pub called_file: Option<String>,
    pub is_resolved: bool,
    pub package_name: Option<String>,
    pub class_name: Option<String>,
}

/// Java作用域信息
#[derive(Debug, Clone)]
pub struct JavaScope {
    pub name: String,
    pub scope_type: JavaSnippetType,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub parent_scope: Option<String>,
    pub package_name: Option<String>,
    pub class_name: Option<String>,
}

/// Java代码分析结果
#[derive(Debug, Clone)]
pub struct JavaAnalysisResult {
    pub snippets: Vec<JavaSnippet>,
    pub method_calls: Vec<JavaMethodCall>,
    pub scopes: Vec<JavaScope>,
    pub imports: Vec<String>,
    pub packages: HashMap<String, Vec<String>>,
    pub classes: HashMap<String, Vec<String>>,
    pub interfaces: HashMap<String, Vec<String>>,
} 