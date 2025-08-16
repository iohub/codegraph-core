use std::path::Path;
use codegraph_cli::codegraph::JavaScriptAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    println!("=== JavaScript Code Analysis Example ===\n");
    
    // 创建JavaScript分析器
    let mut analyzer = JavaScriptAnalyzer::new()?;
    
    // 分析当前目录
    let current_dir = Path::new(".");
    println!("Analyzing JavaScript files in: {}", current_dir.display());
    
    analyzer.analyze_directory(current_dir)?;
    
    // 生成分析报告
    let report = analyzer.generate_report();
    println!("{}", report);
    
    // 获取所有函数信息
    let functions = analyzer.get_all_functions();
    println!("\nFound {} functions:", functions.len());
    
    for function in functions {
        println!("  - {} ({}:{}-{}) [Type: {:?}]", 
            function.name, 
            function.file_path, 
            function.start_line, 
            function.end_line,
            function.snippet_type);
    }
    
    // 获取所有类信息
    let classes = analyzer.get_all_classes();
    println!("\nFound {} classes:", classes.len());
    
    for class in classes {
        println!("  - {} ({}:{}-{})", 
            class.name, 
            class.file_path, 
            class.start_line, 
            class.end_line);
    }
    
    // 获取所有对象信息
    let objects = analyzer.get_all_objects();
    println!("\nFound {} objects:", objects.len());
    
    for object in objects {
        println!("  - {} ({}:{}-{})", 
            object.name, 
            object.file_path, 
            object.start_line, 
            object.end_line);
    }
    
    // 获取分析结果
    let result = analyzer.get_analysis_result();
    println!("\n=== Detailed Analysis ===");
    println!("Total snippets: {}", result.snippets.len());
    println!("Total function calls: {}", result.function_calls.len());
    println!("Total scopes: {}", result.scopes.len());
    println!("Total imports: {}", result.imports.len());
    println!("Total exports: {}", result.exports.len());
    
    Ok(())
} 