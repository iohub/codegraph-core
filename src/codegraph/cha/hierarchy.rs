use std::collections::{HashMap, HashSet, VecDeque};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use petgraph::Direction;
use uuid::Uuid;
use tracing::{info, warn, debug};

use super::types::{
    EnhancedClassInfo, InheritanceEdge, InheritanceEdgeType, MethodSignature
};

/// 类层次结构构建器
/// 负责构建和维护类的继承关系图，支持虚方法解析
pub struct ClassHierarchyBuilder {
    /// 类名 -> 类信息映射
    classes: HashMap<String, EnhancedClassInfo>,
    /// 继承关系图
    inheritance_graph: DiGraph<String, InheritanceEdge>,
    /// 类名 -> 节点索引映射
    class_to_node: HashMap<String, NodeIndex>,
    /// 节点索引 -> 类名映射
    node_to_class: HashMap<NodeIndex, String>,
    /// 方法名 -> 实现类映射（用于快速查找）
    method_implementations: HashMap<String, Vec<String>>,
}

impl ClassHierarchyBuilder {
    /// 创建新的类层次结构构建器
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            inheritance_graph: DiGraph::new(),
            class_to_node: HashMap::new(),
            node_to_class: HashMap::new(),
            method_implementations: HashMap::new(),
        }
    }

    /// 构建类层次结构
    pub fn build_hierarchy(&mut self, classes: Vec<EnhancedClassInfo>) -> Result<(), String> {
        info!("Building class hierarchy for {} classes", classes.len());
        
        // 1. 添加所有类到图中
        for class in classes {
            self.add_class(class)?;
        }
        
        // 2. 建立继承关系
        self.establish_inheritance_relationships()?;
        
        // 3. 计算传递闭包（所有基类）
        self.compute_transitive_closure()?;
        
        // 4. 构建方法实现索引
        self.build_method_implementation_index()?;
        
        // 5. 验证层次结构
        self.validate_hierarchy()?;
        
        info!("Class hierarchy built successfully with {} classes", self.classes.len());
        Ok(())
    }

    /// 添加类到层次结构中
    pub fn add_class(&mut self, class: EnhancedClassInfo) -> Result<(), String> {
        let class_name = class.name.clone();
        
        // 检查是否已存在
        if self.classes.contains_key(&class_name) {
            warn!("Class {} already exists, updating", class_name);
        }
        
        // 添加到类映射
        self.classes.insert(class_name.clone(), class);
        
        // 添加到继承图
        let node_index = self.inheritance_graph.add_node(class_name.clone());
        self.class_to_node.insert(class_name.clone(), node_index);
        self.node_to_class.insert(node_index, class_name);
        
        Ok(())
    }

    /// 建立继承关系
    fn establish_inheritance_relationships(&mut self) -> Result<(), String> {
        info!("Establishing inheritance relationships");
        
        for class in self.classes.values() {
            if let Some(parent_name) = &class.parent_class {
                if let Some(parent_node) = self.class_to_node.get(parent_name) {
                    if let Some(child_node) = self.class_to_node.get(&class.name) {
                        // 添加继承边
                        let edge = InheritanceEdge {
                            edge_type: InheritanceEdgeType::Extends,
                            metadata: None,
                        };
                        self.inheritance_graph.add_edge(*parent_node, *child_node, edge);
                        debug!("Added inheritance edge: {} -> {}", parent_name, class.name);
                    } else {
                        warn!("Child class {} not found in graph", class.name);
                    }
                } else {
                    warn!("Parent class {} not found for {}", parent_name, class.name);
                }
            }
            
            // 处理接口实现
            for interface_name in &class.implemented_interfaces {
                if let Some(interface_node) = self.class_to_node.get(interface_name) {
                    if let Some(class_node) = self.class_to_node.get(&class.name) {
                        let edge = InheritanceEdge {
                            edge_type: InheritanceEdgeType::Implements,
                            metadata: None,
                        };
                        self.inheritance_graph.add_edge(*interface_node, *class_node, edge);
                        debug!("Added implementation edge: {} -> {}", interface_name, class.name);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// 计算传递闭包（所有基类）
    fn compute_transitive_closure(&mut self) -> Result<(), String> {
        info!("Computing transitive closure for inheritance");
        
        for class_name in self.classes.keys().cloned().collect::<Vec<_>>() {
            if let Some(class) = self.classes.get_mut(&class_name) {
                let base_classes = self.get_all_base_classes(&class_name);
                class.base_classes = base_classes;
                
                // 构建继承链
                let inheritance_chain = self.build_inheritance_chain(&class_name);
                class.inheritance_chain = inheritance_chain;
            }
        }
        
        Ok(())
    }

    /// 获取类的所有基类（传递闭包）
    pub fn get_all_base_classes(&self, class_name: &str) -> Vec<String> {
        let mut base_classes = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        if let Some(&start_node) = self.class_to_node.get(class_name) {
            queue.push_back(start_node);
            visited.insert(start_node);
            
            while let Some(current_node) = queue.pop_front() {
                // 遍历所有入边（父类）
                for edge in self.inheritance_graph.edges_directed(current_node, Direction::Incoming) {
                    let parent_node = edge.source();
                    if !visited.contains(&parent_node) {
                        visited.insert(parent_node);
                        queue.push_back(parent_node);
                        
                        if let Some(parent_name) = self.node_to_class.get(&parent_node) {
                            base_classes.insert(parent_name.clone());
                        }
                    }
                }
            }
        }
        
        base_classes.into_iter().collect()
    }

    /// 构建继承链
    fn build_inheritance_chain(&self, class_name: &str) -> Vec<String> {
        let mut chain = Vec::new();
        let mut current_name = class_name;
        
        while let Some(class) = self.classes.get(current_name) {
            if let Some(parent_name) = &class.parent_class {
                chain.push(parent_name.clone());
                current_name = parent_name;
            } else {
                break;
            }
        }
        
        chain.reverse(); // 从根到叶的顺序
        chain
    }

    /// 构建方法实现索引
    fn build_method_implementation_index(&mut self) -> Result<(), String> {
        info!("Building method implementation index");
        
        for class in self.classes.values() {
            for method_name in class.method_signatures.keys() {
                self.method_implementations
                    .entry(method_name.clone())
                    .or_default()
                    .push(class.name.clone());
            }
        }
        
        Ok(())
    }

    /// 验证层次结构
    fn validate_hierarchy(&self) -> Result<(), String> {
        info!("Validating class hierarchy");
        
        // 检查是否有循环依赖
        if let Err(cycle) = toposort(&self.inheritance_graph, None) {
            return Err(format!("Circular dependency detected: {:?}", cycle));
        }
        
        // 检查孤立节点
        let isolated_nodes: Vec<_> = self.inheritance_graph
            .node_indices()
            .filter(|&node| {
                self.inheritance_graph.edges_directed(node, Direction::Incoming).count() == 0 &&
                self.inheritance_graph.edges_directed(node, Direction::Outgoing).count() == 0
            })
            .collect();
        
        if !isolated_nodes.is_empty() {
            warn!("Found {} isolated classes: {:?}", 
                  isolated_nodes.len(), 
                  isolated_nodes.iter()
                      .filter_map(|&node| self.node_to_class.get(&node))
                      .collect::<Vec<_>>());
        }
        
        Ok(())
    }

    /// 获取类的子类型
    pub fn get_subtypes(&self, class_name: &str) -> Vec<String> {
        let mut subtypes = Vec::new();
        
        if let Some(&class_node) = self.class_to_node.get(class_name) {
            // 遍历所有出边（子类）
            for edge in self.inheritance_graph.edges_directed(class_node, Direction::Outgoing) {
                let child_node = edge.target();
                if let Some(child_name) = self.node_to_class.get(&child_node) {
                    subtypes.push(child_name.clone());
                }
            }
        }
        
        subtypes
    }

    /// 获取类的超类型
    pub fn get_supertypes(&self, class_name: &str) -> Vec<String> {
        let mut supertypes = Vec::new();
        
        if let Some(&class_node) = self.class_to_node.get(class_name) {
            // 遍历所有入边（父类）
            for edge in self.inheritance_graph.edges_directed(class_node, Direction::Incoming) {
                let parent_node = edge.source();
                if let Some(parent_name) = self.node_to_class.get(&parent_node) {
                    supertypes.push(parent_name.clone());
                }
            }
        }
        
        supertypes
    }

    /// 查找方法的实现
    pub fn find_method_implementations(&self, method_name: &str, receiver_type: &str) -> Vec<Uuid> {
        let mut implementations = Vec::new();
        
        // 获取接收者类型及其所有子类型
        let mut candidate_types = vec![receiver_type.to_string()];
        candidate_types.extend(self.get_subtypes(receiver_type));
        
        for class_type in candidate_types {
            if let Some(class) = self.classes.get(&class_type) {
                if let Some(signature) = class.method_signatures.get(method_name) {
                    implementations.push(signature.id);
                }
            }
        }
        
        implementations
    }

    /// 解析虚方法调用
    pub fn resolve_virtual_call(&self, method_name: &str, receiver_type: &str) -> Vec<Uuid> {
        // 虚方法调用需要考虑继承层次
        self.find_method_implementations(method_name, receiver_type)
    }

    /// 根据类名查找类
    pub fn find_class_by_name(&self, class_name: &str) -> Option<&EnhancedClassInfo> {
        self.classes.get(class_name)
    }

    /// 获取所有类
    pub fn get_all_classes(&self) -> Vec<&EnhancedClassInfo> {
        self.classes.values().collect()
    }

    /// 获取继承图
    pub fn get_inheritance_graph(&self) -> &DiGraph<String, InheritanceEdge> {
        &self.inheritance_graph
    }

    /// 检查两个类之间是否存在继承关系
    pub fn is_subtype_of(&self, potential_subtype: &str, potential_supertype: &str) -> bool {
        if let Some(&subtype_node) = self.class_to_node.get(potential_subtype) {
            if let Some(&supertype_node) = self.class_to_node.get(potential_supertype) {
                // 使用BFS检查是否存在从supertype到subtype的路径
                let mut visited = HashSet::new();
                let mut queue = VecDeque::new();
                
                queue.push_back(supertype_node);
                visited.insert(supertype_node);
                
                while let Some(current_node) = queue.pop_front() {
                    if current_node == subtype_node {
                        return true;
                    }
                    
                    for edge in self.inheritance_graph.edges_directed(current_node, Direction::Outgoing) {
                        let child_node = edge.target();
                        if !visited.contains(&child_node) {
                            visited.insert(child_node);
                            queue.push_back(child_node);
                        }
                    }
                }
            }
        }
        
        false
    }

    /// 获取类的深度（距离根类的层数）
    pub fn get_class_depth(&self, class_name: &str) -> usize {
        if let Some(class) = self.classes.get(class_name) {
            class.inheritance_chain.len()
        } else {
            0
        }
    }

    /// 获取层次结构的统计信息
    pub fn get_hierarchy_stats(&self) -> HierarchyStats {
        let total_classes = self.classes.len();
        let mut max_depth = 0;
        let mut total_inheritance_edges = 0;
        let mut classes_with_parents = 0;
        
        for class in self.classes.values() {
            max_depth = max_depth.max(class.inheritance_chain.len());
            if class.parent_class.is_some() {
                classes_with_parents += 1;
            }
        }
        
        total_inheritance_edges = self.inheritance_graph.edge_count();
        
        HierarchyStats {
            total_classes,
            max_depth,
            total_inheritance_edges,
            classes_with_parents,
        }
    }
}

/// 层次结构统计信息
#[derive(Debug, Clone)]
pub struct HierarchyStats {
    pub total_classes: usize,
    pub max_depth: usize,
    pub total_inheritance_edges: usize,
    pub classes_with_parents: usize,
}

impl Default for ClassHierarchyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_basic_hierarchy_building() {
        let mut builder = ClassHierarchyBuilder::new();
        
        // 创建测试类
        let base_class = EnhancedClassInfo::new(
            Uuid::new_v4(),
            "BaseClass".to_string(),
            PathBuf::from("test.rs"),
            1, 10,
            "test".to_string(),
            "rust".to_string(),
            ClassType::Class,
        );
        
        let derived_class = EnhancedClassInfo::new(
            Uuid::new_v4(),
            "DerivedClass".to_string(),
            PathBuf::from("test.rs"),
            12, 20,
            "test".to_string(),
            "rust".to_string(),
            ClassType::Class,
        );
        
        // 设置继承关系
        let mut derived = derived_class.clone();
        derived.add_parent_class("BaseClass".to_string());
        
        // 构建层次结构
        let classes = vec![base_class, derived];
        assert!(builder.build_hierarchy(classes).is_ok());
        
        // 验证继承关系
        let subtypes = builder.get_subtypes("BaseClass");
        assert_eq!(subtypes.len(), 1);
        assert_eq!(subtypes[0], "DerivedClass");
        
        let supertypes = builder.get_supertypes("DerivedClass");
        assert_eq!(supertypes.len(), 1);
        assert_eq!(supertypes[0], "BaseClass");
    }
} 