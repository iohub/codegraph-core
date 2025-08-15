use std::collections::HashMap;
use tree_sitter::{Query, Language};

/// TypeScript Tree-sitter查询集合
pub struct TypeScriptQueries {
    /// 函数定义查询
    pub function_definition: Query,
    /// 函数调用查询
    pub function_call: Query,
    /// 类定义查询
    pub class_definition: Query,
    /// 接口定义查询
    pub interface_definition: Query,
    /// 类型定义查询
    pub type_definition: Query,
    /// 导入语句查询
    pub import_statement: Query,
    /// 导出语句查询
    pub export_statement: Query,
    /// 变量声明查询
    pub variable_declaration: Query,
    /// 方法定义查询
    pub method_definition: Query,
    /// 装饰器查询
    pub decorator: Query,
    /// 泛型查询
    pub generic_type: Query,
    /// 枚举定义查询
    pub enum_definition: Query,
    /// 命名空间查询
    pub namespace_definition: Query,
}

impl TypeScriptQueries {
    pub fn new(language: &Language) -> Result<Self, tree_sitter::QueryError> {
        // 函数定义查询 - 最基本的模式
        let function_definition = Query::new(
            language,
            "(function_declaration)",
        )?;

        // 函数调用查询 - 最基本的模式
        let function_call = Query::new(
            language,
            "(call_expression)",
        )?;

        // 类定义查询 - 最基本的模式
        let class_definition = Query::new(
            language,
            "(class_declaration)",
        )?;

        // 接口定义查询 - 最基本的模式
        let interface_definition = Query::new(
            language,
            "(interface_declaration)",
        )?;

        // 类型定义查询 - 最基本的模式
        let type_definition = Query::new(
            language,
            "(type_alias_declaration)",
        )?;

        // 导入语句查询 - 最基本的模式
        let import_statement = Query::new(
            language,
            "(import_statement)",
        )?;

        // 导出语句查询 - 最基本的模式
        let export_statement = Query::new(
            language,
            "(export_statement)",
        )?;

        // 变量声明查询 - 最基本的模式
        let variable_declaration = Query::new(
            language,
            "(variable_declaration)",
        )?;

        // 方法定义查询 - 最基本的模式
        let method_definition = Query::new(
            language,
            "(method_definition)",
        )?;

        // 装饰器查询 - 最基本的模式
        let decorator = Query::new(
            language,
            "(decorator)",
        )?;

        // 泛型查询 - 最基本的模式
        let generic_type = Query::new(
            language,
            "(generic_type)",
        )?;

        // 枚举定义查询 - 最基本的模式
        let enum_definition = Query::new(
            language,
            "(enum_declaration)",
        )?;

        // 命名空间查询 - 使用identifier作为占位符
        let namespace_definition = Query::new(
            language,
            "(identifier)",
        )?;

        Ok(Self {
            function_definition,
            function_call,
            class_definition,
            interface_definition,
            type_definition,
            import_statement,
            export_statement,
            variable_declaration,
            method_definition,
            decorator,
            generic_type,
            enum_definition,
            namespace_definition,
        })
    }
}

/// TypeScript代码片段类型
#[derive(Debug, Clone, PartialEq)]
pub enum TypeScriptSnippetType {
    Function,
    Class,
    Interface,
    Type,
    Method,
    Module,
    Variable,
    Import,
    Export,
    Decorator,
    Generic,
    Enum,
    Namespace,
}

/// TypeScript代码片段信息
#[derive(Debug, Clone)]
pub struct TypeScriptSnippet {
    pub snippet_type: TypeScriptSnippetType,
    pub name: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub file_path: String,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub interface_name: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub decorators: Vec<String>,
    pub type_parameters: Vec<String>,
    pub extends: Vec<String>,
    pub implements: Vec<String>,
}

/// TypeScript函数调用信息
#[derive(Debug, Clone)]
pub struct TypeScriptFunctionCall {
    pub caller_name: String,
    pub called_name: String,
    pub caller_location: (usize, usize), // (line, column)
    pub called_location: (usize, usize),
    pub caller_file: String,
    pub called_file: Option<String>,
    pub is_resolved: bool,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub interface_name: Option<String>,
    pub arguments: Vec<String>,
    pub type_arguments: Vec<String>,
}

/// TypeScript作用域信息
#[derive(Debug, Clone)]
pub struct TypeScriptScope {
    pub name: String,
    pub scope_type: TypeScriptSnippetType,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub parent_scope: Option<String>,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub interface_name: Option<String>,
}

/// TypeScript代码分析结果
#[derive(Debug, Clone)]
pub struct TypeScriptAnalysisResult {
    pub snippets: Vec<TypeScriptSnippet>,
    pub function_calls: Vec<TypeScriptFunctionCall>,
    pub scopes: Vec<TypeScriptScope>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub modules: HashMap<String, Vec<String>>,
    pub classes: HashMap<String, Vec<String>>,
    pub interfaces: HashMap<String, Vec<String>>,
} 