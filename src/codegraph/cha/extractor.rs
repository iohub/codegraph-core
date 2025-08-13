use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use tracing::{info, warn, debug};

use super::types::{
    CallSite, CallType, CallParameter, CallContext, ScopeType, MethodSignature
};
use super::hierarchy::ClassHierarchyBuilder;
use crate::codegraph::treesitter::{TreeSitterParser, AstSymbolInstanceArc};
use crate::codegraph::types::{FunctionInfo, ClassInfo};

/// 调用点提取器
/// 负责从AST中提取调用点信息，分析调用类型和上下文
pub struct CallSiteExtractor {
    /// Tree-sitter解析器
    ts_parser: TreeSitterParser,
    /// 类层次结构构建器
    class_hierarchy: ClassHierarchyBuilder,
    /// 函数信息缓存
    function_cache: HashMap<Uuid, FunctionInfo>,
    /// 类信息缓存
    class_cache: HashMap<Uuid, ClassInfo>,
}

impl CallSiteExtractor {
    /// 创建新的调用点提取器
    pub fn new(
        ts_parser: &TreeSitterParser,
        class_hierarchy: &ClassHierarchyBuilder,
    ) -> Self {
        Self {
            ts_parser: ts_parser.clone(),
            class_hierarchy: class_hierarchy.clone(),
            function_cache: HashMap::new(),
            class_cache: HashMap::new(),
        }
    }

    /// 提取文件中的所有调用点
    pub fn extract_call_sites(&mut self, file_path: &PathBuf) -> Result<Vec<CallSite>, String> {
        info!("Extracting call sites from file: {}", file_path.display());
        
        let mut call_sites = Vec::new();
        
        // 使用Tree-sitter解析文件
        match self.ts_parser.parse_file(file_path) {
            Ok(symbols) => {
                for symbol in symbols {
                    let symbol_guard = symbol.read();
                    let symbol_ref = symbol_guard.as_ref();
                    
                    // 检查是否为函数调用
                    if symbol_ref.symbol_type() == crate::codegraph::treesitter::structs::SymbolType::FunctionCall {
                        if let Some(call_site) = self.analyze_function_call(symbol_ref, file_path)? {
                            call_sites.push(call_site);
                        }
                    }
                }
            },
            Err(e) => {
                warn!("Failed to parse file {} for call site extraction: {:?}", file_path.display(), e);
                return Err(format!("Failed to parse file: {}", e));
            }
        }
        
        info!("Extracted {} call sites from {}", call_sites.len(), file_path.display());
        Ok(call_sites)
    }

    /// 分析函数调用并创建调用点
    fn analyze_function_call(
        &self,
        symbol: &dyn crate::codegraph::treesitter::ast_instance_structs::AstSymbolInstance,
        file_path: &PathBuf,
    ) -> Result<Option<CallSite>, String> {
        let call_name = symbol.name();
        let call_line = symbol.full_range().start_point.row + 1;
        
        // 查找调用者函数
        let caller_function = self.find_caller_function(file_path, call_line)?;
        if caller_function.is_none() {
            debug!("No caller function found for call at line {}", call_line);
            return Ok(None);
        }
        
        let caller = caller_function.unwrap();
        
        // 分析调用类型
        let call_type = self.classify_call_type(&call_name, &caller, symbol)?;
        
        // 分析接收者类型
        let receiver_type = self.analyze_receiver_type(symbol, &caller)?;
        
        // 分析调用参数
        let parameters = self.extract_call_parameters(symbol)?;
        
        // 分析调用上下文
        let context = self.analyze_call_context(&caller, symbol)?;
        
        // 创建调用点
        let call_site = CallSite::new(
            Uuid::new_v4(),
            caller.id,
            call_name.to_string(),
            call_type,
            call_line,
            file_path.clone(),
        )
        .with_context(context);
        
        // 设置接收者类型（如果有）
        let call_site = if let Some(recv_type) = receiver_type {
            call_site.with_receiver_type(recv_type)
        } else {
            call_site
        };
        
        // 添加参数
        let mut call_site = call_site;
        for param in parameters {
            call_site.add_parameter(param);
        }
        
        Ok(Some(call_site))
    }

    /// 查找调用者函数
    fn find_caller_function(
        &self,
        file_path: &PathBuf,
        call_line: usize,
    ) -> Result<Option<FunctionInfo>, String> {
        // 在函数缓存中查找包含调用行的函数
        for function in self.function_cache.values() {
            if function.file_path == *file_path &&
               call_line >= function.line_start &&
               call_line <= function.line_end {
                return Ok(Some(function.clone()));
            }
        }
        
        Ok(None)
    }

