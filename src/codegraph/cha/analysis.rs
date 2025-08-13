use std::collections::HashMap;
use uuid::Uuid;
use tracing::{info, debug, warn};
use super::types::{
    CallSite, CallType, CHAAnalysisStats
};
use super::hierarchy::ClassHierarchyBuilder;
use crate::codegraph::types::{FunctionInfo, PetCodeGraph, CallRelation};

/// 解析结果
#[derive(Debug, Clone)]
enum ResolutionResult {
    Resolved(Vec<Uuid>),
    Unresolved,
}

/// 方法解析结果
#[derive(Debug, Clone)]
pub struct MethodResolutionResult {
    pub call_site_id: Uuid,
    pub target_methods: Vec<Uuid>,
    pub resolution_type: ResolutionType,
    pub confidence: f64,
}

/// 解析类型
#[derive(Debug, Clone)]
pub enum ResolutionType {
    Direct,
    Virtual,
    Unresolved,
}

/// 类层次分析（CHA）核心分析器
/// 实现CHA算法来解析函数调用，特别是虚方法调用
pub struct ClassHierarchyAnalysis {
    /// 类层次结构构建器
    class_hierarchy: ClassHierarchyBuilder,
    /// 调用点列表
    call_sites: Vec<CallSite>,
    /// 已解析的调用映射：调用点ID -> 目标方法列表
    resolved_calls: HashMap<Uuid, Vec<Uuid>>,
    /// 分析统计信息
    stats: CHAAnalysisStats,
}

impl ClassHierarchyAnalysis {
    /// 创建新的CHA分析器
    pub fn new(
        class_hierarchy: ClassHierarchyBuilder,
        call_sites: Vec<CallSite>,
    ) -> Self {
        Self {
            class_hierarchy,
            call_sites,
            resolved_calls: HashMap::new(),
            stats: CHAAnalysisStats::default(),
        }
    }

    /// 执行CHA分析
    pub fn analyze(&mut self) -> Result<(), String> {
        let start_time = std::time::Instant::now();
        info!("Starting CHA analysis for {} call sites", self.call_sites.len());
        
        // 初始化统计信息
        self.stats.total_call_sites = self.call_sites.len();
        
        // 收集所有解析结果，避免借用冲突
        let mut resolution_results = Vec::new();
        
        // 分析每个调用点
        for call_site in &self.call_sites {
            let result = match call_site.call_type {
                CallType::Static => {
                    self.resolve_static_call_internal(call_site)?
                },
                CallType::Virtual => {
                    self.resolve_virtual_call_internal(call_site)?
                },
                CallType::Constructor => {
                    self.resolve_constructor_call_internal(call_site)?
                },
                CallType::Function => {
                    self.resolve_function_call_internal(call_site)?
                },
                CallType::Method => {
                    self.resolve_method_call_internal(call_site)?
                },
                CallType::TraitMethod => {
                    self.resolve_trait_method_call_internal(call_site)?
                },
                CallType::InterfaceMethod => {
                    self.resolve_interface_method_call_internal(call_site)?
                },
            };
            resolution_results.push((call_site.id, result));
        }
        
        // 应用解析结果
        for (call_site_id, result) in resolution_results {
            match result {
                ResolutionResult::Resolved(target_methods) => {
                    self.resolved_calls.insert(call_site_id, target_methods);
                    self.stats.resolved_calls += 1;
                },
                ResolutionResult::Unresolved => {
                    self.stats.unresolved_calls += 1;
                },
            }
        }
        
        // 更新统计信息
        self.stats.resolution_time_ms = start_time.elapsed().as_millis() as u64;
        self.update_stats();
        
        info!("CHA analysis completed in {}ms", self.stats.resolution_time_ms);
        info!("Resolved: {}, Unresolved: {}", 
              self.stats.resolved_calls, self.stats.unresolved_calls);
        
        Ok(())
    }

    /// 内部解析静态方法调用（不需要可变借用）
    fn resolve_static_call_internal(&self, call_site: &CallSite) -> Result<ResolutionResult, String> {
        debug!("Resolving static call: {} at line {}", 
               call_site.callee_name, call_site.line_number);
        
        // 静态调用通常是直接的，不需要考虑继承
        if let Some(target_methods) = self.find_static_method_implementations(&call_site.callee_name) {
            Ok(ResolutionResult::Resolved(target_methods))
        } else {
            Ok(ResolutionResult::Unresolved)
        }
    }

