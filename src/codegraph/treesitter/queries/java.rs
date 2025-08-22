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
    /// 记录定义查询
    pub record_definition: Query,
    /// 模块定义查询
    pub module_definition: Query,
    /// 类型参数查询
    pub type_parameters: Query,
    /// 泛型类型查询
    pub generic_type: Query,
    /// 数组访问查询
    pub array_access: Query,
    /// 异常处理查询
    pub exception_handling: Query,
    /// 循环语句查询
    pub loop_statements: Query,
    /// 条件语句查询
    pub conditional_statements: Query,
}

impl JavaQueries {
    pub fn new(language: &Language) -> Result<Self, tree_sitter::QueryError> {
        // 方法定义查询 - 基于实际语法
        let method_definition = Query::new(
            language,
            r#"
            ; 方法声明
            (method_declaration
              (modifiers) @method.modifiers
              (type_parameters) @method.type_params
              type: (_unannotated_type) @method.return_type
              name: (identifier) @method.name
              parameters: (formal_parameters) @method.params
              body: (block) @method.body
            ) @method.def

            ; 抽象方法声明
            (method_declaration
              (modifiers) @method.modifiers
              (type_parameters) @method.type_params
              type: (_unannotated_type) @method.return_type
              name: (identifier) @method.name
              parameters: (formal_parameters) @method.params
            ) @method.def

            ; 接口方法声明
            (interface_declaration
              body: (interface_body
                (method_declaration
                  (modifiers) @method.modifiers
                  (type_parameters) @method.type_params
                  type: (_unannotated_type) @method.return_type
                  name: (identifier) @method.name
                  parameters: (formal_parameters) @method.params
                )
              )
            ) @method.def

            ; 记录方法声明
            (record_declaration
              body: (class_body
                (method_declaration
                  (modifiers) @method.modifiers
                  (type_parameters) @method.type_params
                  type: (_unannotated_type) @method.return_type
                  name: (identifier) @method.name
                  parameters: (formal_parameters) @method.params
                  body: (block) @method.body
                )
              )
            ) @method.def

            ; 紧凑构造函数声明
            (compact_constructor_declaration
              (modifiers) @method.modifiers
              name: (identifier) @method.name
              body: (block) @method.body
            ) @method.def
            "#,
        )?;

        // 方法调用查询 - 基于实际语法
        let method_call = Query::new(
            language,
            r#"
            ; 方法调用 - 直接调用
            (method_invocation
              (identifier) @method.called
              arguments: (argument_list) @method.args
            ) @method.call

            ; 带对象的方法调用
            (method_invocation
              (primary_expression) @method.object
              (type_arguments) @method.type_args
              (identifier) @method.called
              arguments: (argument_list) @method.args
            ) @method.call

            ; 方法引用
            (method_reference
              (primary_expression) @method.object
              (type_arguments) @method.type_args
              (identifier) @method.called
            ) @method.reference

            ; 构造函数引用
            (method_reference
              (_type) @method.object
              (type_arguments) @method.type_args
              (identifier) @method.called
            ) @method.reference
            "#,
        )?;

        // 类定义查询 - 基于实际语法
        let class_definition = Query::new(
            language,
            r#"
            ; 类声明
            (class_declaration
              (modifiers) @class.modifiers
              name: (identifier) @class.name
              (type_parameters) @class.type_params
              (superclass) @class.superclass
              (super_interfaces) @class.interfaces
              (permits) @class.permits
              body: (class_body) @class.body
            ) @class.def

            ; 记录声明
            (record_declaration
              (modifiers) @class.modifiers
              name: (identifier) @class.name
              (type_parameters) @class.type_params
              parameters: (formal_parameters) @class.params
              (super_interfaces) @class.interfaces
              body: (class_body) @class.body
            ) @class.def
            "#,
        )?;

        // 接口定义查询 - 基于实际语法
        let interface_definition = Query::new(
            language,
            r#"
            ; 接口声明
            (interface_declaration
              (modifiers) @interface.modifiers
              name: (identifier) @interface.name
              (type_parameters) @interface.type_params
              (extends_interfaces) @interface.extends
              (permits) @interface.permits
              body: (interface_body) @interface.body
            ) @interface.def

            ; 注解类型声明
            (annotation_type_declaration
              (modifiers) @interface.modifiers
              name: (identifier) @interface.name
              body: (annotation_type_body) @interface.body
            ) @interface.def
            "#,
        )?;

        // 包声明查询 - 基于实际语法
        let package_declaration = Query::new(
            language,
            r#"
            ; 包声明
            (package_declaration
              (marker_annotation) @package.annotations
              (annotation) @package.annotations
              (identifier) @package.name
            ) @package.decl

            ; 包声明 - 带作用域名称
            (package_declaration
              (marker_annotation) @package.annotations
              (annotation) @package.annotations
              (scoped_identifier) @package.name
            ) @package.decl
            "#,
        )?;

        // 导入声明查询 - 基于实际语法
        let import_declaration = Query::new(
            language,
            r#"
            ; 导入声明
            (import_declaration
              "static" @import.static
              (identifier) @import.name
              (asterisk) @import.asterisk
            ) @import.decl

            ; 导入声明 - 带作用域名称
            (import_declaration
              "static" @import.static
              (scoped_identifier) @import.name
              (asterisk) @import.asterisk
            ) @import.decl
            "#,
        )?;

        // 变量声明查询 - 基于实际语法
        let variable_declaration = Query::new(
            language,
            r#"
            ; 局部变量声明
            (local_variable_declaration
              (modifiers) @variable.modifiers
              type: (_unannotated_type) @variable.type
              declarator: (variable_declarator
                name: (_variable_declarator_id) @variable.name
                value: (_variable_initializer) @variable.value
              )
            ) @variable.decl

            ; 字段声明
            (field_declaration
              (modifiers) @variable.modifiers
              type: (_unannotated_type) @variable.type
              declarator: (variable_declarator
                name: (_variable_declarator_id) @variable.name
                value: (_variable_initializer) @variable.value
              )
            ) @variable.decl

            ; 参数声明
            (formal_parameter
              (modifiers) @variable.modifiers
              type: (_unannotated_type) @variable.type
              name: (_variable_declarator_id) @variable.name
            ) @variable.decl

            ; 记录参数
            (record_declaration
              parameters: (formal_parameters
                (formal_parameter
                  (modifiers) @variable.modifiers
                  type: (_unannotated_type) @variable.type
                  name: (_variable_declarator_id) @variable.name
                )
              )
            ) @variable.decl

            ; 常量声明
            (constant_declaration
              (modifiers) @variable.modifiers
              type: (_unannotated_type) @variable.type
              declarator: (variable_declarator
                name: (_variable_declarator_id) @variable.name
                value: (_variable_initializer) @variable.value
              )
            ) @variable.decl
            "#,
        )?;

        // 构造函数定义查询 - 基于实际语法
        let constructor_definition = Query::new(
            language,
            r#"
            ; 构造函数声明
            (constructor_declaration
              (modifiers) @constructor.modifiers
              (type_parameters) @constructor.type_params
              name: (identifier) @constructor.name
              parameters: (formal_parameters) @constructor.params
              (throws) @constructor.throws
              body: (constructor_body) @constructor.body
            ) @constructor.def

            ; 紧凑构造函数声明
            (compact_constructor_declaration
              (modifiers) @constructor.modifiers
              name: (identifier) @constructor.name
              body: (block) @constructor.body
            ) @constructor.def
            "#,
        )?;

        // 构造函数调用查询 - 基于实际语法
        let constructor_call = Query::new(
            language,
            r#"
            ; 对象创建表达式
            (object_creation_expression
              type_arguments: (type_arguments) @constructor.type_args
              type: (_simple_type) @constructor.type
              arguments: (argument_list) @constructor.args
              body: (class_body) @constructor.body
            ) @constructor.call

            ; 数组创建表达式
            (array_creation_expression
              type: (_simple_type) @constructor.type
              dimensions: (dimensions_expr) @constructor.dimensions
              value: (array_initializer) @constructor.value
            ) @constructor.call

            ; 带维度的数组创建
            (array_creation_expression
              type: (_simple_type) @constructor.type
              dimensions: (dimensions) @constructor.dimensions
            ) @constructor.call
            "#,
        )?;

        // 字段访问查询 - 基于实际语法
        let field_access = Query::new(
            language,
            r#"
            ; 字段访问
            (field_access
              object: (primary_expression) @field.object
              super: (super) @field.super
              field: (identifier) @field.name
            ) @field.access

            ; 带this的字段访问
            (field_access
              object: (this) @field.object
              field: (identifier) @field.name
            ) @field.access
            "#,
        )?;

        // 枚举定义查询 - 基于实际语法
        let enum_definition = Query::new(
            language,
            r#"
            ; 枚举声明
            (enum_declaration
              (modifiers) @enum.modifiers
              name: (identifier) @enum.name
              (super_interfaces) @enum.interfaces
              body: (enum_body) @enum.body
            ) @enum.def

            ; 枚举常量
            (enum_constant
              (modifiers) @enum.constant_modifiers
              name: (identifier) @enum.constant
              arguments: (argument_list) @enum.args
              body: (class_body) @enum.constant_body
            ) @enum.constant_def
            "#,
        )?;

        // 注解查询 - 基于实际语法
        let annotation = Query::new(
            language,
            r#"
            ; 标记注解
            (marker_annotation
              name: (_name) @annotation.name
            ) @annotation.decl

            ; 带参数的注解
            (annotation
              name: (_name) @annotation.name
              arguments: (annotation_argument_list) @annotation.args
            ) @annotation.decl

            ; 注解类型元素声明
            (annotation_type_element_declaration
              (modifiers) @annotation.modifiers
              type: (_unannotated_type) @annotation.type
              name: (identifier) @annotation.name
              (dimensions) @annotation.dimensions
              (_default_value) @annotation.default
            ) @annotation.decl
            "#,
        )?;

        // 记录定义查询 - 基于实际语法
        let record_definition = Query::new(
            language,
            r#"
            ; 记录声明
            (record_declaration
              (modifiers) @record.modifiers
              name: (identifier) @record.name
              (type_parameters) @record.type_params
              parameters: (formal_parameters) @record.params
              (super_interfaces) @record.interfaces
              body: (class_body) @record.body
            ) @record.def
            "#,
        )?;

        // 模块定义查询 - 基于实际语法
        let module_definition = Query::new(
            language,
            r#"
            ; 模块声明
            (module_declaration
              (marker_annotation) @module.annotations
              (annotation) @module.annotations
              (open) @module.open
              (identifier) @module.name
              body: (module_body) @module.body
            ) @module.def

            ; 模块声明 - 带作用域名称
            (module_declaration
              (marker_annotation) @module.annotations
              (annotation) @module.annotations
              (open) @module.open
              (scoped_identifier) @module.name
              body: (module_body) @module.body
            ) @module.def

            ; 模块指令 - requires
            (requires_module_directive
              (requires_modifier) @module.requires_modifiers
              (identifier) @module.requires_module
            ) @module.requires

            (requires_module_directive
              (requires_modifier) @module.requires_modifiers
              (scoped_identifier) @module.requires_module
            ) @module.requires

            ; 模块指令 - exports
            (exports_module_directive
              (identifier) @module.exports_package
              (identifier) @module.exports_modules
            ) @module.exports

            (exports_module_directive
              (scoped_identifier) @module.exports_package
              (scoped_identifier) @module.exports_modules
            ) @module.exports

            ; 模块指令 - opens
            (opens_module_directive
              (identifier) @module.opens_package
              (identifier) @module.opens_modules
            ) @module.opens

            (opens_module_directive
              (scoped_identifier) @module.opens_package
              (scoped_identifier) @module.opens_modules
            ) @module.opens

            ; 模块指令 - uses
            (uses_module_directive
              (identifier) @module.uses_type
            ) @module.uses

            (uses_module_directive
              (scoped_identifier) @module.uses_type
            ) @module.uses

            ; 模块指令 - provides
            (provides_module_directive
              (identifier) @module.provides_provided
              (identifier) @module.provides_provider
            ) @module.provides

            (provides_module_directive
              (scoped_identifier) @module.provides_provided
              (scoped_identifier) @module.provides_provider
            ) @module.provides
            "#,
        )?;

        // 类型参数查询 - 基于实际语法
        let type_parameters = Query::new(
            language,
            r#"
            ; 类型参数
            (type_parameter
              (marker_annotation) @type_param.annotations
              (annotation) @type_param.annotations
              name: (type_identifier) @type_param.name
              (type_bound) @type_param.bound
            ) @type_param.def

            ; 类型边界
            (type_bound
              (extends) @type_param.extends
              types: (_type) @type_param.types
            ) @type_param.bound
            "#,
        )?;

        // 泛型类型查询 - 基于实际语法
        let generic_type = Query::new(
            language,
            r#"
            ; 泛型类型
            (generic_type
              name: (type_identifier) @generic.name
              arguments: (type_arguments) @generic.args
            ) @generic.def

            ; 带作用域的泛型类型
            (generic_type
              name: (scoped_type_identifier) @generic.name
              arguments: (type_arguments) @generic.args
            ) @generic.def

            ; 通配符类型
            (wildcard
              (marker_annotation) @wildcard.annotations
              (annotation) @wildcard.annotations
              (_wildcard_bounds) @wildcard.bounds
            ) @wildcard.def

            ; 通配符边界
            (_wildcard_bounds
              (extends) @wildcard.extends
              (super) @wildcard.super
              type: (_type) @wildcard.type
            ) @wildcard.bounds
            "#,
        )?;

        // 数组访问查询 - 基于实际语法
        let array_access = Query::new(
            language,
            r#"
            ; 数组访问
            (array_access
              array: (primary_expression) @array.array
              index: (expression) @array.index
            ) @array.access
            "#,
        )?;

        // 异常处理查询 - 基于实际语法
        let exception_handling = Query::new(
            language,
            r#"
            ; try语句
            (try_statement
              body: (block) @try.body
              catch_clause: (catch_clause) @try.catch
              finally_clause: (finally_clause) @try.finally
            ) @try.stmt

            ; 带资源的try语句
            (try_with_resources_statement
              resources: (resource_specification) @try.resources
              body: (block) @try.body
              catch_clause: (catch_clause) @try.catch
              finally_clause: (finally_clause) @try.finally
            ) @try.with_resources

            ; catch子句
            (catch_clause
              parameter: (catch_formal_parameter) @catch.param
              body: (block) @catch.body
            ) @catch.clause

            ; finally子句
            (finally_clause
              body: (block) @finally.body
            ) @finally.clause

            ; throw语句
            (throw_statement
              expression: (expression) @throw.expr
            ) @throw.stmt

            ; 资源规范
            (resource_specification
              resources: (resource) @try.resource_list
            ) @try.resource_spec

            ; 资源
            (resource
              (modifiers) @try.resource_modifiers
              type: (_unannotated_type) @try.resource_type
              name: (_variable_declarator_id) @try.resource_name
              value: (expression) @try.resource_value
            ) @try.resource
            "#,
        )?;

        // 循环语句查询 - 基于实际语法
        let loop_statements = Query::new(
            language,
            r#"
            ; for循环
            (for_statement
              init: (_) @for.init
              condition: (expression) @for.condition
              update: (_) @for.update
              body: (statement) @for.body
            ) @for.stmt

            ; 增强for循环
            (enhanced_for_statement
              (modifiers) @for.modifiers
              type: (_unannotated_type) @for.type
              name: (_variable_declarator_id) @for.name
              value: (expression) @for.value
              body: (statement) @for.body
            ) @for.enhanced

            ; while循环
            (while_statement
              condition: (parenthesized_expression) @while.condition
              body: (statement) @while.body
            ) @while.stmt

            ; do-while循环
            (do_statement
              body: (statement) @do.body
              condition: (parenthesized_expression) @do.condition
            ) @do.stmt
            "#,
        )?;

        // 条件语句查询 - 基于实际语法
        let conditional_statements = Query::new(
            language,
            r#"
            ; if语句
            (if_statement
              condition: (parenthesized_expression) @if.condition
              consequence: (statement) @if.consequence
              alternative: (statement) @if.alternative
            ) @if.stmt

            ; switch表达式
            (switch_expression
              condition: (parenthesized_expression) @switch.condition
              body: (switch_block) @switch.body
            ) @switch.expr

            ; switch块
            (switch_block
              statement_groups: (switch_block_statement_group) @switch.statement_groups
              rules: (switch_rule) @switch.rules
            ) @switch.block

            ; switch标签
            (switch_label
              case: (case) @switch.case
              pattern: (pattern) @switch.pattern
              guard: (guard) @switch.guard
              default: (default) @switch.default
            ) @switch.label

            ; 模式
            (pattern
              type: (type_pattern) @switch.type_pattern
              record: (record_pattern) @switch.record_pattern
            ) @switch.pattern

            ; 守卫
            (guard
              when: (when) @switch.when
              expression: (expression) @switch.guard_expr
            ) @switch.guard

            ; 三元表达式
            (ternary_expression
              condition: (expression) @ternary.condition
              consequence: (expression) @ternary.consequence
              alternative: (expression) @ternary.alternative
            ) @ternary.expr
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
            record_definition,
            module_definition,
            type_parameters,
            generic_type,
            array_access,
            exception_handling,
            loop_statements,
            conditional_statements,
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
    Record,
    Module,
    TypeParameter,
    GenericType,
    ArrayAccess,
    ExceptionHandling,
    LoopStatement,
    ConditionalStatement,
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
    pub type_parameters: Option<Vec<String>>,
    pub superclass: Option<String>,
    pub interfaces: Option<Vec<String>>,
    pub generic_arguments: Option<Vec<String>>,
    pub bounds: Option<Vec<String>>,
    pub exception_types: Option<Vec<String>>,
    pub loop_type: Option<String>, // "for", "while", "do-while", "enhanced-for"
    pub condition_type: Option<String>, // "if", "switch", "ternary"
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
    pub object_expression: Option<String>,
    pub arguments: Option<Vec<String>>,
    pub type_arguments: Option<Vec<String>>,
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
    pub type_parameters: Option<Vec<String>>,
    pub superclass: Option<String>,
    pub interfaces: Option<Vec<String>>,
    pub is_open: Option<bool>, // For modules
    pub is_sealed: Option<bool>, // For classes/interfaces
    pub permits: Option<Vec<String>>, // For sealed classes
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
    pub records: HashMap<String, Vec<String>>,
    pub modules: HashMap<String, Vec<String>>,
    pub enums: HashMap<String, Vec<String>>,
    pub annotations: HashMap<String, Vec<String>>,
    pub generics: HashMap<String, Vec<String>>,
} 