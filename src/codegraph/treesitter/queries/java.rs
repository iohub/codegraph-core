use std::collections::HashMap;
use tree_sitter::{Query, Language};

/// Java Tree-sitter查询集合
pub struct JavaQueries {
    /// 方法定义查询
    pub method_definition: Query,
    /// 方法调用查询
    pub method_call: Query,
    /// 类定义查询
    pub class_definition: Query,
    /// 接口定义查询
    pub interface_definition: Query,
    /// 包声明查询
    pub package_declaration: Query,
    /// 导入声明查询
    pub import_declaration: Query,
    /// 变量声明查询
    pub variable_declaration: Query,
    /// 构造函数定义查询
    pub constructor_definition: Query,
    /// 构造函数调用查询
    pub constructor_call: Query,
    /// 字段访问查询
    pub field_access: Query,
    /// 枚举定义查询
    pub enum_definition: Query,
    /// 注解查询
    pub annotation: Query,
}

impl JavaQueries {
    pub fn new(language: &Language) -> Result<Self, tree_sitter::QueryError> {
        // 方法定义查询
        let method_definition = Query::new(
            language,
            r#"
            ; 方法定义
            (method_declaration
              name: (identifier) @method.name
              parameters: (formal_parameters) @method.params
              body: (block) @method.body
            ) @method.def

            ; 抽象方法定义
            (method_declaration
              name: (identifier) @method.name
              parameters: (formal_parameters) @method.params
              body: (_) @method.body
            ) @method.def

            ; 接口方法定义
            (interface_declaration
              body: (interface_body
                (method_declaration
                  name: (identifier) @method.name
                  parameters: (formal_parameters) @method.params
                )
              )
            ) @method.def
            "#,
        )?;

        // 方法调用查询
        let method_call = Query::new(
            language,
            r#"
            ; 方法调用
            (method_invocation
              name: (identifier) @method.called
              arguments: (argument_list) @method.args
            ) @method.call

            ; 链式方法调用
            (method_invocation
              name: (identifier) @method.called
              arguments: (argument_list) @method.args
            ) @method.call
            "#,
        )?;

        // 类定义查询
        let class_definition = Query::new(
            language,
            r#"
            ; 类定义
            (class_declaration
              name: (identifier) @class.name
              body: (class_body) @class.body
            ) @class.def

            ; 抽象类定义
            (class_declaration
              name: (identifier) @class.name
              body: (class_body) @class.body
            ) @class.def

            ; 内部类定义
            (class_declaration
              name: (identifier) @class.name
              body: (class_body) @class.body
            ) @class.def
            "#,
        )?;

        // 接口定义查询
        let interface_definition = Query::new(
            language,
            r#"
            ; 接口定义
            (interface_declaration
              name: (identifier) @interface.name
              body: (interface_body) @interface.body
            ) @interface.def

            ; 注解接口定义
            (annotation_type_declaration
              name: (identifier) @interface.name
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
              name: (scoped_identifier) @package.name
            ) @package.decl
            "#,
        )?;

        // 导入声明查询
        let import_declaration = Query::new(
            language,
            r#"
            ; 导入声明
            (import_declaration
              name: (scoped_identifier) @import.name
            ) @import.decl

            ; 静态导入
            (import_declaration
              name: (scoped_identifier) @import.name
            ) @import.decl
            "#,
        )?;

        // 变量声明查询
        let variable_declaration = Query::new(
            language,
            r#"
            ; 变量声明
            (local_variable_declaration
              declarator: (variable_declarator
                name: (identifier) @variable.name
                value: (_) @variable.value
              )
            ) @variable.decl

            ; 字段声明
            (field_declaration
              declarator: (variable_declarator
                name: (identifier) @variable.name
                value: (_) @variable.value
              )
            ) @variable.decl

            ; 参数声明
            (formal_parameter
              name: (identifier) @variable.name
            ) @variable.decl
            "#,
        )?;

        // 构造函数定义查询
        let constructor_definition = Query::new(
            language,
            r#"
            ; 构造函数定义
            (constructor_declaration
              name: (identifier) @constructor.name
              parameters: (formal_parameters) @constructor.params
              body: (block) @constructor.body
            ) @constructor.def
            "#,
        )?;

        // 构造函数调用查询
        let constructor_call = Query::new(
            language,
            r#"
            ; 构造函数调用
            (object_creation_expression
              type: (type_identifier) @constructor.name
              arguments: (argument_list) @constructor.args
            ) @constructor.call

            ; 数组创建
            (array_creation_expression
              type: (type_identifier) @constructor.name
            ) @constructor.call
            "#,
        )?;

        // 字段访问查询
        let field_access = Query::new(
            language,
            r#"
            ; 字段访问
            (field_access
              field: (identifier) @field.name
            ) @field.access

            ; 字段选择
            (field_access
              field: (identifier) @field.name
            ) @field.access
            "#,
        )?;

        // 枚举定义查询
        let enum_definition = Query::new(
            language,
            r#"
            ; 枚举定义
            (enum_declaration
              name: (identifier) @enum.name
              body: (enum_body) @enum.body
            ) @enum.def
            "#,
        )?;

        // 注解查询
        let annotation = Query::new(
            language,
            r#"
            ; 注解
            (annotation
              name: (identifier) @annotation.name
              arguments: (annotation_argument_list) @annotation.args
            ) @annotation.decl

            ; 标记注解
            (annotation
              name: (identifier) @annotation.name
            ) @annotation.decl
            "#,
        )?;

        Ok(Self {
            method_definition,
            method_call,
            class_definition,
            interface_definition,
            package_declaration,
            import_declaration,
            variable_declaration,
            constructor_definition,
            constructor_call,
            field_access,
            enum_definition,
            annotation,
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
    Variable,
    Constructor,
    Field,
    Enum,
    Annotation,
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
    pub modifiers: Vec<String>,
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
    pub method_signature: Option<String>,
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
    pub modifiers: Vec<String>,
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