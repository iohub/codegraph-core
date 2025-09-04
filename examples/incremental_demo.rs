use std::path::PathBuf;
use std::time::Instant;
use std::fs;
use codegraph_cli::codegraph::parser::CodeParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CodeGraph 增量构建演示");
    println!("================================");
    
    // 创建临时项目目录
    let project_dir = PathBuf::from("demo_project");
    if project_dir.exists() {
        fs::remove_dir_all(&project_dir)?;
    }
    fs::create_dir(&project_dir)?;
    
    // 创建一些测试文件
    create_test_files(&project_dir)?;
    
    let mut parser = CodeParser::new();
    
    // 第一次构建（全量构建）
    println!("\n📁 第一次构建（全量构建）...");
    let start = Instant::now();
    let graph1 = parser.build_petgraph_code_graph(&project_dir)?;
    let duration1 = start.elapsed();
    
    let stats1 = graph1.get_stats();
    println!("✅ 构建完成！");
    println!("   函数数量: {}", stats1.total_functions);
    println!("   构建时间: {:?}", duration1);
    
    // 修改一个文件
    println!("\n✏️  修改文件...");
    let test_file = project_dir.join("src/main.rs");
    let new_content = r#"
pub fn main() {
    println!("Hello, World!");
    greet("Developer");
    calculate_sum(10, 20);
}

pub fn greet(name: &str) {
    println!("Hello, {}!", name);
}

pub fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

pub fn new_function() {
    println!("This is a new function!");
}
"#;
    fs::write(test_file, new_content)?;
    
    // 第二次构建（增量构建）
    println!("\n🔄 第二次构建（增量构建）...");
    let start = Instant::now();
    let graph2 = parser.build_petgraph_code_graph(&project_dir)?;
    let duration2 = start.elapsed();
    
    let stats2 = graph2.get_stats();
    println!("✅ 构建完成！");
    println!("   函数数量: {}", stats2.total_functions);
    println!("   构建时间: {:?}", duration2);
    
    // 再次构建（无变化）
    println!("\n🔄 第三次构建（无变化）...");
    let start = Instant::now();
    let graph3 = parser.build_petgraph_code_graph(&project_dir)?;
    let duration3 = start.elapsed();
    
    let stats3 = graph3.get_stats();
    println!("✅ 构建完成！");
    println!("   函数数量: {}", stats3.total_functions);
    println!("   构建时间: {:?}", duration3);
    
    // 性能对比
    println!("\n📊 性能对比");
    println!("================================");
    println!("首次构建: {:?}", duration1);
    println!("增量构建: {:?}", duration2);
    println!("无变化构建: {:?}", duration3);
    
    if duration2 < duration1 {
        let speedup = duration1.as_millis() as f64 / duration2.as_millis() as f64;
        println!("🚀 增量构建速度提升: {:.1}x", speedup);
    }
    
    if duration3 < duration2 {
        let speedup = duration2.as_millis() as f64 / duration3.as_millis() as f64;
        println!("⚡ 缓存构建速度提升: {:.1}x", speedup);
    }
    
    // 清理
    fs::remove_dir_all(&project_dir)?;
    
    println!("\n🎉 演示完成！");
    Ok(())
}

fn create_test_files(project_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 创建 src 目录
    let src_dir = project_dir.join("src");
    fs::create_dir(&src_dir)?;
    
    // 创建 main.rs
    let main_content = r#"
pub fn main() {
    println!("Hello, World!");
    greet("Developer");
    calculate_sum(10, 20);
}

pub fn greet(name: &str) {
    println!("Hello, {}!", name);
}

pub fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}
"#;
    fs::write(src_dir.join("main.rs"), main_content)?;
    
    // 创建 lib.rs
    let lib_content = r#"
pub mod utils;

pub fn public_function() {
    println!("This is a public function");
    utils::helper_function();
}
"#;
    fs::write(src_dir.join("lib.rs"), lib_content)?;
    
    // 创建 utils.rs
    let utils_content = r#"
pub fn helper_function() {
    println!("This is a helper function");
}

pub fn another_helper() -> i32 {
    42
}
"#;
    fs::write(src_dir.join("utils.rs"), utils_content)?;
    
    Ok(())
} 