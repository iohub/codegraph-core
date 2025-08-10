#[cfg(test)]
mod tests {
    use std::fs;
    use uuid::Uuid;
    use tempfile::TempDir;

    use crate::codegraph::{
        analyzer::CodeGraphAnalyzer, CodeGraph, FunctionInfo, CallRelation,
        types::{CodeGraphStats, ParameterInfo}
    };

    /// 创建测试用的临时目录和文件
    fn create_test_files() -> TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        
        // 创建Python测试文件
        let python_code = r#"
def main():
    result = calculate(10, 20)
    print(result)

def calculate(a, b):
    return add(a, b)

def add(x, y):
    return x + y

def unused_function():
    pass
"#;
        fs::write(temp_dir.path().join("test.py"), python_code).unwrap();

        // 创建Rust测试文件
        let rust_code = r#"
pub fn main() {
    let result = calculate(10, 20);
    println!("{}", result);
}

fn calculate(a: i32, b: i32) -> i32 {
    add(a, b)
}

fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn unused_function() {
    // This function is not called
}
"#;
        fs::write(temp_dir.path().join("test.rs"), rust_code).unwrap();

        // 创建JavaScript测试文件
        let js_code = r#"
function main() {
    const result = calculate(10, 20);
    console.log(result);
}

function calculate(a, b) {
    return add(a, b);
}

function add(x, y) {
    return x + y;
}

function unusedFunction() {
    // This function is not called
}
"#;
        fs::write(temp_dir.path().join("test.js"), js_code).unwrap();

