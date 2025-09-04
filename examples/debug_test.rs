use std::path::PathBuf;
use std::fs;
use codegraph_cli::codegraph::parser::CodeParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 CodeGraph 增量构建调试测试");
    println!("==================================");
    
    // 创建测试项目目录
    let project_dir = PathBuf::from("debug_test_project");
    if project_dir.exists() {
        fs::remove_dir_all(&project_dir)?;
    }
    fs::create_dir(&project_dir)?;
    
    // 创建测试文件
    let test_file = project_dir.join("test.rs");
    let content = r#"
pub fn hello() {
    println!("Hello, world!");
}
"#;
    fs::write(&test_file, content)?;
    
    let mut parser = CodeParser::new();
    
    // 第一次构建
    println!("\n📁 第一次构建...");
    let graph1 = parser.build_code_graph(&project_dir)?;
    let stats1 = graph1.get_stats();
    println!("✅ 构建完成！函数数量: {}", stats1.total_functions);
    
    // 检查缓存目录
    let cache_dir = PathBuf::from(".codegraph_db");
    if cache_dir.exists() {
        println!("📁 缓存目录存在");
        
        // 列出所有项目
        for entry in fs::read_dir(&cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                println!("  - {}", path.file_name().unwrap().to_string_lossy());
                
                // 检查是否有图数据
                let graph_file = path.join("graph.json");
                if graph_file.exists() {
                    println!("    ✓ 有图数据");
                }
                
                // 检查是否有哈希文件
                let hash_file = path.join("file_hashes.json");
                if hash_file.exists() {
                    let hash_content = fs::read_to_string(&hash_file)?;
                    println!("    ✓ 有哈希文件: {}", hash_content);
                }
            }
        }
    }
    
    // 修改文件
    println!("\n✏️  修改文件...");
    let new_content = r#"
pub fn hello() {
    println!("Hello, world!");
}

pub fn greet() {
    println!("Hello!");
}
"#;
    fs::write(&test_file, new_content)?;
    
    // 第二次构建
    println!("\n🔄 第二次构建...");
    let graph2 = parser.build_code_graph(&project_dir)?;
    let stats2 = graph2.get_stats();
    println!("✅ 构建完成！函数数量: {}", stats2.total_functions);
    
    // 清理
    fs::remove_dir_all(&project_dir)?;
    
    println!("\n🎉 调试测试完成！");
    Ok(())
} 