    /// 分类调用类型
    fn classify_call_type(
        &self,
        call_name: &str,
        caller: &FunctionInfo,
        symbol: &crate::codegraph::treesitter::structs::AstSymbolInstance,
    ) -> Result<CallType, String> {
        // 分析AST结构来确定调用类型
        let ast_node = symbol.ast_node();
        
        // 检查是否为静态调用
        if self.is_static_call(ast_node) {
            return Ok(CallType::Static);
        }
        
        // 检查是否为构造函数调用
        if self.is_constructor_call(call_name, ast_node) {
            return Ok(CallType::Constructor);
        }
        
        // 检查是否为虚方法调用
        if self.is_virtual_call(call_name, caller, ast_node) {
            return Ok(CallType::Virtual);
        }
        
        // 检查是否为trait方法调用（Rust）
        if caller.language == "rust" && self.is_trait_method_call(call_name, ast_node) {
            return Ok(CallType::TraitMethod);
        }
        
        // 检查是否为接口方法调用（Java）
        if caller.language == "java" && self.is_interface_method_call(call_name, ast_node) {
            return Ok(CallType::InterfaceMethod);
        }
        
        // 检查是否为实例方法调用
        if self.is_instance_method_call(ast_node) {
            return Ok(CallType::Method);
        }
        
        // 默认为自由函数调用
        Ok(CallType::Function)
    }

    /// 分析接收者类型
    fn analyze_receiver_type(
        &self,
        symbol: &crate::codegraph::treesitter::structs::AstSymbolInstance,
        caller: &FunctionInfo,
    ) -> Result<Option<String>, String> {
        let ast_node = symbol.ast_node();
        
        // 尝试从AST中提取接收者表达式
        if let Some(receiver_expr) = self.extract_receiver_expression(ast_node) {
            // 分析接收者表达式的类型
            if let Some(type_info) = self.infer_expression_type(&receiver_expr, caller) {
                return Ok(Some(type_info));
            }
        }
        
        Ok(None)
    }

    /// 提取调用参数
    fn extract_call_parameters(
        &self,
        symbol: &crate::codegraph::treesitter::structs::AstSymbolInstance,
    ) -> Result<Vec<CallParameter>, String> {
        let mut parameters = Vec::new();
        let ast_node = symbol.ast_node();
        
        // 从AST中提取参数列表
        if let Some(args) = self.extract_argument_list(ast_node) {
            for arg in args {
                let param = CallParameter {
                    expression: arg.expression.clone(),
                    inferred_type: arg.inferred_type.clone(),
                    is_literal: self.is_literal_expression(&arg.expression),
                    is_reference: self.is_reference_expression(&arg.expression),
                };
                parameters.push(param);
            }
        }
        
        Ok(parameters)
    }

    /// 分析调用上下文
    fn analyze_call_context(
        &self,
        caller: &FunctionInfo,
        _symbol: &crate::codegraph::treesitter::structs::AstSymbolInstance,
    ) -> Result<CallContext, String> {
        let mut context = CallContext::default();
        
        // 设置命名空间
        if !caller.namespace.is_empty() {
            context = context.with_namespace(caller.namespace.clone());
        }
        
        // 设置包含的函数
        context = context.with_containing_function(caller.name.clone());
        
        // 检查是否在类中
        if let Some(class_name) = self.find_containing_class(caller) {
            context = context.with_containing_class(class_name);
        }
        
        Ok(context)
    }

    /// 检查是否为静态调用
    fn is_static_call(&self, _ast_node: &crate::codegraph::treesitter::structs::AstNode) -> bool {
        // 实现静态调用检测逻辑
        // 例如：检查是否有类名限定符，或者是否在静态上下文中
        false // 简化实现
    }

    /// 检查是否为构造函数调用
    fn is_constructor_call(
        &self,
        call_name: &str,
        _ast_node: &crate::codegraph::treesitter::structs::AstNode,
    ) -> bool {
        // 检查函数名是否与类名相同（常见的构造函数命名约定）
        // 或者检查是否有new关键字等
        call_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
    }

    /// 检查是否为虚方法调用
    fn is_virtual_call(
        &self,
        call_name: &str,
        caller: &FunctionInfo,
        _ast_node: &crate::codegraph::treesitter::structs::AstNode,
    ) -> bool {
        // 检查调用者是否在类中，以及方法是否为虚方法
        if let Some(class_name) = self.find_containing_class(caller) {
            if let Some(class) = self.class_hierarchy.find_class_by_name(&class_name) {
                return class.is_virtual_method(call_name);
            }
        }
        false
    }

    /// 检查是否为trait方法调用（Rust）
    fn is_trait_method_call(&self, _call_name: &str, _ast_node: &crate::codegraph::treesitter::structs::AstNode) -> bool {
        // 实现Rust trait方法调用检测
        false // 简化实现
    }

