use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use std::fs;

use codegraph_cli::storage::StorageManager;
use codegraph_cli::services::CodeAnalyzer;
use codegraph_cli::codegraph::types::PetCodeGraph;
use uuid::Uuid;

/// 测试构建图功能
#[test]
fn test_build_graph_functionality() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Arc::new(StorageManager::new());
    
    // 测试Rust项目
    test_build_graph_for_project(&storage, "tests/test_repos/simple_rust_project");
    
    // 测试Python项目
    test_build_graph_for_project(&storage, "tests/test_repos/simple_python_project");
    
    // 测试JavaScript项目
    test_build_graph_for_project(&storage, "tests/test_repos/simple_js_project");
}

fn test_build_graph_for_project(storage: &Arc<StorageManager>, project_path: &str) {
    println!("Testing build_graph for project: {}", project_path);
    
    let project_dir = PathBuf::from(project_path);
    assert!(project_dir.exists(), "Project directory {} does not exist", project_path);
    
    // 创建CodeAnalyzer实例
    let mut analyzer = CodeAnalyzer::new();
    
    // 分析目录并构建代码图
    let result = analyzer.analyze_directory(&project_dir);
    assert!(result.is_ok(), "Failed to analyze directory: {:?}", result.err());
    
    // 获取代码图
    let code_graph = analyzer.get_code_graph();
    assert!(code_graph.is_some(), "Code graph should be available after analysis");
    
    let code_graph = code_graph.unwrap();
    
    // 验证代码图包含函数
    assert!(!code_graph.functions.is_empty(), "Code graph should contain functions");
    
    // 验证代码图包含调用关系
    assert!(!code_graph.call_relations.is_empty(), "Code graph should contain call relations");
    
    // 获取统计信息
    let stats = analyzer.get_stats();
    assert!(stats.is_some(), "Stats should be available after analysis");
    
    let stats = stats.unwrap();
    println!("Project {}: {} files, {} functions", 
             project_path, stats.total_files, stats.total_functions);
    
    // 验证统计信息
    assert!(stats.total_files > 0, "Total files should be greater than 0");
    assert!(stats.total_functions > 0, "Total functions should be greater than 0");
    
    // 测试转换为PetCodeGraph
    let mut pet_graph = PetCodeGraph::new();
    
    // 添加所有函数
    for function in code_graph.functions.values() {
        pet_graph.add_function(function.clone());
    }
    
    // 添加所有调用关系
    let mut successful_relations = 0;
    for relation in &code_graph.call_relations {
        if let Err(e) = pet_graph.add_call_relation(relation.clone()) {
            eprintln!("Failed to add call relation: {}", e);
        } else {
            successful_relations += 1;
        }
    }
    
    println!("Successfully added {}/{} call relations to PetCodeGraph", 
             successful_relations, code_graph.call_relations.len());
    
    // 更新统计信息
    pet_graph.update_stats();
    
    // 验证PetCodeGraph
    let pet_stats = pet_graph.get_stats();
    assert!(pet_stats.total_functions > 0, "PetCodeGraph should contain functions");
    
    println!("PetCodeGraph stats: {} functions", pet_stats.total_functions);
}

/// 测试查询调用图功能
#[test]
fn test_query_call_graph_functionality() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Arc::new(StorageManager::new());
    
    // 测试Rust项目
    test_query_call_graph_for_project(&storage, "tests/test_repos/simple_rust_project");
    
    // 测试Python项目
    test_query_call_graph_for_project(&storage, "tests/test_repos/simple_python_project");
    
    // 测试JavaScript项目
    test_query_call_graph_for_project(&storage, "tests/test_repos/simple_python_project");
}