    /// 内部解析虚方法调用（不需要可变借用）
    fn resolve_virtual_call_internal(&self, call_site: &CallSite) -> Result<ResolutionResult, String> {
        debug!("Resolving virtual call: {} at line {}", 
               call_site.callee_name, call_site.line_number);
        
        let receiver_type = call_site.receiver_type.as_ref()
            .ok_or_else(|| format!("No receiver type for virtual call at line {}", call_site.line_number))?;
        
        // 获取接收者类型的继承锥（subtree）
        let possible_types = self.class_hierarchy.get_subtypes(receiver_type);
        let mut all_types = vec![receiver_type.clone()];
        all_types.extend(possible_types);
        
        // 在所有可能的类型中查找方法实现
        let mut target_methods = Vec::new();
        for class_type in all_types {
            let methods = self.class_hierarchy.find_method_implementations(
                &call_site.callee_name, 
                &class_type
            );
            target_methods.extend(methods);
        }
        
        // 去重
        target_methods.sort();
        target_methods.dedup();
        
        if !target_methods.is_empty() {
            Ok(ResolutionResult::Resolved(target_methods))
        } else {
            Ok(ResolutionResult::Unresolved)
        }
    }

    /// 内部解析构造函数调用（不需要可变借用）
    fn resolve_constructor_call_internal(&self, call_site: &CallSite) -> Result<ResolutionResult, String> {
        debug!("Resolving constructor call: {} at line {}", 
               call_site.callee_name, call_site.line_number);
        
        // 构造函数调用通常是直接的
        if let Some(class) = self.class_hierarchy.find_class_by_name(&call_site.callee_name) {
            // 查找构造函数方法
            if let Some(constructor) = class.method_signatures.get(&call_site.callee_name) {
                Ok(ResolutionResult::Resolved(vec![constructor.id]))
            } else {
                Ok(ResolutionResult::Unresolved)
            }
        } else {
            Ok(ResolutionResult::Unresolved)
        }
    }

    /// 内部解析自由函数调用（不需要可变借用）
    fn resolve_function_call_internal(&self, call_site: &CallSite) -> Result<ResolutionResult, String> {
        debug!("Resolving function call: {} at line {}", 
               call_site.callee_name, call_site.line_number);
        
        // 自由函数调用通常是直接的
        if let Some(target_methods) = self.find_function_implementations(&call_site.callee_name) {
            Ok(ResolutionResult::Resolved(target_methods))
        } else {
            Ok(ResolutionResult::Unresolved)
        }
    }

    /// 内部解析实例方法调用（不需要可变借用）
    fn resolve_method_call_internal(&self, call_site: &CallSite) -> Result<ResolutionResult, String> {
        debug!("Resolving method call: {} at line {}", 
               call_site.callee_name, call_site.line_number);
        
        if let Some(_receiver_type) = &call_site.receiver_type {
            // 使用CHA进行方法解析
            self.resolve_virtual_call_internal(call_site)
        } else {
            // 回退到函数调用解析
            self.resolve_function_call_internal(call_site)
        }
    }

    /// 内部解析Trait方法调用（不需要可变借用）
    fn resolve_trait_method_call_internal(&self, call_site: &CallSite) -> Result<ResolutionResult, String> {
        debug!("Resolving trait method call: {} at line {}", 
               call_site.callee_name, call_site.line_number);
        
        // Trait方法调用需要特殊处理
        // 这里可以扩展为更复杂的trait解析逻辑
        self.resolve_virtual_call_internal(call_site)
    }

    /// 内部解析接口方法调用（不需要可变借用）
    fn resolve_interface_method_call_internal(&self, call_site: &CallSite) -> Result<ResolutionResult, String> {
        debug!("Resolving interface method call: {} at line {}", 
               call_site.callee_name, call_site.line_number);
        
        // 接口方法调用类似于虚方法调用
        self.resolve_virtual_call_internal(call_site)
    }

    /// 查找静态方法实现
    fn find_static_method_implementations(&self, method_name: &str) -> Option<Vec<Uuid>> {
        let mut implementations = Vec::new();
        
        for class in self.class_hierarchy.get_all_classes() {
            if let Some(signature) = class.method_signatures.get(method_name) {
                if signature.is_static {
                    implementations.push(signature.id);
                }
            }
        }
        
        if implementations.is_empty() {
            None
        } else {
            Some(implementations)
        }
    }

    /// 查找函数实现
    fn find_function_implementations(&self, function_name: &str) -> Option<Vec<Uuid>> {
        // 这里需要与现有的函数注册表集成
        // 暂时返回None，表示未实现
        None
    }

    /// 更新统计信息
    fn update_stats(&mut self) {
        // 统计信息已经在各个解析方法中更新
        debug!("CHA Analysis Stats: {:?}", self.stats);
    }

