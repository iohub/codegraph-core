use std::path::Path;
use codegraph_cli::codegraph::CppAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    println!("=== C++ Code Analysis Example ===\n");
    
    // 创建C++分析器
    let mut analyzer = CppAnalyzer::new()?;
    
    // 分析当前目录
    let current_dir = Path::new(".");
    println!("Analyzing directory: {}", current_dir.display());
    
    analyzer.analyze_directory(current_dir)?;
    
    // 获取分析结果
    let functions = analyzer.get_all_functions();
    println!("\nFound {} functions:", functions.len());
    
    for function in &functions {
        println!("  - {} ({}:{}-{})", 
            function.name, function.file_path.display(), 
            function.line_start, function.line_end);
        
        if !function.namespace.is_empty() {
            println!("    Namespace: {}", function.namespace);
        }
        
        if !function.parameters.is_empty() {
            println!("    Parameters: {}", function.parameters.len());
            for param in &function.parameters {
                println!("      - {}: {:?}", param.name, param.type_name);
            }
        }
    }
    
    // 查找特定函数
    println!("\nSearching for 'main' function:");
    let main_functions = analyzer.find_functions_by_name("main");
    for function in main_functions {
        println!("  Found main function in {}", function.file_path.display());
    }
    
    // 生成详细报告
    println!("\n=== Analysis Report ===");
    let report = analyzer.generate_report();
    println!("{}", report);
    
    Ok(())
} 