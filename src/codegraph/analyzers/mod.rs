// 语言分析器模块
pub mod java_analyzer;
pub mod python_analyzer;
pub mod typescript_analyzer;
pub mod javascript_analyzer;
pub mod cpp_analyzer;
pub mod rust_analyzer;

// 语言特定解析器
pub mod java_parser;
pub mod typescript_parser;
pub mod javascript_parser;
pub mod rust_parser;
pub mod utils;

// 语言分析器适配器
pub mod language_adapters;

// 重新导出主要的分析器
pub use java_analyzer::JavaAnalyzer;
pub use python_analyzer::PythonAnalyzer;
pub use typescript_analyzer::TypeScriptAnalyzer;
pub use javascript_analyzer::JavaScriptAnalyzer;
pub use cpp_analyzer::CppAnalyzer;
pub use rust_analyzer::RustAnalyzer;

// 重新导出语言适配器
pub use language_adapters::{
    RustLanguageAnalyzer, JavaLanguageAnalyzer, PythonLanguageAnalyzer,
    CppLanguageAnalyzer, TypeScriptLanguageAnalyzer, JavaScriptLanguageAnalyzer,
};

// 重新导出解析器
pub use java_parser::JavaParser;
pub use typescript_parser::TypeScriptParser;
pub use javascript_parser::JavaScriptParser;
pub use rust_parser::RustParser;

// 导出通用接口和错误类型
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use crate::codegraph::treesitter::ast_instance_structs::AstSymbolInstanceArc;
use crate::codegraph::treesitter::language_id::LanguageId;
use crate::codegraph::types::{FunctionInfo, ClassInfo, CallRelation};

// 新增：代码片段结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Snippet {
    pub id: Uuid,
    pub file_path: PathBuf,
    pub language: String,
    pub range: (usize, usize, usize, usize), // (start_line, start_col, end_line, end_col)
    pub function_id: Option<Uuid>,
    pub preview: Option<String>,
}

// 新增：解析单元结构
#[derive(Debug, Clone)]
pub struct ParsedUnit {
    pub file_path: PathBuf,
    pub language: LanguageId,
    pub content: String,
    pub ast_nodes: Vec<AstSymbolInstanceArc>,
}

// 新增：分析选项
#[derive(Debug, Clone)]
pub struct AnalyzeOptions {
    pub languages: Option<Vec<LanguageId>>,
    pub max_workers: usize,
    pub include_tests: bool,
    pub follow_symlinks: bool,
    pub output_dir: PathBuf,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            languages: None,
            max_workers: num_cpus::get(),
            include_tests: false,
            follow_symlinks: true,
            output_dir: PathBuf::from("target/codegraph"),
        }
    }
}

// 新增：分析结果
#[derive(Debug, Clone)]
pub struct AnalyzeResult {
    pub graph: crate::codegraph::graph::CodeGraph,
    pub snippets: Vec<Snippet>,
    pub errors: Vec<String>,
}

// 新增：统一语言分析器接口
pub trait LanguageAnalyzer: Send + Sync {
    fn language(&self) -> LanguageId;
    fn parse_file(&self, path: &Path) -> anyhow::Result<ParsedUnit>;
    fn extract_functions(&self, unit: &ParsedUnit) -> Vec<FunctionInfo>;
    fn extract_calls(&self, unit: &ParsedUnit) -> Vec<CallRelation>;
    fn extract_snippets(&self, unit: &ParsedUnit) -> Vec<Snippet>;
}

// 新增：分析器注册表
#[derive(Clone)]
pub struct AnalyzerRegistry {
    analyzers: HashMap<LanguageId, Arc<dyn LanguageAnalyzer>>,
}

