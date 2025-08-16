use std::collections::HashMap;
use tree_sitter::{Query, Language};

/// JavaScript Tree-sitter查询集合
pub struct JavaScriptQueries {
    /// 函数定义查询
    pub function_definition: Query,
    /// 函数调用查询
    pub function_call: Query,
    /// 类定义查询
    pub class_definition: Query,
    /// 对象定义查询
    pub object_definition: Query,
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
    /// 箭头函数查询
    pub arrow_function: Query,
    /// 函数表达式查询
    pub function_expression: Query,
    /// 对象方法查询
    pub object_method: Query,
    /// 类方法查询
    pub class_method: Query,
    /// 构造函数查询
    pub constructor: Query,
    /// 模块查询
    pub module: Query,
}

impl JavaScriptQueries {
    pub fn new(language: &Language) -> Result<Self, tree_sitter::QueryError> {
        // 函数定义查询 - 函数声明
        let function_definition = Query::new(
            language,
            "(function_declaration)",
        )?;

        // 函数调用查询
        let function_call = Query::new(
            language,
            "(call_expression)",
        )?;

        // 类定义查询
        let class_definition = Query::new(
            language,
            "(class_declaration)",
        )?;

        // 对象定义查询
        let object_definition = Query::new(
            language,
            "(object)",
        )?;

        // 导入语句查询
        let import_statement = Query::new(
            language,
            "(import_statement)",
        )?;

        // 导出语句查询
        let export_statement = Query::new(
            language,
            "(export_statement)",
        )?;

        // 变量声明查询
        let variable_declaration = Query::new(
            language,
            "(variable_declaration)",
        )?;

        // 方法定义查询
        let method_definition = Query::new(
            language,
            "(method_definition)",
        )?;

        // 装饰器查询
        let decorator = Query::new(
            language,
            "(decorator)",
        )?;

        // 箭头函数查询
        let arrow_function = Query::new(
            language,
            "(arrow_function)",
        )?;

        // 函数表达式查询
        let function_expression = Query::new(
            language,
            "(function_expression)",
        )?;

        // 对象方法查询
        let object_method = Query::new(
            language,
            "(pair key: (property_identifier) value: (function_expression))",
        )?;

        // 类方法查询
        let class_method = Query::new(
            language,
            "(method_definition)",
        )?;

        // 构造函数查询
        let constructor = Query::new(
            language,
            "(method_definition name: (property_identifier) @constructor)",
        )?;

        // 模块查询
        let module = Query::new(
            language,
            "(program)",
        )?;

        Ok(Self {
            function_definition,
            function_call,
            class_definition,
            object_definition,
            import_statement,
            export_statement,
            variable_declaration,
            method_definition,
            decorator,
            arrow_function,
            function_expression,
            object_method,
            class_method,
            constructor,
            module,
        })
    }
}

/// JavaScript代码片段类型
#[derive(Debug, Clone, PartialEq)]
pub enum JavaScriptSnippetType {
    Function,
    Class,
    Object,
    Method,
    Module,
    Variable,
    Import,
    Export,
    Decorator,
    ArrowFunction,
    FunctionExpression,
    ObjectMethod,
    ClassMethod,
    Constructor,
}

/// JavaScript代码片段信息
#[derive(Debug, Clone)]
pub struct JavaScriptSnippet {
    pub snippet_type: JavaScriptSnippetType,
    pub name: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub file_path: String,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub object_name: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub decorators: Vec<String>,
    pub extends: Vec<String>,
    pub implements: Vec<String>,
    pub is_async: bool,
    pub is_generator: bool,
}

/// JavaScript函数调用信息
#[derive(Debug, Clone)]
pub struct JavaScriptFunctionCall {
    pub caller_name: String,
    pub called_name: String,
    pub caller_location: (usize, usize), // (line, column)
    pub called_location: (usize, usize),
    pub caller_file: String,
    pub called_file: Option<String>,
    pub is_resolved: bool,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub object_name: Option<String>,
    pub arguments: Vec<String>,
    pub is_method_call: bool,
    pub is_constructor_call: bool,
}

/// JavaScript作用域信息
#[derive(Debug, Clone)]
pub struct JavaScriptScope {
    pub name: String,
    pub scope_type: JavaScriptSnippetType,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub parent_scope: Option<String>,
    pub module_name: Option<String>,
    pub class_name: Option<String>,
    pub object_name: Option<String>,
}

/// JavaScript代码分析结果
#[derive(Debug, Clone)]
pub struct JavaScriptAnalysisResult {
    pub snippets: Vec<JavaScriptSnippet>,
    pub function_calls: Vec<JavaScriptFunctionCall>,
    pub scopes: Vec<JavaScriptScope>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub modules: HashMap<String, Vec<String>>,
    pub classes: HashMap<String, Vec<String>>,
    pub objects: HashMap<String, Vec<String>>,
} 