use std::collections::HashMap;
use tree_sitter::{Language, Query, QueryError};

/// C++查询集合
pub struct CppQueries {
    pub function_definition: Query,
    pub function_call: Query,
    pub class_definition: Query,
    pub namespace: Query,
    pub include: Query,
}

impl CppQueries {
    pub fn new(language: &Language) -> Result<Self, QueryError> {
        // 函数定义查询 - 只使用基本的函数定义
        let function_definition = Query::new(
            language,
            r#"
            ; 函数定义
            (function_definition
              declarator: (function_declarator
                declarator: (_) @function.name
                parameters: (parameter_list) @function.params
              )
              body: (compound_statement) @function.body
            ) @function.def
            "#,
        )?;

        // 函数调用查询
        let function_call = Query::new(
            language,
            r#"
            ; 函数调用
            (call_expression
              function: (identifier) @function.called
              arguments: (argument_list) @function.args
            ) @function.call
            "#,
        )?;

        // 类定义查询
        let class_definition = Query::new(
            language,
            r#"
            ; 类定义
            (class_specifier
              name: (type_identifier) @class.name
            ) @class.def

            ; 结构体定义
            (struct_specifier
              name: (type_identifier) @class.name
            ) @class.def

            ; 联合体定义
            (union_specifier
              name: (type_identifier) @class.name
            ) @class.def
            "#,
        )?;

        // 命名空间查询
        let namespace = Query::new(
            language,
            r#"
            ; 命名空间定义
            (namespace_definition
              name: (namespace_identifier) @namespace.name
            ) @namespace.def
            "#,
        )?;

        // 包含文件查询
        let include = Query::new(
            language,
            r#"
            ; 包含文件
            (preproc_include
              path: (_) @include.path
            ) @include.stmt
            "#,
        )?;

        Ok(Self {
            function_definition,
            function_call,
            class_definition,
            namespace,
            include,
        })
    }
}

/// C++代码片段类型
#[derive(Debug, Clone, PartialEq)]
pub enum CppSnippetType {
    Function,
    Class,
    Namespace,
    Variable,
    Include,
    Method,
    Constructor,
    Destructor,
}

/// C++代码片段信息
#[derive(Debug, Clone)]
pub struct CppSnippet {
    pub snippet_type: CppSnippetType,
    pub name: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub file_path: String,
    pub namespace: Option<String>,
    pub class_name: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
}

/// C++函数调用信息
#[derive(Debug, Clone)]
pub struct CppFunctionCall {
    pub caller_name: String,
    pub called_name: String,
    pub caller_location: (usize, usize), // (line, column)
    pub called_location: (usize, usize),
    pub caller_file: String,
    pub called_file: Option<String>,
    pub is_resolved: bool,
    pub namespace: Option<String>,
    pub class_name: Option<String>,
}

/// C++作用域信息
#[derive(Debug, Clone)]
pub struct CppScope {
    pub name: String,
    pub scope_type: CppSnippetType,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub parent_scope: Option<String>,
    pub namespace: Option<String>,
    pub class_name: Option<String>,
}

/// C++代码分析结果
#[derive(Debug, Clone)]
pub struct CppAnalysisResult {
    pub snippets: Vec<CppSnippet>,
    pub function_calls: Vec<CppFunctionCall>,
    pub scopes: Vec<CppScope>,
    pub includes: Vec<String>,
    pub namespaces: HashMap<String, Vec<String>>,
    pub classes: HashMap<String, Vec<String>>,
}
