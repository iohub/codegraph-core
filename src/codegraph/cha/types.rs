use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 增强的类信息结构，支持继承层次分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedClassInfo {
    pub id: Uuid,
    pub name: String,
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub namespace: String,
    pub language: String,
    pub parent_class: Option<String>,           // 直接父类
    pub base_classes: Vec<String>,              // 所有基类（传递闭包）
    pub virtual_methods: Vec<Uuid>,            // 虚方法/可重写方法
    pub method_signatures: HashMap<String, MethodSignature>, // 方法重载支持
    pub inheritance_chain: Vec<String>,        // 完整继承路径
    pub implemented_interfaces: Vec<String>,   // 实现的接口
    pub class_type: ClassType,                 // 类类型
    pub member_functions: Vec<Uuid>,           // 成员函数
    pub member_variables: Vec<String>,         // 成员变量
}

/// 方法签名信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSignature {
    pub id: Uuid,
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub is_virtual: bool,
    pub is_override: bool,
    pub base_method: Option<String>,           // 被重写的基类方法
    pub access_modifier: AccessModifier,       // 访问修饰符
    pub is_static: bool,                       // 是否为静态方法
}

/// 参数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub type_name: Option<String>,
    pub default_value: Option<String>,
    pub is_reference: bool,                    // 是否为引用参数
    pub is_const: bool,                        // 是否为const参数
}

/// 访问修饰符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessModifier {
    Public,
    Protected,
    Private,
    Internal,
    Default,
}

/// 类类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClassType {
    Class,
    Struct,
    Interface,
    Trait,
    Enum,
    Abstract,
}

/// 调用点信息结构
#[derive(Debug, Clone)]
pub struct CallSite {
    pub id: Uuid,
    pub caller_function: Uuid,
    pub callee_name: String,
    pub receiver_type: Option<String>,         // 接收者对象的声明类型
    pub receiver_expression: Option<String>,   // 接收者的AST表达式
    pub call_type: CallType,
    pub line_number: usize,
    pub file_path: PathBuf,
    pub parameters: Vec<CallParameter>,
    pub context: CallContext,                  // 调用上下文
}

/// 调用类型
#[derive(Debug, Clone)]
pub enum CallType {
    Static,             // 静态方法调用
    Virtual,            // 虚方法调用
    Constructor,        // 构造函数调用
    Function,           // 自由函数调用
    Method,             // 实例方法调用
    TraitMethod,        // Trait方法调用（Rust）
    InterfaceMethod,    // 接口方法调用（Java）
}

/// 调用参数
#[derive(Debug, Clone)]
pub struct CallParameter {
    pub expression: String,
    pub inferred_type: Option<String>,
    pub is_literal: bool,
    pub is_reference: bool,
}

/// 调用上下文
#[derive(Debug, Clone)]
pub struct CallContext {
    pub scope_type: ScopeType,                 // 作用域类型
    pub containing_class: Option<String>,      // 包含的类
    pub containing_function: Option<String>,   // 包含的函数
    pub namespace: Option<String>,             // 命名空间
}

/// 作用域类型
#[derive(Debug, Clone)]
pub enum ScopeType {
    Global,
    Class,
    Function,
    Block,
}

/// 继承边信息
#[derive(Debug, Clone)]
pub struct InheritanceEdge {
    pub edge_type: InheritanceEdgeType,
    pub metadata: Option<serde_json::Value>,
}

/// 继承边类型
#[derive(Debug, Clone)]
pub enum InheritanceEdgeType {
    Extends,            // 继承
    Implements,         // 实现接口
    Mixin,              // 混入（如Rust的trait）
}

/// 方法解析结果
#[derive(Debug, Clone)]
pub struct MethodResolutionResult {
    pub call_site_id: Uuid,
    pub target_methods: Vec<Uuid>,
    pub resolution_type: ResolutionType,
    pub confidence: f64,                       // 解析置信度
}

/// 解析类型
#[derive(Debug, Clone)]
pub enum ResolutionType {
    Direct,             // 直接解析
    Virtual,            // 虚方法解析
    Override,           // 重写方法解析
    Interface,          // 接口方法解析
    Trait,              // Trait方法解析
    Ambiguous,          // 歧义解析
    Unresolved,         // 未解析
}

