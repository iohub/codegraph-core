use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use tracing::{info, warn};

use crate::codegraph::types::{FunctionInfo, CallRelation, PetCodeGraph};
use crate::codegraph::graph::CodeGraph;
use crate::codegraph::treesitter::TreeSitterParser;

/// 代码解析器，负责解析源代码文件并提取函数调用关系
pub struct CodeParser {
    /// 文件路径 -> 函数列表映射
    file_functions: HashMap<PathBuf, Vec<FunctionInfo>>,
    /// 函数名 -> 函数信息映射（用于解析调用关系）
    function_registry: HashMap<String, FunctionInfo>,
    /// Tree-sitter解析器
    ts_parser: TreeSitterParser,
}

impl CodeParser {
    pub fn new() -> Self {
        Self {
            file_functions: HashMap::new(),
            function_registry: HashMap::new(),
            ts_parser: TreeSitterParser::new(),
        }
    }

    /// 扫描目录下的所有支持的文件
    pub fn scan_directory(&mut self, dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self._scan_directory_recursive(dir, &mut files);
        files
    }

    fn _scan_directory_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 跳过常见的忽略目录
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "__pycache__" {
                            continue;
                        }
                    }
                    self._scan_directory_recursive(&path, files);
                } else if self.is_supported_file(&path) {
                    files.push(path);
                }
            }
        }
    }

    /// 判断文件是否为支持的源代码文件
    fn is_supported_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(),
                "cpp" | "cc" | "cxx" | "c++" | "c" | "h" | "hpp" | "hxx" | "hh" |
                "inl" | "inc" | "tpp" | "tpl" |
                "py" | "py3" | "pyx" |
                "java" |
                "js" | "jsx" |
                "rs" |
                "ts" |
                "tsx"
            )
        } else {
            false
        }
    }

    /// 解析单个文件（简化版本，仅用于演示）
    pub fn parse_file(&mut self, file_path: &PathBuf) -> Result<(), String> {
        info!("Parsing file: {}", file_path.display());
        
        // 读取文件内容
        let code = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path.display(), e))?;

        // 简化的函数提取逻辑（实际实现需要使用tree-sitter等解析器）
        let functions = self._extract_functions_simple(&code, file_path);
        
        // 注册函数
        for function in &functions {
            self.function_registry.insert(function.name.clone(), function.clone());
        }
        
        // 保存文件函数映射
        self.file_functions.insert(file_path.clone(), functions);

        Ok(())
    }

    /// 简化的函数提取（实际实现需要使用tree-sitter等解析器）
    fn _extract_functions_simple(&self, code: &str, file_path: &PathBuf) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        
        // 获取文件扩展名以确定语言
        let language = if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "rs" => "rust".to_string(),
                "py" | "py3" | "pyx" => "python".to_string(),
                "js" | "jsx" => "javascript".to_string(),
                "ts" | "tsx" => "typescript".to_string(),
                "java" => "java".to_string(),
                "cpp" | "cc" | "cxx" | "c++" | "c" | "h" | "hpp" | "hxx" | "hh" => "cpp".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        };
        
        // 简单的函数名提取（实际实现需要更复杂的解析逻辑）
        // 这里仅作为示例，实际项目中需要使用tree-sitter等解析器
        let lines: Vec<&str> = code.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            // 简单的函数检测逻辑（仅用于演示）
            if line.contains("fn ") || line.contains("def ") || line.contains("function ") {
                let function_name = self._extract_function_name(line);
                if !function_name.is_empty() {
                    let function_info = FunctionInfo {
                        id: Uuid::new_v4(),
                        name: function_name,
                        file_path: file_path.clone(),
                        line_start: i + 1,
                        line_end: i + 1,
                        namespace: "global".to_string(),
                        language: language.clone(),
                        signature: Some(line.trim().to_string()),
                        return_type: None,
                        parameters: vec![],
                    };
                    functions.push(function_info);
                }
            }
        }
        
        functions
    }

    /// 简化的函数名提取
    fn _extract_function_name(&self, line: &str) -> String {
        let line = line.trim();
        
        // Rust函数: fn name(...)
        if let Some(start) = line.find("fn ") {
            if let Some(end) = line[start+3..].find('(') {
                return line[start+3..start+3+end].trim().to_string();
            }
        }
        
        // Python函数: def name(...)
        if let Some(start) = line.find("def ") {
            if let Some(end) = line[start+4..].find('(') {
                return line[start+4..start+4+end].trim().to_string();
            }
        }
        
        // JavaScript/TypeScript函数: function name(...)
        if let Some(start) = line.find("function ") {
            if let Some(end) = line[start+9..].find('(') {
                return line[start+9..start+9+end].trim().to_string();
            }
        }
        
        String::new()
    }

    /// 解析目录下的所有文件
    pub fn parse_directory(&mut self, dir: &Path) -> Result<(), String> {
        let files = self.scan_directory(dir);
        info!("Found {} files to parse", files.len());

        for file in files {
            if let Err(e) = self.parse_file(&file) {
                warn!("Failed to parse {}: {}", file.display(), e);
            }
        }

        Ok(())
    }

    /// 构建完整的代码图
    pub fn build_code_graph(&mut self, dir: &Path) -> Result<CodeGraph, String> {
        // 1. 解析所有文件
        self.parse_directory(dir)?;
        
        // 2. 构建代码图
        let mut code_graph = CodeGraph::new();
        
        // 3. 提取函数信息并直接添加到代码图
        for (_file_path, functions) in &self.file_functions {
            for function in functions {
                code_graph.add_function(function.clone());
            }
        }
        
        // 4. 分析调用关系（简化版）
        self._analyze_call_relations(&mut code_graph);
        
        // 5. 更新统计信息
        code_graph.update_stats();
        
        Ok(code_graph)
    }

    /// 构建基于petgraph的代码图
    pub fn build_petgraph_code_graph(&mut self, dir: &Path) -> Result<PetCodeGraph, String> {
        // 1. 解析所有文件
        self.parse_directory(dir)?;
        
        // 2. 构建petgraph代码图
        let mut code_graph = PetCodeGraph::new();
        
        // 3. 提取函数信息并直接添加到代码图
        for (_file_path, functions) in &self.file_functions {
            for function in functions {
                code_graph.add_function(function.clone());
            }
        }
        
        // 4. 分析调用关系（简化版）
        self._analyze_petgraph_call_relations(&mut code_graph);
        
        // 5. 更新统计信息
        code_graph.update_stats();
        
        Ok(code_graph)
    }

    /// 分析调用关系（简化版）
    fn _analyze_call_relations(&self, code_graph: &mut CodeGraph) {
        // 简化版调用关系分析
        // 实际实现需要更复杂的逻辑来解析函数调用
        for (_file_path, functions) in &self.file_functions {
            for function in functions {
                // 模拟一些调用关系
                for (other_file_path, other_functions) in &self.file_functions {
                    if other_file_path != _file_path {
                        for other_function in other_functions {
                            if function.name != other_function.name {
                                let relation = CallRelation {
                                    caller_id: function.id,
                                    callee_id: other_function.id,
                                    caller_name: function.name.clone(),
                                    callee_name: other_function.name.clone(),
                                    caller_file: function.file_path.clone(),
                                    callee_file: other_function.file_path.clone(),
                                    line_number: 1,
                                    is_resolved: true,
                                };
                                code_graph.add_call_relation(relation);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    /// 分析petgraph调用关系（简化版）
    fn _analyze_petgraph_call_relations(&self, code_graph: &mut PetCodeGraph) {
        // 简化版调用关系分析
        // 实际实现需要更复杂的逻辑来解析函数调用
        for (_file_path, functions) in &self.file_functions {
            for function in functions {
                // 模拟一些调用关系
                for (other_file_path, other_functions) in &self.file_functions {
                    if other_file_path != _file_path {
                        for other_function in other_functions {
                            if function.name != other_function.name {
                                let relation = CallRelation {
                                    caller_id: function.id,
                                    callee_id: other_function.id,
                                    caller_name: function.name.clone(),
                                    callee_name: other_function.name.clone(),
                                    caller_file: function.file_path.clone(),
                                    callee_file: other_function.file_path.clone(),
                                    line_number: 1,
                                    is_resolved: true,
                                };
                                if let Err(e) = code_graph.add_call_relation(relation) {
                                    warn!("Failed to add call relation: {}", e);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for CodeParser {
    fn default() -> Self {
        Self::new()
    }
}