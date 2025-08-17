use std::collections::HashMap;
use tree_sitter::{Query, Language};

/// Rust Tree-sitter查询集合
pub struct RustQueries {
    /// 函数定义查询
    pub function_definition: Query,
    /// 函数调用查询
    pub function_call: Query,
    /// 结构体定义查询
    pub struct_definition: Query,
    /// 枚举定义查询
    pub enum_definition: Query,
    /// 模块定义查询
    pub mod_definition: Query,
    /// trait定义查询
    pub trait_definition: Query,
    /// impl块查询
    pub impl_block: Query,
    /// 变量声明查询
    pub variable_declaration: Query,
    /// 类型定义查询
    pub type_definition: Query,
    /// 宏定义查询
    pub macro_definition: Query,
    /// 宏调用查询
    pub macro_invocation: Query,
    /// 字段访问查询
    pub field_access: Query,
    /// 方法调用查询
    pub method_call: Query,
    /// 泛型参数查询
    pub type_parameters: Query,
    /// 生命周期查询
    pub lifetime: Query,
    /// 属性查询
    pub attribute: Query,
    /// 导入查询
    pub use_declaration: Query,
    /// 常量定义查询
    pub const_definition: Query,
    /// 静态变量定义查询
    pub static_definition: Query,
}

impl RustQueries {
    pub fn new(language: &Language) -> Result<Self, tree_sitter::QueryError> {
        // 函数定义查询
        let function_definition = Query::new(
            language,
            r#"
            ; 函数定义
            (function_item
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
            ; 函数调用
            (call_expression
              function: (identifier) @function.called
              arguments: (arguments) @function.args
            ) @function.call
            "#,
        )?;

        // 结构体定义查询
        let struct_definition = Query::new(
            language,
            r#"
            ; 结构体定义
            (struct_item
              name: (type_identifier) @struct.name
            ) @struct.def
            "#,
        )?;

        // 枚举定义查询
        let enum_definition = Query::new(
            language,
            r#"
            ; 枚举定义
            (enum_item
              name: (type_identifier) @enum.name
            ) @enum.def
            "#,
        )?;

        // 模块定义查询
        let mod_definition = Query::new(
            language,
            r#"
            ; 模块定义
            (mod_item
              name: (identifier) @mod.name
            ) @mod.def
            "#,
        )?;

        // trait定义查询
        let trait_definition = Query::new(
            language,
            r#"
            ; trait定义
            (trait_item
              name: (type_identifier) @trait.name
            ) @trait.def
            "#,
        )?;

        // impl块查询
        let impl_block = Query::new(
            language,
            r#"
            ; impl块
            (impl_item
              type: (type_identifier) @impl.type
            ) @impl.block
            "#,
        )?;

        // 变量声明查询
        let variable_declaration = Query::new(
            language,
            r#"
            ; let声明
            (let_declaration
              pattern: (identifier) @variable.name
            ) @variable.decl
            "#,
        )?;

        // 类型定义查询
        let type_definition = Query::new(
            language,
            r#"
            ; 类型别名
            (type_item
              name: (type_identifier) @type.name
            ) @type.def
            "#,
        )?;

        // 宏定义查询
        let macro_definition = Query::new(
            language,
            r#"
            ; 宏定义
            (macro_definition
              name: (identifier) @macro.name
            ) @macro.def
            "#,
        )?;

        // 宏调用查询
        let macro_invocation = Query::new(
            language,
            r#"
            ; 宏调用
            (macro_invocation
              macro: (identifier) @macro.called
            ) @macro.call
            "#,
        )?;

        // 字段访问查询
        let field_access = Query::new(
            language,
            r#"
            ; 字段访问
            (field_expression
              field: (field_identifier) @field.name
            ) @field.access
            "#,
        )?;

        // 方法调用查询
        let method_call = Query::new(
            language,
            r#"
            ; 方法调用
            (call_expression
              function: (field_expression
                field: (field_identifier) @method.called
              )
              arguments: (arguments) @method.args
            ) @method.call
            "#,
        )?;

        // 泛型参数查询
        let type_parameters = Query::new(
            language,
            r#"
            ; 类型参数
            (type_parameters) @type.params
            "#,
        )?;

        // 生命周期查询
        let lifetime = Query::new(
            language,
            r#"
            ; 生命周期
            (lifetime) @lifetime.name
            "#,
        )?;

        // 属性查询
        let attribute = Query::new(
            language,
            r#"
            ; 属性
            (attribute_item) @attr.item
            "#,
        )?;

        // 导入查询
        let use_declaration = Query::new(
            language,
            r#"
            ; use声明
            (use_declaration) @use.decl
            "#,
        )?;

        // 常量定义查询
        let const_definition = Query::new(
            language,
            r#"
            ; 常量定义
            (const_item
              name: (identifier) @const.name
            ) @const.def
            "#,
        )?;

        // 静态变量定义查询
        let static_definition = Query::new(
            language,
            r#"
            ; 静态变量定义
            (static_item
              name: (identifier) @static.name
            ) @static.def
            "#,
        )?;

        Ok(Self {
            function_definition,
            function_call,
            struct_definition,
            enum_definition,
            mod_definition,
            trait_definition,
            impl_block,
            variable_declaration,
            type_definition,
            macro_definition,
            macro_invocation,
            field_access,
            method_call,
            type_parameters,
            lifetime,
            attribute,
            use_declaration,
            const_definition,
            static_definition,
        })
    }
}

/// Rust代码片段类型
#[derive(Debug, Clone)]
pub enum RustSnippetType {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Macro,
    Type,
    Const,
    Static,
    Variable,
}

/// Rust代码片段
#[derive(Debug, Clone)]
pub struct RustSnippet {
    pub snippet_type: RustSnippetType,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub file_path: String,
    pub content: String,
    pub scope: Option<String>,
    pub modifiers: Vec<String>,
    pub generics: Vec<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
}

/// Rust方法调用
#[derive(Debug, Clone)]
pub struct RustMethodCall {
    pub caller_name: String,
    pub called_name: String,
    pub location: (usize, usize), // (line, column)
    pub file_path: String,
    pub arguments: Vec<String>,
    pub type_arguments: Vec<String>,
}

/// Rust作用域信息
#[derive(Debug, Clone)]
pub struct RustScope {
    pub name: String,
    pub scope_type: RustSnippetType,
    pub start_line: usize,
    pub end_line: usize,
    pub file_path: String,
    pub parent_scope: Option<String>,
}

/// Rust分析结果
#[derive(Debug, Clone)]
pub struct RustAnalysisResult {
    pub snippets: Vec<RustSnippet>,
    pub method_calls: Vec<RustMethodCall>,
    pub scopes: Vec<RustScope>,
    pub imports: Vec<String>,
    pub modules: HashMap<String, Vec<String>>,
    pub structs: HashMap<String, Vec<String>>,
    pub enums: HashMap<String, Vec<String>>,
    pub traits: HashMap<String, Vec<String>>,
    pub macros: HashMap<String, Vec<String>>,
    pub types: HashMap<String, Vec<String>>,
} 