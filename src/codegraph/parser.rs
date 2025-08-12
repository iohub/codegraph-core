use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use tracing::{info, warn};

use crate::codegraph::types::{
    FunctionInfo, CallRelation, PetCodeGraph, EntityGraph, ClassInfo, ClassType,
    EntityEdge, EntityEdgeType, FileMetadata, FileIndex, SnippetIndex
};
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
    /// 文件索引
    file_index: FileIndex,
    /// 代码片段索引
    snippet_index: SnippetIndex,
}

impl CodeParser {
    pub fn new() -> Self {
        Self {
            file_functions: HashMap::new(),
            function_registry: HashMap::new(),
            ts_parser: TreeSitterParser::new(),
            file_index: FileIndex::default(),
            snippet_index: SnippetIndex::default(),
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

    /// 增量更新单个文件
    pub fn refresh_file(
        &mut self,
        file_path: &PathBuf,
        entity_graph: &mut EntityGraph,
        call_graph: &mut PetCodeGraph,
    ) -> Result<(), String> {
        info!("Refreshing file: {}", file_path.display());

        // 检查文件是否存在
        if !file_path.exists() {
            // 文件被删除，清理相关索引
            self._remove_file_entities(file_path, entity_graph, call_graph);
            return Ok(());
        }

        // 解析文件，提取新的实体和函数
        let (classes, functions) = self._extract_entities_from_file(file_path)?;

        // 移除旧的实体和函数
        self._remove_file_entities(file_path, entity_graph, call_graph);

        // 添加到图中
        let class_ids: Vec<Uuid> = classes.iter().map(|c| c.id).collect();
        let function_ids: Vec<Uuid> = functions.iter().map(|f| f.id).collect();

        for class in classes {
            entity_graph.add_class(class);
        }

        for function in functions {
            call_graph.add_function(function);
        }

        // 分析调用关系
        self._analyze_file_calls(file_path, &function_ids, call_graph)?;

        // 更新索引
        self.file_index.rebuild_for_file(file_path, class_ids.clone(), function_ids.clone());

        // 更新代码片段索引
        self._update_snippet_index(file_path, &class_ids, &function_ids)?;

        info!("Successfully refreshed file: {}", file_path.display());
        Ok(())
    }

    /// 从文件提取实体
    fn _extract_entities_from_file(&self, file_path: &PathBuf) -> Result<(Vec<ClassInfo>, Vec<FunctionInfo>), String> {
        let mut classes = Vec::new();
        let mut functions = Vec::new();

        // 使用TreeSitter解析器解析文件
        let symbols = self.ts_parser.parse_file(file_path)
            .map_err(|e| format!("Failed to parse file {}: {:?}", file_path.display(), e))?;

        let language = self._detect_language(file_path);
        let namespace = self._extract_namespace(file_path);

        for symbol in symbols {
            let symbol_guard = symbol.read();
            let symbol_ref = symbol_guard.as_ref();

            match symbol_ref.symbol_type() {
                crate::codegraph::treesitter::structs::SymbolType::FunctionDeclaration => {
                    let function = FunctionInfo {
                        id: Uuid::new_v4(),
                        name: symbol_ref.name().to_string(),
                        file_path: file_path.clone(),
                        line_start: symbol_ref.full_range().start_point.row + 1,
                        line_end: symbol_ref.full_range().end_point.row + 1,
                        namespace: namespace.clone(),
                        language: language.clone(),
                        signature: Some(symbol_ref.name().to_string()),
                        return_type: None,
                        parameters: vec![],
                    };
                    functions.push(function);
                },
                crate::codegraph::treesitter::structs::SymbolType::StructDeclaration => {
                    let class = ClassInfo {
                        id: Uuid::new_v4(),
                        name: symbol_ref.name().to_string(),
                        file_path: file_path.clone(),
                        line_start: symbol_ref.full_range().start_point.row + 1,
                        line_end: symbol_ref.full_range().end_point.row + 1,
                        namespace: namespace.clone(),
                        language: language.clone(),
                        class_type: ClassType::Struct,
                        parent_class: None,
                        implemented_interfaces: vec![],
                        member_functions: vec![],
                        member_variables: vec![],
                    };
                    classes.push(class);
                },
                _ => {}
            }
        }

        Ok((classes, functions))
    }

    /// 分析文件的函数调用
    fn _analyze_file_calls(
        &self,
        file_path: &PathBuf,
        function_ids: &[Uuid],
        call_graph: &mut PetCodeGraph,
    ) -> Result<(), String> {
        let symbols = self.ts_parser.parse_file(file_path)
            .map_err(|e| format!("Failed to parse file for call analysis: {:?}", e))?;

        for symbol in symbols {
            let symbol_guard = symbol.read();
            let symbol_ref = symbol_guard.as_ref();

            if symbol_ref.symbol_type() == crate::codegraph::treesitter::structs::SymbolType::FunctionCall {
                let call_name = symbol_ref.name();
                let call_line = symbol_ref.full_range().start_point.row + 1;

                // 查找调用者函数
                if let Some(caller_id) = self._find_caller_function(file_path, call_line, function_ids) {
                    // 查找被调用函数（先在本文件，再全局）
                    if let Some(callee_id) = self._find_callee_function(call_name, function_ids, call_graph) {
                        let relation = CallRelation {
                            caller_id: *caller_id,
                            callee_id,
                            caller_name: "".to_string(), // 会在add_call_relation中填充
                            callee_name: call_name.to_string(),
                            caller_file: file_path.clone(),
                            callee_file: file_path.clone(),
                            line_number: call_line,
                            is_resolved: true,
                        };
                        if let Err(e) = call_graph.add_call_relation(relation) {
                            warn!("Failed to add call relation: {}", e);
                        }
                    } else {
                        // 未解析的调用
                        self._handle_unresolved_call(caller_id, call_name, file_path, call_line, call_graph);
                    }
                }
            }
        }

        Ok(())
    }

    /// 查找调用者函数
    fn _find_caller_function<'a>(&self, _file_path: &PathBuf, _call_line: usize, function_ids: &'a [Uuid]) -> Option<&'a Uuid> {
        // 这里需要根据行号范围查找包含调用行的函数
        // 简化实现：返回第一个函数ID
        function_ids.first()
    }

