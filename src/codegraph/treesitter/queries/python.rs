use std::collections::HashMap;
use tree_sitter::{Query, Language};

/// Python Tree-sitter查询集合
pub struct PythonQueries {
    /// 函数定义查询
    pub function_definition: Query,
    /// 函数调用查询
    pub function_call: Query,
    /// 类定义查询
    pub class_definition: Query,
    /// 导入语句查询
    pub import_statement: Query,
    /// 变量赋值查询
    pub variable_assignment: Query,
    /// 装饰器查询
    pub decorator: Query,
}

impl PythonQueries {
    pub fn new(language: &Language) -> Result<Self, tree_sitter::QueryError> {
        // 函数定义查询
        let function_definition = Query::new(
            language,
            r#"
            (function_definition
              name: (identifier) @function.name
              parameters: (parameters) @function.params
              body: (block) @function.body
            ) @function.def
            "#,
        )?;

        // 函数调用查询
        let function_call = Query::new(
            language,
            r#"
            (call
              function: (identifier) @function.called
              arguments: (argument_list) @function.args
            ) @function.call
            "#,
        )?;

        // 类定义查询
        let class_definition = Query::new(
            language,
            r#"
            (class_definition
              name: (identifier) @class.name
              body: (block) @class.body
            ) @class.def
            "#,
        )?;

        // 导入语句查询
        let import_statement = Query::new(
            language,
            r#"
            (import_statement
              (aliased_import
                name: (dotted_name) @import.module
                alias: (identifier) @import.alias
              )
            ) @import.stmt
            "#,
        )?;

        // 变量赋值查询
        let variable_assignment = Query::new(
            language,
            r#"
            (assignment
              left: (identifier) @variable.name
              right: (_) @variable.value
            ) @variable.assign
            "#,
        )?;

        // 装饰器查询
        let decorator = Query::new(
            language,
            r#"
            (decorator
              (call
                function: (identifier) @decorator.name
                arguments: (argument_list) @decorator.args
              )
            ) @decorator.stmt
            "#,
        )?;

        Ok(Self {
            function_definition,
            function_call,
            class_definition,
            import_statement,
            variable_assignment,
            decorator,
        })
    }
}

/// Python代码片段类型
#[derive(Debug, Clone, PartialEq)]
pub enum PythonSnippetType {
    Function,
    Class,
    Method,
    Module,
    Variable,
    Import,
    Decorator,
}

/// Python代码片段信息
#[derive(Debug, Clone)]
pub struct PythonSnippet {
    pub snippet_type: PythonSnippetType,
    pub name: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub file_path: String,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub decorators: Vec<String>,
}

/// Python函数调用信息
#[derive(Debug, Clone)]
pub struct PythonFunctionCall {
    pub caller_name: String,
    pub called_name: String,
    pub caller_location: (usize, usize), // (line, column)
    pub called_location: (usize, usize),
    pub caller_file: String,
    pub called_file: Option<String>,
    pub is_resolved: bool,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub arguments: Vec<String>,
    pub keyword_arguments: HashMap<String, String>,
}

/// Python作用域信息
#[derive(Debug, Clone)]
pub struct PythonScope {
    pub name: String,
    pub scope_type: PythonSnippetType,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub parent_scope: Option<String>,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
}

/// Python代码分析结果
#[derive(Debug, Clone)]
pub struct PythonAnalysisResult {
    pub snippets: Vec<PythonSnippet>,
    pub function_calls: Vec<PythonFunctionCall>,
    pub scopes: Vec<PythonScope>,
    pub imports: Vec<String>,
    pub modules: HashMap<String, Vec<String>>,
    pub classes: HashMap<String, Vec<String>>,
} 