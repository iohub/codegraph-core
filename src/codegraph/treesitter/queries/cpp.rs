use std::collections::HashMap;
use tree_sitter::{Query, Language};

/// C++ Tree-sitter查询集合
pub struct CppQueries {
    /// 函数定义查询
    pub function_definition: Query,
    /// 函数调用查询
    pub function_call: Query,
    /// 类定义查询
    pub class_definition: Query,
    /// 命名空间查询
    pub namespace: Query,
    /// 包含文件查询
    pub include: Query,
    /// 变量声明查询
    pub variable_declaration: Query,
    /// 方法调用查询
    pub method_call: Query,
    /// 构造函数调用查询
    pub constructor_call: Query,
    /// 析构函数调用查询
    pub destructor_call: Query,
}

impl CppQueries {
    pub fn new(language: &Language) -> Result<Self, tree_sitter::QueryError> {
        // 函数定义查询 - 包括普通函数、成员函数、模板函数
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

            ; 成员函数定义
            (function_definition
              declarator: (function_declarator
                declarator: (field_identifier) @function.name
                parameters: (parameter_list) @function.params
              )
              body: (compound_statement) @function.body
            ) @function.def

            ; 模板函数定义
            (template_declaration
              (function_definition
                declarator: (function_declarator
                  declarator: (_) @function.name
                  parameters: (parameter_list) @function.params
                )
                body: (compound_statement) @function.body
              )
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
            ) @function.call

            ; 成员函数调用
            (call_expression
              function: (field_identifier) @function.called
            ) @function.call

            ; 模板函数调用
            (call_expression
              function: (template_function
                name: (identifier) @function.called
              )
            ) @function.call
            "#,
        )?;

        // 类定义查询
        let class_definition = Query::new(
            language,
            r#"
            ; 类定义
            (class_declaration
              name: (type_identifier) @class.name
              body: (field_declaration_list) @class.body
            ) @class.def

            ; 结构体定义
            (struct_declaration
              name: (type_identifier) @class.name
              body: (field_declaration_list) @class.body
            ) @class.def

            ; 模板类定义
            (template_declaration
              (class_declaration
                name: (type_identifier) @class.name
                body: (field_declaration_list) @class.body
              )
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
              body: (declaration_list) @namespace.body
            ) @namespace.def

            ; 命名空间声明
            (namespace_declaration
              name: (namespace_identifier) @namespace.name
            ) @namespace.decl
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

            ; 系统头文件
            (preproc_include
              path: (system_lib_string) @include.path
            ) @include.stmt

            ; 用户头文件
            (preproc_include
              path: (string_literal) @include.path
            ) @include.stmt
            "#,
        )?;

        // 变量声明查询
        let variable_declaration = Query::new(
            language,
            r#"
            ; 变量声明
            (declaration
              declarator: (init_declarator
                declarator: (identifier) @variable.name
                value: (_) @variable.value
              )
            ) @variable.decl

            ; 成员变量声明
            (field_declaration
              declarator: (field_identifier) @variable.name
            ) @variable.decl
            "#,
        )?;

        // 方法调用查询
        let method_call = Query::new(
            language,
            r#"
            ; 方法调用
            (call_expression
              function: (field_identifier) @method.name
            ) @method.call

            ; 指针方法调用
            (call_expression
              function: (field_identifier) @method.name
            ) @method.call
            "#,
        )?;

        // 构造函数调用查询
        let constructor_call = Query::new(
            language,
            r#"
            ; 构造函数调用
            (call_expression
              function: (type_identifier) @constructor.name
            ) @constructor.call

            ; new表达式
            (new_expression
              type: (type_identifier) @constructor.name
            ) @constructor.call
            "#,
        )?;

        // 析构函数调用查询
        let destructor_call = Query::new(
            language,
            r#"
            ; 析构函数调用
            (call_expression
              function: (destructor_name) @destructor.name
            ) @destructor.call

            ; delete表达式
            (delete_expression
              argument: (_) @destructor.target
            ) @destructor.call
            "#,
        )?;

        Ok(Self {
            function_definition,
            function_call,
            class_definition,
            namespace,
            include,
            variable_declaration,
            method_call,
            constructor_call,
            destructor_call,
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
