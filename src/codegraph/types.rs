use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use petgraph::visit::EdgeRef;

/// 函数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub id: Uuid,
    pub name: String,
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub namespace: String,
    pub language: String,
    pub signature: Option<String>,
    pub return_type: Option<String>,
    pub parameters: Vec<ParameterInfo>,
}

/// 参数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub type_name: Option<String>,
    pub default_value: Option<String>,
}

/// 调用关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRelation {
    pub caller_id: Uuid,
    pub callee_id: Uuid,
    pub caller_name: String,
    pub callee_name: String,
    pub caller_file: PathBuf,
    pub callee_file: PathBuf,
    pub line_number: usize,
    pub is_resolved: bool,
}

/// 图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub function_info: FunctionInfo,
    pub in_degree: usize,
    pub out_degree: usize,
}

/// 图关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRelation {
    pub source: Uuid,
    pub target: Uuid,
    pub relation_type: RelationType,
}

/// 关系类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    Call,
    Import,
    Inherit,
    Implement,
}

/// 代码图统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraphStats {
    pub total_functions: usize,
    pub total_files: usize,
    pub total_languages: usize,
    pub resolved_calls: usize,
    pub unresolved_calls: usize,
    pub languages: HashMap<String, usize>,
}

impl Default for CodeGraphStats {
    fn default() -> Self {
        Self {
            total_functions: 0,
            total_files: 0,
            total_languages: 0,
            resolved_calls: 0,
            unresolved_calls: 0,
            languages: HashMap::new(),
        }
    }
}

/// 基于petgraph的代码图结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetCodeGraph {
    /// petgraph有向图
    pub graph: DiGraph<FunctionInfo, CallRelation>,
    /// 函数ID -> 节点索引映射
    pub function_to_node: HashMap<Uuid, NodeIndex>,
    /// 节点索引 -> 函数ID映射
    pub node_to_function: HashMap<NodeIndex, Uuid>,
    /// 函数名 -> 函数ID列表（支持重载）
    pub function_names: HashMap<String, Vec<Uuid>>,
    /// 文件路径 -> 函数ID列表
    pub file_functions: HashMap<PathBuf, Vec<Uuid>>,
    /// 统计信息
    pub stats: CodeGraphStats,
}