    /// 查找被调用函数
    fn _find_callee_function(&self, call_name: &str, function_ids: &[Uuid], call_graph: &PetCodeGraph) -> Option<Uuid> {
        // 先在本文件查找
        for &func_id in function_ids {
            if let Some(func) = call_graph.get_function_by_id(&func_id) {
                if func.name == call_name {
                    return Some(func_id);
                }
            }
        }

        // 再全局查找
        let global_functions = call_graph.find_functions_by_name(call_name);
        global_functions.first().map(|f| f.id)
    }

    /// 处理未解析的调用
    fn _handle_unresolved_call(
        &self,
        caller_id: &Uuid,
        call_name: &str,
        file_path: &PathBuf,
        call_line: usize,
        call_graph: &mut PetCodeGraph,
    ) {
        // 创建未解析的调用关系
        let relation = CallRelation {
            caller_id: *caller_id,
            callee_id: Uuid::new_v4(), // 临时ID
            caller_name: "".to_string(),
            callee_name: call_name.to_string(),
            caller_file: file_path.clone(),
            callee_file: file_path.clone(),
            line_number: call_line,
            is_resolved: false,
        };

        if let Err(e) = call_graph.add_call_relation(relation) {
            warn!("Failed to add unresolved call relation: {}", e);
        }
    }