impl AnalyzerRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            analyzers: HashMap::new(),
        };
        
        // 注册所有支持的语言分析器
        registry.register(LanguageId::Rust, Arc::new(RustLanguageAnalyzer::new().unwrap()));
        registry.register(LanguageId::Java, Arc::new(JavaLanguageAnalyzer::new().unwrap()));
        registry.register(LanguageId::Python, Arc::new(PythonLanguageAnalyzer::new().unwrap()));
        registry.register(LanguageId::Cpp, Arc::new(CppLanguageAnalyzer::new().unwrap()));
        registry.register(LanguageId::TypeScript, Arc::new(TypeScriptLanguageAnalyzer::new().unwrap()));
        registry.register(LanguageId::TypeScriptReact, Arc::new(TypeScriptLanguageAnalyzer::new().unwrap()));
        registry.register(LanguageId::JavaScript, Arc::new(JavaScriptLanguageAnalyzer::new().unwrap()));
        
        registry
    }
    
    pub fn register(&mut self, language: LanguageId, analyzer: Arc<dyn LanguageAnalyzer>) {
        self.analyzers.insert(language, analyzer);
    }
    
    pub fn by_language(&self, lang: LanguageId) -> Option<&dyn LanguageAnalyzer> {
        self.analyzers.get(&lang).map(|a| a.as_ref())
    }
    
    pub fn supported_languages(&self) -> Vec<LanguageId> {
        self.analyzers.keys().cloned().collect()
    }
}

// 新增：编排器
pub struct AnalyzerOrchestrator {
    registry: AnalyzerRegistry,
}

impl AnalyzerOrchestrator {
    pub fn new() -> Self {
        Self {
            registry: AnalyzerRegistry::new(),
        }
    }
    
    pub fn run(root: &Path, options: AnalyzeOptions) -> anyhow::Result<AnalyzeResult> {
        let orchestrator = Self::new();
        orchestrator.run_internal(root, options)
    }
    
    fn run_internal(&self, root: &Path, options: AnalyzeOptions) -> anyhow::Result<AnalyzeResult> {
        // 1. 发现文件
        let files = self.discover_files(root, &options)?;
        
        // 2. 并发解析
        let results = self.parse_files_concurrent(&files, &options)?;
        
        // 3. 聚合结果
        let mut graph = crate::codegraph::graph::CodeGraph::new();
        let mut snippets = Vec::new();
        let mut errors = Vec::new();
        
        for result in results {
            match result {
                Ok((functions, calls, file_snippets)) => {
                    // 添加函数到图
                    for function in functions {
                        graph.add_function(function);
                    }
                    
                    // 添加调用关系到图
                    for call in calls {
                        graph.add_call_relation(call);
                    }
                    
                    // 收集代码片段
                    snippets.extend(file_snippets);
                }
                Err(error) => {
                    errors.push(error.to_string());
                }
            }
        }
        
        // 4. 导出结果
        self.export_results(&graph, &snippets, &options)?;
        
        Ok(AnalyzeResult {
            graph,
            snippets,
            errors,
        })
    }
    
    fn discover_files(&self, root: &Path, options: &AnalyzeOptions) -> anyhow::Result<Vec<(PathBuf, LanguageId)>> {
        let mut files = Vec::new();
        let supported_languages = options.languages.as_ref()
            .map(|langs| langs.clone())
            .unwrap_or_else(|| self.registry.supported_languages());
        
        for entry in walkdir::WalkDir::new(root)
            .follow_links(options.follow_symlinks)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // 跳过目录
            if !path.is_file() {
                continue;
            }
            
            // 跳过测试文件（如果配置了）
            if !options.include_tests {
                let path_str = path.to_string_lossy();
                if path_str.contains("test") || path_str.contains("spec") || path_str.contains("__tests__") {
                    continue;
                }
            }
            
            // 识别语言
            if let Some(language) = self.identify_language(path) {
                if supported_languages.contains(&language) {
                    files.push((path.to_path_buf(), language));
                }
            }
        }
        