fn test_query_call_graph_for_project(storage: &Arc<StorageManager>, project_path: &str) {
    println!("Testing query_call_graph for project: {}", project_path);
    
    let project_dir = PathBuf::from(project_path);
    assert!(project_dir.exists(), "Project directory {} does not exist", project_path);
    
    // 首先构建图
    let mut analyzer = CodeAnalyzer::new();
    let result = analyzer.analyze_directory(&project_dir);
    assert!(result.is_ok(), "Failed to analyze directory: {:?}", result.err());
    
    let code_graph = analyzer.get_code_graph().unwrap();
    
    // 转换为PetCodeGraph并保存
    let mut pet_graph = PetCodeGraph::new();
    for function in code_graph.functions.values() {
        pet_graph.add_function(function.clone());
    }
    
    for relation in &code_graph.call_relations {
        let _ = pet_graph.add_call_relation(relation.clone());
    }
    
    pet_graph.update_stats();
    
    // 生成项目ID
    let project_id = format!("{:x}", md5::compute(project_path.as_bytes()));
    
    // 保存图
    let save_result = storage.get_persistence().save_graph(&project_id, &pet_graph);
    assert!(save_result.is_ok(), "Failed to save graph: {:?}", save_result.err());
    
    // 测试查询特定函数
    test_query_function_by_name(&pet_graph, "main");
    test_query_function_by_name(&pet_graph, "process_data");
    test_query_function_by_name(&pet_graph, "calculate_sum");
    
    // 测试查询文件中的所有函数
    test_query_functions_by_file(&pet_graph, project_path);
    
    // 测试调用链扩展
    test_call_chain_expansion(&pet_graph);
}

fn test_query_function_by_name(graph: &PetCodeGraph, function_name: &str) {
    println!("Testing query for function: {}", function_name);
    
    let functions = graph.find_functions_by_name(function_name);
    if !functions.is_empty() {
        println!("Found {} functions with name '{}'", functions.len(), function_name);
        
        for function in &functions {
            let callers = graph.get_callers(&function.id);
            let callees = graph.get_callees(&function.id);
            
            println!("Function {}: {} callers, {} callees", 
                     function.name, callers.len(), callees.len());
            
            // 验证调用关系
            for (caller_func, relation) in &callers {
                println!("  Called by: {} at {}:{}", 
                         caller_func.name, caller_func.file_path.display(), relation.line_number);
            }
            
            for (callee_func, relation) in &callees {
                println!("  Calls: {} at {}:{}", 
                         callee_func.name, callee_func.file_path.display(), relation.line_number);
            }
        }
    } else {
        println!("No functions found with name '{}'", function_name);
    }
}

fn test_query_functions_by_file(graph: &PetCodeGraph, project_path: &str) {
    println!("Testing query for functions in project: {}", project_path);
    
    // 查找项目中的主要源文件
    let project_dir = PathBuf::from(project_path);
    let source_files = find_source_files(&project_dir);
    
    for source_file in source_files {
        let file_functions = graph.find_functions_by_file(&source_file);
        if !file_functions.is_empty() {
            println!("File {}: {} functions", source_file.display(), file_functions.len());
            
            for function in &file_functions {
                let callers = graph.get_callers(&function.id);
                let callees = graph.get_callees(&function.id);
                
                println!("  Function {}: {} callers, {} callees", 
                         function.name, callers.len(), callees.len());
            }
        }
    }
}

fn test_call_chain_expansion(graph: &PetCodeGraph) {
    println!("Testing call chain expansion");
    
    // 查找一些函数来测试调用链
    let test_functions = ["main", "process_data", "calculate_sum"];
    
    for func_name in test_functions {
        let functions = graph.find_functions_by_name(func_name);
        if !functions.is_empty() {
            let function = &functions[0];
            
            // 测试调用者链
            let mut visited = std::collections::HashSet::new();
            let callers_chain = expand_call_chain(graph, &function.id.to_string(), &mut visited, 3, true);
            
            if !callers_chain.is_empty() {
                println!("Callers chain for {}: {} levels", func_name, callers_chain.len());
            }
            
            // 测试被调用者链
            let mut visited = std::collections::HashSet::new();
            let callees_chain = expand_call_chain(graph, &function.id.to_string(), &mut visited, 3, false);
            
            if !callees_chain.is_empty() {
                println!("Callees chain for {}: {} levels", func_name, callees_chain.len());
            }
        }
    }
}

