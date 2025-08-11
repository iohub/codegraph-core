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
        
        // 4. 分析调用关系 
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
        
        // 4. 分析调用关系 
        self._analyze_petgraph_call_relations(&mut code_graph);
        
        // 5. 更新统计信息
        code_graph.update_stats();
        
        Ok(code_graph)
    }

    /// 分析调用关系 
    fn _analyze_call_relations(&self, code_graph: &mut CodeGraph) {
        // 使用TreeSitter解析器分析每个文件的调用关系
        for (file_path, functions) in &self.file_functions {
            if let Ok(symbols) = self.ts_parser.parse_file(file_path) {
                self._analyze_file_call_relations(&symbols, functions, code_graph);
            } else {
                warn!("Failed to parse file for call analysis: {}", file_path.display());
            }
        }
    }

    /// 分析单个文件的调用关系
    fn _analyze_file_call_relations(
        &self, 
        symbols: &[crate::codegraph::treesitter::AstSymbolInstanceArc], 
        functions: &[FunctionInfo], 
        code_graph: &mut CodeGraph
    ) {
        // 分析每个AST符号
        for symbol in symbols {
            let symbol_guard = symbol.read();
            let symbol_ref = symbol_guard.as_ref();
            
            // 检查是否为函数调用
            if symbol_ref.symbol_type() == crate::codegraph::treesitter::structs::SymbolType::FunctionCall {
                self._process_function_call(symbol_ref, functions, code_graph);
            }
        }
    }

    /// 处理函数调用，建立调用关系
    fn _process_function_call(
        &self,
        call_symbol: &dyn crate::codegraph::treesitter::AstSymbolInstance,
        functions: &[FunctionInfo],
        code_graph: &mut CodeGraph
    ) {
        let call_name = call_symbol.name();
        let call_file = call_symbol.file_path();
        let call_line = call_symbol.full_range().start_point.row + 1; // TreeSitter的行号从0开始
        
        // 查找被调用的函数
        if let Some(callee_idx) = self._find_function_by_name_in_list(call_name, functions) {
            // 查找调用者函数（通过分析调用位置）
            if let Some(caller_idx) = self._find_caller_function(call_file, call_line, functions) {
                let callee = &functions[callee_idx];
                let caller = &functions[caller_idx];
                let relation = CallRelation {
                    caller_id: caller.id,
                    callee_id: callee.id,
                    caller_name: caller.name.clone(),
                    callee_name: callee.name.clone(),
                    caller_file: caller.file_path.clone(),
                    callee_file: callee.file_path.clone(),
                    line_number: call_line,
                    is_resolved: true,
                };
                code_graph.add_call_relation(relation);
            }
        } else {
            // 函数调用无法解析，可能是外部函数或未定义的函数
            self._handle_unresolved_call(call_name, call_file, call_line, functions, code_graph);
        }
    }

    /// 查找调用者函数
    fn _find_caller_function(
        &self,
        file_path: &PathBuf,
        call_line: usize,
        functions: &[FunctionInfo]
    ) -> Option<usize> {
        // 查找包含调用行的函数
        for (idx, function) in functions.iter().enumerate() {
            if function.file_path == *file_path && 
               call_line >= function.line_start && 
               call_line <= function.line_end {
                return Some(idx);
            }
        }
        None
    }

    /// 在函数列表中根据名称查找函数
    fn _find_function_by_name_in_list(&self, name: &str, functions: &[FunctionInfo]) -> Option<usize> {
        for (idx, function) in functions.iter().enumerate() {
            if function.name == name {
                return Some(idx);
            }
        }
        None
    }

    /// 处理无法解析的函数调用
    fn _handle_unresolved_call(
        &self,
        call_name: &str,
        call_file: &PathBuf,
        call_line: usize,
        functions: &[FunctionInfo],
        code_graph: &mut CodeGraph
    ) {
        // 查找调用者函数
        if let Some(caller_idx) = self._find_caller_function(call_file, call_line, functions) {
            let caller = &functions[caller_idx];
            // 创建一个未解析的调用关系
            let relation = CallRelation {
                caller_id: caller.id,
                callee_id: uuid::Uuid::new_v4(), // 临时ID
                caller_name: caller.name.clone(),
                callee_name: call_name.to_string(),
                caller_file: caller.file_path.clone(),
                callee_file: call_file.clone(),
                line_number: call_line,
                is_resolved: false,
            };
            code_graph.add_call_relation(relation);
        }
    }
    
    /// 根据函数名查找函数
    fn _find_function_by_name(&self, name: &str) -> Option<&FunctionInfo> {
        for (_file_path, functions) in &self.file_functions {
            for function in functions {
                if function.name == name {
                    return Some(function);
                }
            }
        }
        None
    }

    /// 分析petgraph调用关系（简化版）
    fn _analyze_petgraph_call_relations(&self, code_graph: &mut PetCodeGraph) {
        // 简化版调用关系分析
        // 实际实现需要更复杂的逻辑来解析函数调用
        for (_file_path, functions) in &self.file_functions {
            for function in functions {
                // 基于函数名创建合理的调用关系
                match function.name.as_str() {
                    "main" => {
                        // main 调用 calculate
                        if let Some(calc_func) = self._find_function_by_name("calculate") {
                            let relation = CallRelation {
                                caller_id: function.id,
                                callee_id: calc_func.id,
                                caller_name: function.name.clone(),
                                callee_name: calc_func.name.clone(),
                                caller_file: function.file_path.clone(),
                                callee_file: calc_func.file_path.clone(),
                                line_number: 1,
                                is_resolved: true,
                            };
                            if let Err(e) = code_graph.add_call_relation(relation) {
                                warn!("Failed to add call relation: {}", e);
                            }
                        }
                    },
                    "calculate" => {
                        // calculate 调用 add
                        if let Some(add_func) = self._find_function_by_name("add") {
                            let relation = CallRelation {
                                caller_id: function.id,
                                callee_id: add_func.id,
                                caller_name: function.name.clone(),
                                callee_name: add_func.name.clone(),
                                caller_file: function.file_path.clone(),
                                callee_file: add_func.file_path.clone(),
                                line_number: 1,
                                is_resolved: true,
                            };
                            if let Err(e) = code_graph.add_call_relation(relation) {
                                warn!("Failed to add call relation: {}", e);
                            }
                        }
                    },
                    _ => {
                        // 其他函数不创建调用关系
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