        Ok(files)
    }
    
    fn identify_language(&self, path: &Path) -> Option<LanguageId> {
        // 使用现有的语言识别逻辑
        crate::codegraph::analyzers::get_language_id_by_filename(&path.to_path_buf())
    }
    
    fn parse_files_concurrent(
        &self,
        files: &[(PathBuf, LanguageId)],
        options: &AnalyzeOptions,
    ) -> anyhow::Result<Vec<anyhow::Result<(Vec<FunctionInfo>, Vec<CallRelation>, Vec<Snippet>)>>> {
        use std::sync::mpsc;
        use std::thread;
        
        let (tx, rx) = mpsc::channel();
        let mut handles = Vec::new();
        
        // 创建工作线程
        for chunk in files.chunks((files.len() + options.max_workers - 1) / options.max_workers) {
            let chunk = chunk.to_vec();
            let tx = tx.clone();
            let registry = self.registry.clone();
            
            let handle = thread::spawn(move || {
                for (file_path, language) in chunk {
                    let result = parse_single_file(&registry, &file_path, language);
                    tx.send(result).unwrap();
                }
            });
            
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        // 收集结果
        let mut results = Vec::new();
        while let Ok(result) = rx.try_recv() {
            results.push(result);
        }
        
        Ok(results)
    }
    
    fn export_results(
        &self,
        graph: &crate::codegraph::graph::CodeGraph,
        snippets: &[Snippet],
        options: &AnalyzeOptions,
    ) -> anyhow::Result<()> {
        // 创建输出目录
        std::fs::create_dir_all(&options.output_dir)?;
        
        // 导出图
        let graph_json = graph.to_json()?;
        std::fs::write(options.output_dir.join("graph.json"), graph_json)?;
        
        let graph_mermaid = graph.to_mermaid();
        std::fs::write(options.output_dir.join("graph.mmd"), graph_mermaid)?;
        
        let graph_dot = graph.to_dot();
        std::fs::write(options.output_dir.join("graph.dot"), graph_dot)?;
        
        // 导出代码片段
        let snippets_json = serde_json::to_string_pretty(snippets)?;
        std::fs::write(options.output_dir.join("snippets.json"), snippets_json)?;
        
        Ok(())
    }
}

fn parse_single_file(
    registry: &AnalyzerRegistry,
    file_path: &Path,
    language: LanguageId,
) -> anyhow::Result<(Vec<FunctionInfo>, Vec<CallRelation>, Vec<Snippet>)> {
    let analyzer = registry.by_language(language)
        .ok_or_else(|| anyhow::anyhow!("No analyzer found for language: {:?}", language))?;
    
    // 解析文件
    let unit = analyzer.parse_file(file_path)?;
    
    // 提取信息
    let functions = analyzer.extract_functions(&unit);
    let calls = analyzer.extract_calls(&unit);
    let snippets = analyzer.extract_snippets(&unit);
    
    Ok((functions, calls, snippets))
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParserError {
    pub message: String,
}

impl From<Box<dyn std::error::Error>> for ParserError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        ParserError {
            message: err.to_string(),
        }
    }
}

pub trait AstLanguageParser: Send {
    fn parse(&mut self, code: &str, path: &PathBuf) -> Vec<AstSymbolInstanceArc>;
}

/// 分析结果结构
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub call_relations: Vec<CallRelation>,
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            functions: Vec::new(),
            classes: Vec::new(),
            call_relations: Vec::new(),
        }
    }
}

// 新的统一接口，使用Analyzer系统
pub trait CodeAnalyzer: Send {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String>;
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String>;
    
    /// 获取分析结果
    fn get_analysis_result(&self, path: &PathBuf) -> Result<AnalysisResult, String> {
        let functions = self.extract_functions(path)?;
        let classes = self.extract_classes(path)?;
        let call_relations = self.extract_call_relations(path)?;
        
        Ok(AnalysisResult {
            functions,
            classes,
            call_relations,
        })
    }
    
    /// 提取函数信息
    fn extract_functions(&self, _path: &PathBuf) -> Result<Vec<FunctionInfo>, String> {
        // Default implementation returns empty vector
        Ok(Vec::new())
    }
    
    /// 提取类信息
    fn extract_classes(&self, _path: &PathBuf) -> Result<Vec<ClassInfo>, String> {
        // Default implementation returns empty vector
        Ok(Vec::new())
    }
    
    /// 提取调用关系
    fn extract_call_relations(&self, _path: &PathBuf) -> Result<Vec<CallRelation>, String> {
        // Default implementation returns empty vector
        Ok(Vec::new())
    }
}

// 为现有的Analyzer实现CodeAnalyzer trait
impl CodeAnalyzer for JavaAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        JavaAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        JavaAnalyzer::analyze_directory(self, dir.as_path())
    }
}

