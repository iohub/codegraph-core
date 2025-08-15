use std::path::Path;
use codegraph_cli::codegraph::PythonAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    println!("=== Python Code Analysis Example ===\n");
    
    // 创建Python分析器
    let mut analyzer = PythonAnalyzer::new()?;
    
    // 分析当前目录
    let current_dir = Path::new(".");
    println!("Analyzing Python files in: {}", current_dir.display());
    
    analyzer.analyze_directory(current_dir)?;
    
    // 生成分析报告
    let report = analyzer.generate_report();
    println!("{}", report);
    
    // 获取所有函数信息
    let functions = analyzer.get_all_functions();
    println!("\nFound {} functions:", functions.len());
    
    for function in functions {
        println!("  - {} ({}:{}-{})", 
            function.name, 
            function.file_path.display(), 
            function.line_start, 
            function.line_end);
    }
    
    Ok(())
} 