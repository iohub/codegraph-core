use std::path::Path;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::analyzers::{RustAnalyzer, CodeAnalyzer};
use codegraph_cli::codegraph::{JavaScriptAnalyzer, TypeScriptAnalyzer, PythonAnalyzer, CppAnalyzer};

#[test]
fn test_multiple_analyzers_workflow() {
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("mixed_project");
    fs::create_dir(&project_dir).unwrap();
    
    // 创建Rust文件
    let rust_file = project_dir.join("src").join("lib.rs");
    fs::create_dir_all(rust_file.parent().unwrap()).unwrap();
    let rust_content = r#"
        pub fn rust_function() -> &'static str {
            "Hello from Rust"
        }
    "#;
    fs::write(&rust_file, rust_content).unwrap();
    
    // 创建JavaScript文件
    let js_file = project_dir.join("frontend").join("app.js");
    fs::create_dir_all(js_file.parent().unwrap()).unwrap();
    let js_content = r#"
        function jsFunction() {
            return "Hello from JavaScript";
        }
    "#;
    fs::write(&js_file, js_content).unwrap();
    
    // 创建TypeScript文件
    let ts_file = project_dir.join("frontend").join("types.ts");
    let ts_content = r#"
        interface User {
            name: string;
            id: number;
        }
        
        function tsFunction(): User {
            return { name: "Test", id: 1 };
        }
    "#;
    fs::write(&ts_file, ts_content).unwrap();
    
    // 创建Python文件
    let py_file = project_dir.join("scripts").join("process.py");
    fs::create_dir_all(py_file.parent().unwrap()).unwrap();
    let py_content = r#"
        def python_function():
            return "Hello from Python"
    "#;
    fs::write(&py_file, py_content).unwrap();
    
    // 创建C++文件
    let cpp_file = project_dir.join("native").join("engine.cpp");
    fs::create_dir_all(cpp_file.parent().unwrap()).unwrap();
    let cpp_content = r#"
        #include <string>
        
        std::string cppFunction() {
            return "Hello from C++";
        }
    "#;
    fs::write(&cpp_file, cpp_content).unwrap();
    
    // 测试Rust分析器
    let mut rust_analyzer = RustAnalyzer::new().expect("Failed to create Rust analyzer");
    let rust_result = rust_analyzer.analyze_file(&rust_file);
    assert!(rust_result.is_ok());
    
    // 测试JavaScript分析器
    let mut js_analyzer = JavaScriptAnalyzer::new().expect("Failed to create JavaScript analyzer");
    let js_result = js_analyzer.analyze_file(&js_file);
    assert!(js_result.is_ok());
    
    // 测试TypeScript分析器
    let mut ts_analyzer = TypeScriptAnalyzer::new().expect("Failed to create TypeScript analyzer");
    let ts_result = ts_analyzer.analyze_file(&ts_file);
    assert!(ts_result.is_ok());
    
    // 测试Python分析器
    let mut py_analyzer = PythonAnalyzer::new().expect("Failed to create Python analyzer");
    let py_result = py_analyzer.analyze_file(&py_file);
    assert!(py_result.is_ok());
    
    // 测试C++分析器
    let mut cpp_analyzer = CppAnalyzer::new().expect("Failed to create C++ analyzer");
    let cpp_result = cpp_analyzer.analyze_file(&cpp_file);
    assert!(cpp_result.is_ok());
    
    // 验证所有分析器都能找到相应的代码片段
    let rust_functions = rust_analyzer.extract_functions(&rust_file.to_path_buf()).unwrap();
    assert!(rust_functions.len() >= 1);
    
    let js_functions = js_analyzer.get_all_functions();
    assert!(js_functions.len() >= 1);
    
    let ts_functions = ts_analyzer.get_all_functions();
    assert!(ts_functions.len() >= 1);
    
    let py_functions = py_analyzer.get_all_functions();
    assert!(py_functions.len() >= 1);
    
    let cpp_functions = cpp_analyzer.get_all_functions();
    assert!(cpp_functions.len() >= 1);
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_analyzer_error_handling() {
    // 测试分析器对不存在文件的处理
    let mut rust_analyzer = RustAnalyzer::new().expect("Failed to create Rust analyzer");
    let non_existent_file = Path::new("/non/existent/file.rs");
    
    let result = rust_analyzer.analyze_file(non_existent_file);
    assert!(result.is_err(), "Should fail on non-existent file");
    
    // 测试分析器对空目录的处理
    let temp_dir = TempDir::new().unwrap();
    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir(&empty_dir).unwrap();
    
    let result = rust_analyzer.analyze_directory(&empty_dir);
    assert!(result.is_ok(), "Should handle empty directory gracefully");
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_analyzer_performance() {
    // 测试分析器处理大量文件的性能
    let temp_dir = TempDir::new().unwrap();
    let large_dir = temp_dir.path().join("large_project");
    fs::create_dir(&large_dir).unwrap();
    
    // 创建多个小文件
    for i in 0..10 {
        let file_path = large_dir.join(format!("file_{}.rs", i));
        let content = format!(r#"
            pub fn function_{}() -> i32 {{
                {}
            }}
        "#, i, i);
        fs::write(file_path, content).unwrap();
    }
    
    let mut rust_analyzer = RustAnalyzer::new().expect("Failed to create Rust analyzer");
    
    // 测量分析时间
    let start = std::time::Instant::now();
    let result = rust_analyzer.analyze_directory(&large_dir);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 5000, "Analysis took too long: {:?}", duration);
    
    // 清理
    temp_dir.close().unwrap();
} 