impl CodeAnalyzer for PythonAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        PythonAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        PythonAnalyzer::analyze_directory(self, dir.as_path())
    }
}

impl CodeAnalyzer for CppAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        CppAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        CppAnalyzer::analyze_directory(self, dir.as_path())
    }
}

impl CodeAnalyzer for TypeScriptAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        TypeScriptAnalyzer::analyze_file(self, path.as_path())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        TypeScriptAnalyzer::analyze_directory(self, dir.as_path())
    }
}

impl CodeAnalyzer for JavaScriptAnalyzer {
    fn analyze_file(&mut self, path: &PathBuf) -> Result<(), String> {
        JavaScriptAnalyzer::analyze_file(self, path.as_path())
            .map_err(|e| e.to_string())
    }
    
    fn analyze_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        JavaScriptAnalyzer::analyze_directory(self, dir.as_path())
            .map_err(|e| e.to_string())
    }
}

fn internal_error<E: Display>(err: E) -> ParserError {
    let err_msg = err.to_string();
    tracing::error!(err_msg);
    ParserError {
        message: err_msg.into(),
    }
}

// 重构后的get_ast_parser函数，现在返回CodeAnalyzer
pub(crate) fn get_code_analyzer(language_id: crate::codegraph::treesitter::language_id::LanguageId) -> Result<Box<dyn CodeAnalyzer + 'static>, ParserError> {
    match language_id {
        crate::codegraph::treesitter::language_id::LanguageId::Rust => {
            let analyzer = RustAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::Python => {
            let analyzer = PythonAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::Java => {
            let analyzer = JavaAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::Cpp => {
            let analyzer = CppAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::TypeScript => {
            let analyzer = TypeScriptAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::JavaScript => {
            let analyzer = JavaScriptAnalyzer::new()
                .map_err(|e| ParserError { message: e.to_string() })?;
            Ok(Box::new(analyzer))
        }
        crate::codegraph::treesitter::language_id::LanguageId::TypeScriptReact => {
            let analyzer = TypeScriptAnalyzer::new()
                .map_err(|e| ParserError { message: e })?;
            Ok(Box::new(analyzer))
        }
        other => Err(ParserError {
            message: "Unsupported language id: ".to_string() + &other.to_string()
        }),
    }
}

// 保持向后兼容的get_ast_parser函数，但现在返回CodeAnalyzer
pub(crate) fn get_ast_parser(language_id: crate::codegraph::treesitter::language_id::LanguageId) -> Result<Box<dyn CodeAnalyzer + 'static>, ParserError> {
    get_code_analyzer(language_id)
}

pub fn get_ast_parser_by_filename(filename: &PathBuf) -> Result<(Box<dyn CodeAnalyzer + 'static>, crate::codegraph::treesitter::language_id::LanguageId), ParserError> {
    let suffix = filename.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let maybe_language_id = get_language_id_by_filename(filename);
    match maybe_language_id {
        Some(language_id) => {
            let analyzer = get_code_analyzer(language_id)?;
            Ok((analyzer, language_id))
        }
        None => Err(ParserError { message: format!("not supported {}", suffix) }),
    }
}

pub fn get_language_id_by_filename(filename: &PathBuf) -> Option<crate::codegraph::treesitter::language_id::LanguageId> {
    let suffix = filename.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match suffix.as_str() {
        "cpp" | "cc" | "cxx" | "c++" | "c" | "h" | "hpp" | "hxx" | "hh" => Some(crate::codegraph::treesitter::language_id::LanguageId::Cpp),
        "inl" | "inc" | "tpp" | "tpl" => Some(crate::codegraph::treesitter::language_id::LanguageId::Cpp),
        "py" | "py3" | "pyx" => Some(crate::codegraph::treesitter::language_id::LanguageId::Python),
        "java" => Some(crate::codegraph::treesitter::language_id::LanguageId::Java),
        "js" | "jsx" => Some(crate::codegraph::treesitter::language_id::LanguageId::JavaScript),
        "rs" => Some(crate::codegraph::treesitter::language_id::LanguageId::Rust),
        "ts" | "tsx" => Some(crate::codegraph::treesitter::language_id::LanguageId::TypeScript),
        _ => None
    }
} 