use std::path::PathBuf;
use codegraph_cli::codegraph::analyzers::{RustAnalyzer, CodeAnalyzer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Rust Analyzer Demo ===\n");
    
    // 创建Rust分析器
    let mut analyzer = RustAnalyzer::new()?;
    println!("✓ Rust分析器创建成功\n");
    
    // 创建一个示例Rust文件
    let example_code = r#"
        // 示例Rust代码
        use std::collections::HashMap;
        
        /// 简单的计算器结构体
        pub struct Calculator {
            value: i32,
        }
        
        impl Calculator {
            /// 创建新的计算器实例
            pub fn new() -> Self {
                Calculator { value: 0 }
            }
            
            /// 添加数字
            pub fn add(&mut self, x: i32) -> i32 {
                self.value += x;
                self.value
            }
            
            /// 获取当前值
            pub fn get_value(&self) -> i32 {
                self.value
            }
        }
        
        /// 数学运算trait
        pub trait MathOperations {
            fn multiply(&self, x: i32) -> i32;
            fn divide(&self, x: i32) -> Result<i32, String>;
        }
        
        impl MathOperations for Calculator {
            fn multiply(&self, x: i32) -> i32 {
                self.value * x
            }
            
            fn divide(&self, x: i32) -> Result<i32, String> {
                if x == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(self.value / x)
                }
            }
        }
        
        /// 主函数
        fn main() {
            let mut calc = Calculator::new();
            let result = calc.add(5);
            println!("Result: {}", result);
        }
    "#;
    
    // 写入临时文件
    let temp_dir = tempfile::TempDir::new()?;
    let rust_file = temp_dir.path().join("example.rs");
    std::fs::write(&rust_file, example_code)?;
    
    println!("✓ 创建示例Rust文件: {}", rust_file.display());
    
    // 分析文件
    println!("\n开始分析Rust文件...");
    analyzer.analyze_file(&rust_file)?;
    println!("✓ 文件分析完成\n");
    
    // 创建多个文件进行目录分析
    let rust_dir = temp_dir.path().join("rust_project");
    std::fs::create_dir(&rust_dir)?;
    
    let file1 = rust_dir.join("lib.rs");
    let file2 = rust_dir.join("utils.rs");
    
    let lib_code = r#"
        pub mod utils;
        
        pub fn public_function() -> &'static str {
            "Hello from lib.rs"
        }
    "#;
    
    let utils_code = r#"
        pub fn helper_function() -> i32 {
            42
        }
        
        pub struct Helper {
            value: String,
        }
        
        impl Helper {
            pub fn new() -> Self {
                Helper { value: "helper".to_string() }
            }
        }
    "#;
    
    std::fs::write(&file1, lib_code)?;
    std::fs::write(&file2, utils_code)?;
    
    println!("✓ 创建Rust项目目录: {}", rust_dir.display());
    
    // 分析目录
    println!("\n开始分析Rust项目目录...");
    analyzer.analyze_directory(&rust_dir)?;
    println!("✓ 目录分析完成\n");
    
    println!("=== 演示完成 ===");
    println!("Rust分析器成功分析了:");
    println!("  - 单个Rust文件");
    println!("  - Rust项目目录");
    println!("  - 包含结构体、trait、impl块等Rust语法结构");
    
    Ok(())
} 