impl PetCodeGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            function_to_node: HashMap::new(),
            node_to_function: HashMap::new(),
            function_names: HashMap::new(),
            file_functions: HashMap::new(),
            stats: CodeGraphStats::default(),
        }
    }

    /// 添加函数节点
    pub fn add_function(&mut self, function: FunctionInfo) -> NodeIndex {
        let id = function.id;
        let name = function.name.clone();
        let file_path = function.file_path.clone();
        let language = function.language.clone();

        // 添加到petgraph
        let node_index = self.graph.add_node(function.clone());
        
        // 更新映射
        self.function_to_node.insert(id, node_index);
        self.node_to_function.insert(node_index, id);
        
        // 添加到函数名映射
        self.function_names.entry(name.clone()).or_default().push(id);
        
        // 添加到文件映射
        self.file_functions.entry(file_path).or_default().push(id);
        
        // 更新统计信息
        self.stats.total_functions += 1;
        *self.stats.languages.entry(language).or_default() += 1;

        node_index
    }

    /// 添加调用关系边
    pub fn add_call_relation(&mut self, relation: CallRelation) -> Result<(), String> {
        let caller_node = self.function_to_node.get(&relation.caller_id)
            .ok_or_else(|| format!("Caller function {} not found", relation.caller_id))?;
        let callee_node = self.function_to_node.get(&relation.callee_id)
            .ok_or_else(|| format!("Callee function {} not found", relation.callee_id))?;

        // 添加到petgraph
        self.graph.add_edge(*caller_node, *callee_node, relation.clone());
        
        // 更新统计信息
        if relation.is_resolved {
            self.stats.resolved_calls += 1;
        } else {
            self.stats.unresolved_calls += 1;
        }

        Ok(())
    }

    /// 根据函数ID获取节点索引
    pub fn get_node_index(&self, function_id: &Uuid) -> Option<NodeIndex> {
        self.function_to_node.get(function_id).copied()
    }

    /// 根据节点索引获取函数信息
    pub fn get_function(&self, node_index: NodeIndex) -> Option<&FunctionInfo> {
        self.graph.node_weight(node_index)
    }

    /// 根据函数ID获取函数信息
    pub fn get_function_by_id(&self, function_id: &Uuid) -> Option<&FunctionInfo> {
        self.function_to_node.get(function_id)
            .and_then(|&node_index| self.graph.node_weight(node_index))
    }

    /// 获取函数的调用者
    pub fn get_callers(&self, function_id: &Uuid) -> Vec<(&FunctionInfo, &CallRelation)> {
        let mut callers = Vec::new();
        if let Some(&node_index) = self.function_to_node.get(function_id) {
            for edge in self.graph.edges_directed(node_index, Direction::Incoming) {
                let caller_node = edge.source();
                let caller_function = self.graph.node_weight(caller_node).unwrap();
                let relation = edge.weight();
                callers.push((caller_function, relation));
            }
        }
        callers
    }

    /// 获取函数调用的函数
    pub fn get_callees(&self, function_id: &Uuid) -> Vec<(&FunctionInfo, &CallRelation)> {
        let mut callees = Vec::new();
        if let Some(&node_index) = self.function_to_node.get(function_id) {
            for edge in self.graph.edges_directed(node_index, Direction::Outgoing) {
                let callee_node = edge.target();
                let callee_function = self.graph.node_weight(callee_node).unwrap();
                let relation = edge.weight();
                callees.push((callee_function, relation));
            }
        }
        callees
    }

    /// 根据函数名查找函数
    pub fn find_functions_by_name(&self, name: &str) -> Vec<&FunctionInfo> {
        self.function_names
            .get(name)
            .map(|ids| ids.iter().filter_map(|id| self.get_function_by_id(id)).collect())
            .unwrap_or_default()
    }

    /// 根据文件路径查找函数
    pub fn find_functions_by_file(&self, file_path: &PathBuf) -> Vec<&FunctionInfo> {
        self.file_functions
            .get(file_path)
            .map(|ids| ids.iter().filter_map(|id| self.get_function_by_id(id)).collect())
            .unwrap_or_default()
    }

    /// 获取调用链（递归）
    pub fn get_call_chain(&self, function_id: &Uuid, max_depth: usize) -> Vec<Vec<Uuid>> {
        let mut chains = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self._get_call_chain_recursive(function_id, &mut chains, &mut visited, 0, max_depth);
        chains
    }

    fn _get_call_chain_recursive(
        &self,
        function_id: &Uuid,
        chains: &mut Vec<Vec<Uuid>>,
        visited: &mut std::collections::HashSet<Uuid>,
        depth: usize,
        max_depth: usize,
    ) {
        if depth >= max_depth || visited.contains(function_id) {
            return;
        }

        visited.insert(*function_id);
        let callees = self.get_callees(function_id);
        
        if callees.is_empty() {
            chains.push(vec![*function_id]);
        } else {
            for (callee_function, _) in callees {
                let mut sub_chains = Vec::new();
                self._get_call_chain_recursive(&callee_function.id, &mut sub_chains, visited, depth + 1, max_depth);
                
                for mut chain in sub_chains {
                    chain.insert(0, *function_id);
                    chains.push(chain);
                }
            }
        }
    }

    /// 导出为Mermaid格式
    pub fn to_mermaid(&self) -> String {
        let mut mermaid = String::from("graph TD\n");
        
        // 添加节点
        for node_index in self.graph.node_indices() {
            if let Some(function) = self.graph.node_weight(node_index) {
                let node_id = function.id.to_string().replace("-", "_");
                let label = format!("{}\\n{}", function.name, function.file_path.display());
                mermaid.push_str(&format!("    {}[\"{}\"]\n", node_id, label));
            }
        }
        
        // 添加边
        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                if let (Some(caller), Some(callee)) = (self.graph.node_weight(source), self.graph.node_weight(target)) {
                    let caller_id = caller.id.to_string().replace("-", "_");
                    let callee_id = callee.id.to_string().replace("-", "_");
                    if let Some(relation) = self.graph.edge_weight(edge) {
                        let style = if relation.is_resolved { "" } else { ":::unresolved" };
                        mermaid.push_str(&format!("    {} --> {}{}\n", caller_id, callee_id, style));
                    }
                }
            }
        }
        
        // 添加样式
        mermaid.push_str("\nclassDef unresolved stroke-dasharray: 5 5\n");
        
        mermaid
    }

    /// 导出为DOT格式
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph CodeGraph {\n");
        dot.push_str("    rankdir=TB;\n");
        dot.push_str("    node [shape=box];\n\n");
        
        // 添加节点
        for node_index in self.graph.node_indices() {
            if let Some(function) = self.graph.node_weight(node_index) {
                let node_id = function.id.to_string().replace("-", "_");
                let label = format!("{}\\n{}", function.name, function.file_path.display());
                dot.push_str(&format!("    {} [label=\"{}\"];\n", node_id, label));
            }
        }
        
        // 添加边
        for edge in self.graph.edge_indices() {
            if let Some((source, target)) = self.graph.edge_endpoints(edge) {
                if let (Some(caller), Some(callee)) = (self.graph.node_weight(source), self.graph.node_weight(target)) {
                    let caller_id = caller.id.to_string().replace("-", "_");
                    let callee_id = callee.id.to_string().replace("-", "_");
                    if let Some(relation) = self.graph.edge_weight(edge) {
                        let style = if relation.is_resolved { "" } else { " [style=dashed]" };
                        dot.push_str(&format!("    {} -> {}{};\n", caller_id, callee_id, style));
                    }
                }
            }
        }
        
        dot.push_str("}\n");
        dot
    }

    /// 导出为JSON格式
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// 从JSON格式加载
    pub fn from_json(json_str: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json_str)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &CodeGraphStats {
        &self.stats
    }

    /// 更新统计信息
    pub fn update_stats(&mut self) {
        self.stats.total_files = self.file_functions.len();
        self.stats.total_languages = self.stats.languages.len();
    }

    /// 获取所有函数
    pub fn get_all_functions(&self) -> Vec<&FunctionInfo> {
        self.graph.node_weights().collect()
    }

    /// 获取所有调用关系
    pub fn get_all_call_relations(&self) -> Vec<&CallRelation> {
        self.graph.edge_weights().collect()
    }

    /// 检查是否存在循环依赖
    pub fn has_cycles(&self) -> bool {
        petgraph::algo::is_cyclic_directed(&self.graph)
    }

    /// 获取拓扑排序
    pub fn topological_sort(&self) -> Result<Vec<NodeIndex>, petgraph::algo::Cycle<NodeIndex>> {
        petgraph::algo::toposort(&self.graph, None)
    }

    /// 获取强连通分量
    pub fn strongly_connected_components(&self) -> Vec<Vec<NodeIndex>> {
        petgraph::algo::kosaraju_scc(&self.graph)
    }
}

impl Default for PetCodeGraph {
    fn default() -> Self {
        Self::new()
    }
}