    /// 检查是否为接口方法调用（Java）
    fn is_interface_method_call(&self, _call_name: &str, _ast_node: &crate::codegraph::treesitter::structs::AstNode) -> bool {
        // 实现Java接口方法调用检测
        false // 简化实现
    }

    /// 检查是否为实例方法调用
    fn is_instance_method_call(&self, _ast_node: &crate::codegraph::treesitter::structs::AstNode) -> bool {
        // 检查是否有接收者表达式
        false // 简化实现
    }

    /// 提取接收者表达式
    fn extract_receiver_expression(
        &self,
        _ast_node: &crate::codegraph::treesitter::structs::AstNode,
    ) -> Option<String> {
        // 从AST中提取接收者表达式
        // 例如：obj.method() 中的 obj
        None // 简化实现
    }

    /// 推断表达式类型
    fn infer_expression_type(
        &self,
        _expression: &str,
        _caller: &FunctionInfo,
    ) -> Option<String> {
        // 实现类型推断逻辑
        // 可以基于变量声明、函数参数类型等
        None // 简化实现
    }

    /// 提取参数列表
    fn extract_argument_list(
        &self,
        _ast_node: &crate::codegraph::treesitter::structs::AstNode,
    ) -> Option<Vec<ArgumentInfo>> {
        // 从AST中提取函数调用的参数列表
        None // 简化实现
    }

    /// 检查是否为字面量表达式
    fn is_literal_expression(&self, expression: &str) -> bool {
        // 检查是否为数字、字符串、布尔值等字面量
        expression.parse::<i64>().is_ok() ||
        expression.parse::<f64>().is_ok() ||
        expression.starts_with('"') ||
        expression.starts_with('\'') ||
        expression == "true" ||
        expression == "false"
    }

    /// 检查是否为引用表达式
    fn is_reference_expression(&self, expression: &str) -> bool {
        // 检查是否为引用类型（如 &var, *ptr 等）
        expression.starts_with('&') || expression.starts_with('*')
    }

    /// 查找包含的类
    fn find_containing_class(&self, function: &FunctionInfo) -> Option<String> {
        // 基于函数信息查找包含的类
        // 可以通过命名空间、文件路径等推断
        if function.namespace.contains("::") {
            // Rust风格的命名空间
            let parts: Vec<&str> = function.namespace.split("::").collect();
            if parts.len() > 1 {
                return Some(parts[parts.len() - 2].to_string());
            }
        } else if function.namespace.contains('.') {
            // Java风格的包名
            let parts: Vec<&str> = function.namespace.split('.').collect();
            if parts.len() > 1 {
                return Some(parts[parts.len() - 1].to_string());
            }
        }
        
        None
    }

    /// 添加函数到缓存
    pub fn add_function(&mut self, function: FunctionInfo) {
        self.function_cache.insert(function.id, function);
    }

    /// 添加类到缓存
    pub fn add_class(&mut self, class: ClassInfo) {
        self.class_cache.insert(class.id, class);
    }

    /// 获取函数缓存
    pub fn get_function_cache(&self) -> &HashMap<Uuid, FunctionInfo> {
        &self.function_cache
    }

    /// 获取类缓存
    pub fn get_class_cache(&self) -> &HashMap<Uuid, ClassInfo> {
        &self.class_cache
    }
}

/// 参数信息（用于内部处理）
#[derive(Debug, Clone)]
struct ArgumentInfo {
    expression: String,
    inferred_type: Option<String>,
}

impl Default for CallSiteExtractor {
    fn default() -> Self {
        Self {
            ts_parser: TreeSitterParser::new(),
            class_hierarchy: ClassHierarchyBuilder::new(),
            function_cache: HashMap::new(),
            class_cache: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_call_site_extractor_creation() {
        let extractor = CallSiteExtractor::default();
        assert_eq!(extractor.function_cache.len(), 0);
        assert_eq!(extractor.class_cache.len(), 0);
    }

    #[test]
    fn test_literal_expression_detection() {
        let extractor = CallSiteExtractor::default();
        
        assert!(extractor.is_literal_expression("42"));
        assert!(extractor.is_literal_expression("3.14"));
        assert!(extractor.is_literal_expression("\"hello\""));
        assert!(extractor.is_literal_expression("true"));
        assert!(!extractor.is_literal_expression("variable"));
    }

    #[test]
    fn test_reference_expression_detection() {
        let extractor = CallSiteExtractor::default();
        
        assert!(extractor.is_reference_expression("&var"));
        assert!(extractor.is_reference_expression("*ptr"));
        assert!(!extractor.is_reference_expression("var"));
    }
} 