    /// 获取方法解析结果
    pub fn get_method_resolution_result(&self, call_site_id: &Uuid) -> Option<MethodResolutionResult> {
        if let Some(target_methods) = self.resolved_calls.get(call_site_id) {
            let resolution_type = if target_methods.len() == 1 {
                ResolutionType::Direct
            } else if target_methods.len() > 1 {
                ResolutionType::Virtual
            } else {
                ResolutionType::Unresolved
            };
            
            let confidence = if target_methods.is_empty() {
                0.0
            } else if target_methods.len() == 1 {
                1.0
            } else {
                0.8 // 虚方法调用的置信度
            };
            
            Some(MethodResolutionResult {
                call_site_id: *call_site_id,
                target_methods: target_methods.clone(),
                resolution_type,
                confidence,
            })
        } else {
            None
        }
    }

    /// 获取所有已解析的调用
    pub fn get_resolved_calls(&self) -> &HashMap<Uuid, Vec<Uuid>> {
        &self.resolved_calls
    }

    /// 获取调用点
    pub fn get_call_site(&self, call_site_id: &Uuid) -> Option<&CallSite> {
        self.call_sites.iter().find(|cs| cs.id == *call_site_id)
    }

    /// 获取所有调用点
    pub fn get_all_call_sites(&self) -> &[CallSite] {
        &self.call_sites
    }

    /// 获取分析统计信息
    pub fn get_stats(&self) -> &CHAAnalysisStats {
        &self.stats
    }

    /// 构建调用图
    pub fn build_call_graph(&self, functions: &[FunctionInfo]) -> Result<PetCodeGraph, String> {
        let mut call_graph = PetCodeGraph::new();
        
        // 添加所有函数到图中
        for function in functions {
            call_graph.add_function(function.clone());
        }
        
        // 添加调用关系
        for (call_site_id, target_methods) in &self.resolved_calls {
            if let Some(call_site) = self.get_call_site(call_site_id) {
                for target_method_id in target_methods {
                    // 查找目标函数信息
                    if let Some(target_function) = functions.iter().find(|f| f.id == *target_method_id) {
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
                            warn!("Failed to add call relation: {}", e);
                        }
                    }
                }
            }
        }
        
        // 更新统计信息
        call_graph.update_stats();
        
        Ok(call_graph)
    }

    /// 验证分析结果
    pub fn validate_results(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        
        // 检查是否有未解析的调用
        let unresolved_count = self.call_sites.len() - self.resolved_calls.len();
        if unresolved_count > 0 {
            errors.push(format!("{} call sites remain unresolved", unresolved_count));
        }
        
        // 检查是否有空的解析结果
        for (call_site_id, target_methods) in &self.resolved_calls {
            if target_methods.is_empty() {
                if let Some(call_site) = self.get_call_site(call_site_id) {
                    errors.push(format!("Call site at line {} has empty resolution", call_site.line_number));
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("Validation errors: {}", errors.join("; ")))
        }
    }

    /// 获取分析摘要
    pub fn get_analysis_summary(&self) -> String {
        format!(
            "CHA Analysis Summary:\n\
             Total Call Sites: {}\n\
             Resolved Calls: {}\n\
             Unresolved Calls: {}\n\
             Virtual Calls: {}\n\
             Static Calls: {}\n\
             Constructor Calls: {}\n\
             Function Calls: {}\n\
             Method Calls: {}\n\
             Resolution Time: {}ms",
            self.stats.total_call_sites,
            self.stats.resolved_calls,
            self.stats.unresolved_calls,
            self.stats.virtual_calls,
            self.stats.static_calls,
            self.stats.constructor_calls,
            self.stats.function_calls,
            self.stats.method_calls,
            self.stats.resolution_time_ms
        )
    }
}

impl Default for ClassHierarchyAnalysis {
    fn default() -> Self {
        Self {
            class_hierarchy: ClassHierarchyBuilder::new(),
            call_sites: Vec::new(),
            resolved_calls: HashMap::new(),
            stats: CHAAnalysisStats::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_cha_analysis_creation() {
        let analysis = ClassHierarchyAnalysis::default();
        assert_eq!(analysis.call_sites.len(), 0);
        assert_eq!(analysis.resolved_calls.len(), 0);
    }

    #[test]
    fn test_analysis_summary() {
        let analysis = ClassHierarchyAnalysis::default();
        let summary = analysis.get_analysis_summary();
        assert!(summary.contains("CHA Analysis Summary"));
        assert!(summary.contains("Total Call Sites: 0"));
    }
} 