use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::analyzers::{RustAnalyzer, CodeAnalyzer};

#[test]
fn test_rust_analyzer_creation() {
    let analyzer = RustAnalyzer::new();
    assert!(analyzer.is_ok(), "Failed to create Rust analyzer: {:?}", analyzer.err());
}

#[test]
fn test_rust_analyzer_analyze_file() {
    let mut analyzer = RustAnalyzer::new().expect("Failed to create Rust analyzer");
    
    // 创建临时Rust文件
    let temp_dir = TempDir::new().unwrap();
    let rust_file = temp_dir.path().join("test.rs");
    
    let rust_content = r#"
        // 简单的Rust测试代码
        pub struct Calculator {
            value: i32,
        }

        impl Calculator {
            pub fn new() -> Self {
                Calculator { value: 0 }
            }
            
            pub fn add(&mut self, x: i32) -> i32 {
                self.value += x;
                self.value
            }
        }

        fn main() {
            let mut calc = Calculator::new();
            let result = calc.add(5);
            println!("Result: {}", result);
        }
    "#;
    
    fs::write(&rust_file, rust_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&rust_file);
    println!("Analysis result: {:?}", result);
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_rust_analyzer_analyze_directory() {
    let mut analyzer = RustAnalyzer::new().expect("Failed to create Rust analyzer");
    
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();
    let rust_dir = temp_dir.path().join("rust");
    fs::create_dir(&rust_dir).unwrap();
    
    // 创建多个Rust文件
    let rust_file1 = rust_dir.join("file1.rs");
    let rust_file2 = rust_dir.join("file2.rs");
    
    let rust_content1 = r#"
        pub fn helper() -> &'static str {
            "help"
        }
    "#;
    
    let rust_content2 = r#"
        pub struct Helper {
            value: String,
        }

        impl Helper {
            pub fn new() -> Self {
                Helper { value: "value".to_string() }
            }
        }
    "#;
    
    fs::write(&rust_file1, rust_content1).unwrap();
    fs::write(&rust_file2, rust_content2).unwrap();
    
    // 分析目录
    let result = analyzer.analyze_directory(&rust_dir);
    println!("Directory analysis result: {:?}", result);
    
    // 清理
    temp_dir.close().unwrap();
} 