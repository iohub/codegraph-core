use std::collections::HashMap;
use uuid::Uuid;
use tracing::{info, debug};

use super::types::{SimpleClassInfo, SimpleCallSite, SimpleMethodResolution};
use crate::codegraph::types::{FunctionInfo, CallRelation, PetCodeGraph};

/// 简化的类层次分析（CHA）实现
pub struct SimpleCHA {
    /// 类信息映射
    classes: HashMap<String, SimpleClassInfo>,
    /// 调用点列表
    call_sites: Vec<SimpleCallSite>,
    /// 已解析的调用映射
    resolved_calls: HashMap<Uuid, Vec<Uuid>>,
    /// 函数信息映射
    functions: HashMap<Uuid, FunctionInfo>,
}

impl SimpleCHA {
    /// 创建新的简化CHA分析器
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            call_sites: Vec::new(),
            resolved_calls: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// 添加类信息
    pub fn add_class(&mut self, class: SimpleClassInfo) {
        self.classes.insert(class.name.clone(), class);
    }

    /// 添加函数信息
    pub fn add_function(&mut self, function: FunctionInfo) {
        self.functions.insert(function.id, function);
    }

    /// 添加调用点
    pub fn add_call_site(&mut self, call_site: SimpleCallSite) {
        self.call_sites.push(call_site);
    }

    /// 执行CHA分析
    pub fn analyze(&mut self) -> Result<(), String> {
        info!("Starting simple CHA analysis for {} call sites", self.call_sites.len());
        
        for call_site in &self.call_sites {
            self.resolve_call_site(call_site)?;
        }
        
        info!("Simple CHA analysis completed");
        Ok(())
    }

    /// 解析单个调用点
    fn resolve_call_site(&mut self, call_site: &SimpleCallSite) -> Result<(), String> {
        debug!("Resolving call site: {} at line {}", 
               call_site.callee_name, call_site.line_number);
        
        let mut target_methods = Vec::new();
        
        // 1. 如果有接收者类型，使用CHA解析
        if let Some(receiver_type) = &call_site.receiver_type {
            target_methods = self.resolve_method_call_with_cha(&call_site.callee_name, receiver_type);
        } else {
            // 2. 否则使用简单的名称匹配
            target_methods = self.resolve_method_call_by_name(&call_site.callee_name);
        }
        
        // 存储解析结果
        if !target_methods.is_empty() {
            self.resolved_calls.insert(call_site.id, target_methods);
            debug!("Resolved call to {} methods", target_methods.len());
        }
        
        Ok(())
    }

    /// 使用CHA解析方法调用
    fn resolve_method_call_with_cha(&self, method_name: &str, receiver_type: &str) -> Vec<Uuid> {
        let mut target_methods = Vec::new();
        
        // 获取接收者类型及其所有子类型
        let mut candidate_types = vec![receiver_type.to_string()];
        candidate_types.extend(self.get_subtypes(receiver_type));
        
        for class_type in candidate_types {
            if let Some(class) = self.classes.get(&class_type) {
                // 查找匹配的方法
                for &method_id in &class.methods {
                    if let Some(function) = self.functions.get(&method_id) {
                        if function.name == method_name {
                            target_methods.push(method_id);
                        }
                    }
                }
            }
        }
        
        target_methods
    }

    /// 使用名称匹配解析方法调用
    fn resolve_method_call_by_name(&self, method_name: &str) -> Vec<Uuid> {
        let mut target_methods = Vec::new();
        
        for function in self.functions.values() {
            if function.name == method_name {
                target_methods.push(function.id);
            }
        }
        
        target_methods
    }

    /// 获取类的子类型
    fn get_subtypes(&self, class_name: &str) -> Vec<String> {
        let mut subtypes = Vec::new();
        
        for class in self.classes.values() {
            if let Some(parent) = &class.parent_class {
                if parent == class_name {
                    subtypes.push(class.name.clone());
                    // 递归获取子类型的子类型
                    subtypes.extend(self.get_subtypes(&class.name));
                }
            }
        }
        
        subtypes
    }

    /// 构建调用图
    pub fn build_call_graph(&self) -> Result<PetCodeGraph, String> {
        let mut call_graph = PetCodeGraph::new();
        
        // 添加所有函数到图中
        for function in self.functions.values() {
            call_graph.add_function(function.clone());
        }
        
        // 添加调用关系
        for (call_site_id, target_methods) in &self.resolved_calls {
            if let Some(call_site) = self.get_call_site(*call_site_id) {
                for target_method_id in target_methods {
                    if let Some(target_function) = self.functions.get(target_method_id) {
                        let relation = CallRelation {
                            caller_id: call_site.caller_function,
                            callee_id: *target_method_id,
                            caller_name: "".to_string(), // 将由add_call_relation填充
                            callee_name: call_site.callee_name.clone(),
                            caller_file: call_site.file_path.clone(),
                            callee_file: target_function.file_path.clone(),
                            line_number: call_site.line_number,
                            is_resolved: true,
                        };
                        
                        if let Err(e) = call_graph.add_call_relation(relation) {
                            tracing::warn!("Failed to add call relation: {}", e);
                        }
                    }
                }
            }
        }
        
        // 更新统计信息
        call_graph.update_stats();
        
        Ok(call_graph)
    }

    /// 获取调用点
    fn get_call_site(&self, call_site_id: Uuid) -> Option<&SimpleCallSite> {
        self.call_sites.iter().find(|cs| cs.id == call_site_id)
    }

    /// 获取分析统计信息
    pub fn get_stats(&self) -> SimpleCHAStats {
        let total_call_sites = self.call_sites.len();
        let resolved_calls = self.resolved_calls.len();
        let unresolved_calls = total_call_sites - resolved_calls;
        
        SimpleCHAStats {
            total_call_sites,
            resolved_calls,
            unresolved_calls,
            total_classes: self.classes.len(),
            total_functions: self.functions.len(),
        }
    }
}

/// 简化CHA统计信息
#[derive(Debug, Clone)]
pub struct SimpleCHAStats {
    pub total_call_sites: usize,
    pub resolved_calls: usize,
    pub unresolved_calls: usize,
    pub total_classes: usize,
    pub total_functions: usize,
}

impl Default for SimpleCHA {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_simple_cha_creation() {
        let cha = SimpleCHA::new();
        assert_eq!(cha.classes.len(), 0);
        assert_eq!(cha.call_sites.len(), 0);
    }

    #[test]
    fn test_class_addition() {
        let mut cha = SimpleCHA::new();
        let class = SimpleClassInfo::new(
            Uuid::new_v4(),
            "TestClass".to_string(),
            PathBuf::from("test.rs"),
            1, 10,
            "test".to_string(),
            "rust".to_string(),
        );
        
        cha.add_class(class);
        assert_eq!(cha.classes.len(), 1);
    }
} 