fn expand_call_chain(
    graph: &PetCodeGraph,
    function_id: &str,
    visited: &mut std::collections::HashSet<String>,
    depth: usize,
    is_caller: bool,
) -> Vec<String> {
    if depth == 0 || visited.contains(function_id) {
        return Vec::new();
    }
    
    visited.insert(function_id.to_string());
    let mut chain = Vec::new();
    
    // 解析UUID
    let uuid = match uuid::Uuid::parse_str(function_id) {
        Ok(uuid) => uuid,
        Err(_) => return chain,
    };
    
    let relations = if is_caller {
        graph.get_callers(&uuid)
    } else {
        graph.get_callees(&uuid)
    };
    
    for (func, _) in relations {
        chain.push(func.name.clone());
        let sub_chain = expand_call_chain(graph, &func.id.to_string(), visited, depth - 1, is_caller);
        chain.extend(sub_chain);
    }
    
    chain
}

fn find_source_files(project_dir: &PathBuf) -> Vec<PathBuf> {
    let mut source_files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy();
                    if matches!(ext_str.as_ref(), "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c") {
                        source_files.push(path);
                    }
                }
            }
        }
    }
    
    source_files
}

/// 测试完整的构建和查询流程
#[test]
fn test_complete_build_and_query_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = Arc::new(StorageManager::new());
    
    // 选择Rust项目进行完整测试
    let project_path = "tests/test_repos/simple_rust_project";
    let project_dir = PathBuf::from(project_path);
    
    println!("Testing complete workflow for: {}", project_path);
    
    // 步骤1: 构建图
    let mut analyzer = CodeAnalyzer::new();
    let result = analyzer.analyze_directory(&project_dir);
    assert!(result.is_ok(), "Failed to analyze directory");
    
    let code_graph = analyzer.get_code_graph().unwrap();
    let stats = analyzer.get_stats().unwrap();
    
    println!("Built graph with {} files and {} functions", 
             stats.total_files, stats.total_functions);
    
    // 步骤2: 转换为PetCodeGraph
    let mut pet_graph = PetCodeGraph::new();
    for function in code_graph.functions.values() {
        pet_graph.add_function(function.clone());
    }
    
    for relation in &code_graph.call_relations {
        let _ = pet_graph.add_call_relation(relation.clone());
    }
    
    pet_graph.update_stats();
    
    // 步骤3: 保存图
    let project_id = format!("{:x}", md5::compute(project_path.as_bytes()));
    let save_result = storage.get_persistence().save_graph(&project_id, &pet_graph);
    assert!(save_result.is_ok(), "Failed to save graph");
    
    // 步骤4: 加载图
    let loaded_graph_result = storage.get_persistence().load_graph(&project_id);
    assert!(loaded_graph_result.is_ok(), "Failed to load graph");
    assert!(loaded_graph_result.as_ref().unwrap().is_some(), "Loaded graph should exist");
    
    // 步骤5: 查询调用图
    let loaded_graph = loaded_graph_result.unwrap().unwrap();
    
    // 查询main函数
    let main_functions = loaded_graph.find_functions_by_name("main");
    assert!(!main_functions.is_empty(), "Should find main function");
    
    let main_func = &main_functions[0];
    let callers = loaded_graph.get_callers(&main_func.id);
    let callees = loaded_graph.get_callees(&main_func.id);
    
    println!("Main function: {} callers, {} callees", callers.len(), callees.len());
    
    // 验证main函数调用了其他函数
    assert!(!callees.is_empty(), "Main function should call other functions");
    
    // 查询process_data函数
    let process_functions = loaded_graph.find_functions_by_name("process_data");
    assert!(!process_functions.is_empty(), "Should find process_data function");
    
    let process_func = &process_functions[0];
    let process_callees = loaded_graph.get_callees(&process_func.id);
    
    println!("Process_data function: {} callees", process_callees.len());
    
    // 验证process_data的调用链
    assert!(process_callees.len() >= 3, "process_data should call multiple functions");
    
    println!("Complete workflow test passed!");
} 