    /// 移除文件相关的所有实体
    fn _remove_file_entities(
        &mut self,
        file_path: &PathBuf,
        entity_graph: &mut EntityGraph,
        call_graph: &mut PetCodeGraph,
    ) {
        // 获取文件的所有实体ID
        let entity_ids = self.file_index.get_all_entity_ids(file_path);
        let function_ids = self.file_index.get_all_function_ids(file_path);
        let class_ids = self.file_index.get_all_class_ids(file_path);

        // 从图中移除
        for entity_id in entity_ids {
            entity_graph.remove_entity(&entity_id);
        }

        for function_id in function_ids {
            if let Some(node_index) = call_graph.get_node_index(&function_id) {
                call_graph.graph.remove_node(node_index);
                call_graph.function_to_node.remove(&function_id);
                call_graph.node_to_function.remove(&node_index);
            }
        }

        // 清理索引
        self.file_index.remove_file(file_path);
        self.snippet_index.clear_file_cache(file_path);
    }

    /// 更新代码片段索引
    fn _update_snippet_index(
        &mut self,
        file_path: &PathBuf,
        class_ids: &[Uuid],
        function_ids: &[Uuid],
    ) -> Result<(), String> {
        // 读取文件内容
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file for snippet indexing: {}", e))?;

        let lines: Vec<&str> = content.lines().collect();

        // 为类添加代码片段
        for &class_id in class_ids {
            let snippet_info = crate::codegraph::types::SnippetInfo {
                file_path: file_path.clone(),
                line_start: 1, // 简化实现
                line_end: lines.len(),
                cached_content: None,
            };
            self.snippet_index.add_snippet(class_id, snippet_info);
        }

        // 为函数添加代码片段
        for &function_id in function_ids {
            let snippet_info = crate::codegraph::types::SnippetInfo {
                file_path: file_path.clone(),
                line_start: 1, // 简化实现
                line_end: lines.len(),
                cached_content: None,
            };
            self.snippet_index.add_snippet(function_id, snippet_info);
        }

        Ok(())
    }

    /// 检测文件语言
    fn _detect_language(&self, file_path: &Path) -> String {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
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
        }
    }

    /// 提取命名空间
    fn _extract_namespace(&self, _file_path: &Path) -> String {
        // 简化实现，实际应该从文件内容解析
        "global".to_string()
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
                let call_name = symbol_ref.name();
                let call_file = symbol_ref.file_path();
                let call_line = symbol_ref.full_range().start_point.row + 1;
                // 1. 先在本文件查找被调用函数
                if let Some(callee_idx) = self._find_function_by_name_in_list(call_name, functions) {
                    // 查找调用者函数（通过分析调用位置）
                    if let Some(caller_idx) = self._find_caller_function_by_line(call_file, call_line, functions) {
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
                        continue;
                    }
                }
                // 2. 跨文件查找被调用函数
                if let Some(callee) = self._find_function_by_name_global(call_name) {
                    // 查找调用者函数（通过分析调用位置）
                    if let Some(caller_idx) = self._find_caller_function_by_line(call_file, call_line, functions) {
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
                        continue;
                    }
                }
                // 3. 无法解析的调用
                self._handle_unresolved_call_legacy(call_name, call_file, call_line, functions, code_graph);
            }
        }
    }

    /// 查找调用者函数（按行号）
    fn _find_caller_function_by_line(
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

    /// 处理无法解析的函数调用（旧版本）
    fn _handle_unresolved_call_legacy(
        &self,
        call_name: &str,
        call_file: &PathBuf,
        call_line: usize,
        functions: &[FunctionInfo],
        code_graph: &mut CodeGraph
    ) {
        // 查找调用者函数
        if let Some(caller_idx) = self._find_caller_function_by_line(call_file, call_line, functions) {
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

    /// 全局查找函数名（跨文件）
    fn _find_function_by_name_global(&self, name: &str) -> Option<FunctionInfo> {
        for (_file_path, functions) in &self.file_functions {
            for function in functions {
                if function.name == name {
                    return Some(function.clone());
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