        temp_dir
    }

    #[test]
    fn test_code_graph_creation() {
        let mut code_graph = CodeGraph::new();
        
        // 创建测试函数
        let function1 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "test_function".to_string(),
            file_path: "test.rs".into(),
            line_start: 1,
            line_end: 10,
            namespace: "".to_string(),
            language: "rust".to_string(),
            signature: Some("fn test_function()".to_string()),
            return_type: Some("i32".to_string()),
            parameters: vec![
                ParameterInfo {
                    name: "x".to_string(),
                    type_name: Some("i32".to_string()),
                    default_value: None,
                }
            ],
        };

        let function2 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "another_function".to_string(),
            file_path: "test.rs".into(),
            line_start: 12,
            line_end: 20,
            namespace: "".to_string(),
            language: "rust".to_string(),
            signature: Some("fn another_function()".to_string()),
            return_type: None,
            parameters: vec![],
        };

        // 添加函数到图
        code_graph.add_function(function1.clone());
        code_graph.add_function(function2.clone());

        // 添加调用关系
        let call_relation = CallRelation {
            caller_id: function1.id,
            callee_id: function2.id,
            caller_name: function1.name.clone(),
            callee_name: function2.name.clone(),
            caller_file: function1.file_path.clone(),
            callee_file: function2.file_path.clone(),
            line_number: 5,
            is_resolved: true,
        };

        code_graph.add_call_relation(call_relation);

        // 验证图的状态
        assert_eq!(code_graph.functions.len(), 2);
        assert_eq!(code_graph.call_relations.len(), 1);
        
        let stats = code_graph.get_stats();
        assert_eq!(stats.total_functions, 2);
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.resolved_calls, 1);
    }

    #[test]
    fn test_call_relation_creation() {
        let mut code_graph = CodeGraph::new();
        
        let function1 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "caller".to_string(),
            file_path: "caller.rs".into(),
            line_start: 1,
            line_end: 10,
            namespace: "".to_string(),
            language: "rust".to_string(),
            signature: None,
            return_type: None,
            parameters: vec![],
        };

        let function2 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "callee".to_string(),
            file_path: "callee.rs".into(),
            line_start: 1,
            line_end: 5,
            namespace: "".to_string(),
            language: "rust".to_string(),
            signature: None,
            return_type: None,
            parameters: vec![],
        };

        code_graph.add_function(function1.clone());
        code_graph.add_function(function2.clone());

        let relation = CallRelation {
            caller_id: function1.id,
            callee_id: function2.id,
            caller_name: function1.name.clone(),
            callee_name: function2.name.clone(),
            caller_file: function1.file_path.clone(),
            callee_file: function2.file_path.clone(),
            line_number: 3,
            is_resolved: true,
        };

        code_graph.add_call_relation(relation);

        let callers = code_graph.get_callers(&function2.id);
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].caller_id, function1.id);

        let callees = code_graph.get_callees(&function1.id);
        assert_eq!(callees.len(), 1);
        assert_eq!(callees[0].callee_id, function2.id);
    }

    #[test]
    fn test_function_lookup() {
        let mut code_graph = CodeGraph::new();
        
        let function1 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "main".to_string(),
            file_path: "main.rs".into(),
            line_start: 1,
            line_end: 10,
            namespace: "".to_string(),
            language: "rust".to_string(),
            signature: None,
            return_type: None,
            parameters: vec![],
        };

        let function2 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "main".to_string(),
            file_path: "main.py".into(),
            line_start: 1,
            line_end: 5,
            namespace: "".to_string(),
            language: "python".to_string(),
            signature: None,
            return_type: None,
            parameters: vec![],
        };

        code_graph.add_function(function1.clone());
        code_graph.add_function(function2.clone());

        let functions = code_graph.find_functions_by_name("main");
        assert_eq!(functions.len(), 2);
        
        let rust_functions: Vec<_> = functions.iter()
            .filter(|f| f.language == "rust")
            .collect();
        assert_eq!(rust_functions.len(), 1);
        
        let python_functions: Vec<_> = functions.iter()
            .filter(|f| f.language == "python")
            .collect();
        assert_eq!(python_functions.len(), 1);
    }

    #[test]
    fn test_mermaid_export() {
        let mut code_graph = CodeGraph::new();
        
        let function1 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "start".to_string(),
            file_path: "start.rs".into(),
            line_start: 1,
            line_end: 10,
            namespace: "".to_string(),
            language: "rust".to_string(),
            signature: None,
            return_type: None,
            parameters: vec![],
        };

        let function2 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "end".to_string(),
            file_path: "end.rs".into(),
            line_start: 1,
            line_end: 5,
            namespace: "".to_string(),
            language: "rust".to_string(),
            signature: None,
            return_type: None,
            parameters: vec![],
        };

        code_graph.add_function(function1.clone());
        code_graph.add_function(function2.clone());

        let relation = CallRelation {
            caller_id: function1.id,
            callee_id: function2.id,
            caller_name: function1.name.clone(),
            callee_name: function2.name.clone(),
            caller_file: function1.file_path.clone(),
            callee_file: function2.file_path.clone(),
            line_number: 5,
            is_resolved: true,
        };

        code_graph.add_call_relation(relation);

        let mermaid = code_graph.to_mermaid();
        
        // 验证Mermaid输出包含必要的元素
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("start"));
        assert!(mermaid.contains("end"));
        assert!(mermaid.contains("start --> end"));
    }

    #[test]
    fn test_dot_export() {
        let mut code_graph = CodeGraph::new();
        
        let function1 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "source".to_string(),
            file_path: "source.rs".into(),
            line_start: 1,
            line_end: 10,
            namespace: "".to_string(),
            language: "rust".to_string(),
            signature: None,
            return_type: None,
            parameters: vec![],
        };

        let function2 = FunctionInfo {
            id: Uuid::new_v4(),
            name: "target".to_string(),
            file_path: "target.rs".into(),
            line_start: 1,
            line_end: 5,
            namespace: "".to_string(),
            language: "rust".to_string(),
            signature: None,
            return_type: None,
            parameters: vec![],
        };

        code_graph.add_function(function1.clone());
        code_graph.add_function(function2.clone());

        let relation = CallRelation {
            caller_id: function1.id,
            callee_id: function2.id,
            caller_name: function1.name.clone(),
            callee_name: function2.name.clone(),
            caller_file: function1.file_path.clone(),
            callee_file: function2.file_path.clone(),
            line_number: 3,
            is_resolved: true,
        };

        code_graph.add_call_relation(relation);

        let dot = code_graph.to_dot();
        
        // 验证DOT输出包含必要的元素
        assert!(dot.contains("digraph CodeGraph"));
        assert!(dot.contains("source"));
        assert!(dot.contains("target"));
        assert!(dot.contains("source -> target"));
    }

    #[test]
    fn test_analyzer_basic_functionality() {
        let temp_dir = create_test_files();
        let mut analyzer = CodeGraphAnalyzer::new();
        
        let result = analyzer.analyze_directory(temp_dir.path());
        assert!(result.is_ok());
        
        let code_graph = analyzer.get_code_graph();
        assert!(code_graph.is_some());
        
        let stats = analyzer.get_stats();
        assert!(stats.is_some());
        
        let stats = stats.unwrap();
        assert!(stats.total_functions > 0);
        assert!(stats.total_files > 0);
    }

    #[test]
    fn test_analyzer_call_chains() {
        let temp_dir = create_test_files();
        let mut analyzer = CodeGraphAnalyzer::new();
        
        analyzer.analyze_directory(temp_dir.path()).unwrap();
        
        let chains = analyzer.find_call_chains("main", 3);
        assert!(!chains.is_empty());
        
        // 验证调用链的深度
        for chain in chains {
            assert!(chain.len() <= 3);
        }
    }

    #[test]
    fn test_analyzer_circular_dependencies() {
        let temp_dir = create_test_files();
        let mut analyzer = CodeGraphAnalyzer::new();
        
        analyzer.analyze_directory(temp_dir.path()).unwrap();
        
        let cycles = analyzer.find_circular_dependencies();
        // 对于简单的测试代码，应该没有循环依赖
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_analyzer_complexity_analysis() {
        let temp_dir = create_test_files();
        let mut analyzer = CodeGraphAnalyzer::new();
        
        analyzer.analyze_directory(temp_dir.path()).unwrap();
        
        let complex_functions = analyzer.find_most_complex_functions(5);
        assert!(!complex_functions.is_empty());
        
        // 验证复杂度排序（降序）
        for i in 1..complex_functions.len() {
            assert!(complex_functions[i-1].1 >= complex_functions[i].1);
        }
    }

    #[test]
    fn test_analyzer_distribution_analysis() {
        let temp_dir = create_test_files();
        let mut analyzer = CodeGraphAnalyzer::new();
        
        analyzer.analyze_directory(temp_dir.path()).unwrap();
        
        let lang_dist = analyzer.get_language_distribution();
        assert!(!lang_dist.is_empty());
        
        let file_dist = analyzer.get_file_distribution();
        assert!(!file_dist.is_empty());
        
        // 验证语言分布包含预期的语言
        assert!(lang_dist.contains_key("rust"));
        assert!(lang_dist.contains_key("python"));
        assert!(lang_dist.contains_key("javascript"));
    }

    #[test]
    fn test_analyzer_report_generation() {
        let temp_dir = create_test_files();
        let mut analyzer = CodeGraphAnalyzer::new();
        
        analyzer.analyze_directory(temp_dir.path()).unwrap();
        
        let report = analyzer.generate_call_report();
        assert!(!report.is_empty());
        
        // 验证报告包含关键信息
        assert!(report.contains("Code Graph Call Report"));
        assert!(report.contains("Total Functions"));
        assert!(report.contains("Language Distribution"));
        assert!(report.contains("File Distribution"));
    }

    #[test]
    fn test_analyzer_export_formats() {
        let temp_dir = create_test_files();
        let mut analyzer = CodeGraphAnalyzer::new();
        
        analyzer.analyze_directory(temp_dir.path()).unwrap();
        
        // 测试Mermaid导出
        let mermaid = analyzer.export_mermaid();
        assert!(mermaid.is_some());
        assert!(!mermaid.unwrap().is_empty());
        
        // 测试DOT导出
        let dot = analyzer.export_dot();
        assert!(dot.is_some());
        assert!(!dot.unwrap().is_empty());
        
        // 测试JSON导出
        let json = analyzer.export_json();
        assert!(json.is_some());
        let json_result = json.unwrap();
        assert!(json_result.is_ok());
    }

    #[test]
    fn test_stats_default() {
        let stats = CodeGraphStats::default();
        
        assert_eq!(stats.total_functions, 0);
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_languages, 0);
        assert_eq!(stats.resolved_calls, 0);
        assert_eq!(stats.unresolved_calls, 0);
    }

    #[test]
    fn test_unsupported_file_handling() {
        let temp_dir = tempfile::tempdir().unwrap();
        
        // 创建不支持的文件类型
        fs::write(temp_dir.path().join("test.txt"), "This is a text file").unwrap();
        fs::write(temp_dir.path().join("test.md"), "# Markdown file").unwrap();
        
        let mut analyzer = CodeGraphAnalyzer::new();
        let result = analyzer.analyze_directory(temp_dir.path());
        
        // 应该能够处理不支持的文件类型（跳过它们）
        assert!(result.is_ok());
        
        let stats = analyzer.get_stats().unwrap();
        assert_eq!(stats.total_functions, 0);
        assert_eq!(stats.total_files, 0);
    }

    #[test]
    fn test_generate_and_print_codegraph() {
        let temp_dir = create_test_files();
        let mut analyzer = CodeGraphAnalyzer::new();
        
        analyzer.analyze_directory(temp_dir.path()).unwrap();
        
        let code_graph = analyzer.get_code_graph().unwrap();
        let stats = code_graph.get_stats();
        
        // 打印统计信息进行验证
        println!("Generated code graph with:");
        println!("  Functions: {}", stats.total_functions);
        println!("  Files: {}", stats.total_files);
        println!("  Languages: {}", stats.total_languages);
        println!("  Resolved calls: {}", stats.resolved_calls);
        println!("  Unresolved calls: {}", stats.unresolved_calls);
        
        // 验证基本统计信息
        assert!(stats.total_functions > 0);
        assert!(stats.total_files > 0);
        assert!(stats.total_languages > 0);
    }
} 