/// CHA分析统计信息
#[derive(Debug, Clone, Default)]
pub struct CHAAnalysisStats {
    pub total_call_sites: usize,
    pub resolved_calls: usize,
    pub unresolved_calls: usize,
    pub virtual_calls: usize,
    pub static_calls: usize,
    pub constructor_calls: usize,
    pub function_calls: usize,
    pub method_calls: usize,
    pub resolution_time_ms: u64,
}

impl EnhancedClassInfo {
    /// 创建新的增强类信息
    pub fn new(
        id: Uuid,
        name: String,
        file_path: PathBuf,
        line_start: usize,
        line_end: usize,
        namespace: String,
        language: String,
        class_type: ClassType,
    ) -> Self {
        Self {
            id,
            name,
            file_path,
            line_start,
            line_end,
            namespace,
            language,
            parent_class: None,
            base_classes: Vec::new(),
            virtual_methods: Vec::new(),
            method_signatures: HashMap::new(),
            inheritance_chain: Vec::new(),
            implemented_interfaces: Vec::new(),
            class_type,
            member_functions: Vec::new(),
            member_variables: Vec::new(),
        }
    }

    /// 添加父类
    pub fn add_parent_class(&mut self, parent: String) {
        self.parent_class = Some(parent.clone());
        self.inheritance_chain.push(parent);
    }

    /// 添加基类
    pub fn add_base_class(&mut self, base: String) {
        if !self.base_classes.contains(&base) {
            self.base_classes.push(base);
        }
    }

    /// 添加虚方法
    pub fn add_virtual_method(&mut self, method_id: Uuid) {
        if !self.virtual_methods.contains(&method_id) {
            self.virtual_methods.push(method_id);
        }
    }

    /// 添加方法签名
    pub fn add_method_signature(&mut self, signature: MethodSignature) {
        self.method_signatures.insert(signature.name.clone(), signature);
    }

    /// 检查是否为虚方法
    pub fn is_virtual_method(&self, method_name: &str) -> bool {
        if let Some(signature) = self.method_signatures.get(method_name) {
            signature.is_virtual
        } else {
            false
        }
    }

    /// 获取方法的完整签名
    pub fn get_method_signature(&self, method_name: &str) -> Option<&MethodSignature> {
        self.method_signatures.get(method_name)
    }
}

impl CallSite {
    /// 创建新的调用点
    pub fn new(
        id: Uuid,
        caller_function: Uuid,
        callee_name: String,
        call_type: CallType,
        line_number: usize,
        file_path: PathBuf,
    ) -> Self {
        Self {
            id,
            caller_function,
            callee_name,
            receiver_type: None,
            receiver_expression: None,
            call_type,
            line_number,
            file_path,
            parameters: Vec::new(),
            context: CallContext::default(),
        }
    }

    /// 设置接收者类型
    pub fn with_receiver_type(mut self, receiver_type: String) -> Self {
        self.receiver_type = Some(receiver_type);
        self
    }

    /// 设置接收者表达式
    pub fn with_receiver_expression(mut self, expression: String) -> Self {
        self.receiver_expression = Some(expression);
        self
    }

    /// 添加参数
    pub fn add_parameter(&mut self, parameter: CallParameter) {
        self.parameters.push(parameter);
    }

    /// 设置上下文
    pub fn with_context(mut self, context: CallContext) -> Self {
        self.context = context;
        self
    }
}

impl CallContext {
    /// 创建默认上下文
    pub fn default() -> Self {
        Self {
            scope_type: ScopeType::Global,
            containing_class: None,
            containing_function: None,
            namespace: None,
        }
    }

    /// 设置包含的类
    pub fn with_containing_class(mut self, class: String) -> Self {
        self.containing_class = Some(class);
        self.scope_type = ScopeType::Class;
        self
    }

    /// 设置包含的函数
    pub fn with_containing_function(mut self, function: String) -> Self {
        self.containing_function = Some(function);
        self.scope_type = ScopeType::Function;
        self
    }

    /// 设置命名空间
    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);
        self
    }
}

impl Default for CallContext {
    fn default() -> Self {
        Self::default()
    }
} 