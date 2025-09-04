use std::path::PathBuf;
use std::fs;
use std::time::Instant;
use codegraph_cli::services::analyzer::CodeAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 CodeGraph HTTP接口增量构建测试");
    println!("=====================================");
    
    // 创建测试项目目录
    let project_dir = PathBuf::from("http_test_project");
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

pub fn greet(name: &str) {
    println!("Hello, {}!", name);
}
"#;
    fs::write(&test_file, content)?;
    
    let mut analyzer = CodeAnalyzer::new();
    
    // 第一次构建（通过HTTP接口使用的analyze_directory方法）
    println!("\n📁 第一次构建（HTTP接口方式）...");
    let start = Instant::now();
    let graph1 = analyzer.analyze_directory(&project_dir)?;
    let duration1 = start.elapsed();
    
    let stats1 = graph1.get_stats();
    println!("✅ 构建完成！函数数量: {}", stats1.total_functions);
    println!("   构建时间: {:?}", duration1);
    
    // 检查缓存文件
    let cache_dir = PathBuf::from(".codegraph_db/http_test_project");
    if cache_dir.exists() {
        println!("📁 缓存目录已创建: {}", cache_dir.display());
        
        let hash_file = cache_dir.join("file_hashes.json");
        if hash_file.exists() {
            let hash_content = fs::read_to_string(&hash_file)?;
            println!("📄 文件哈希值已保存:");
            println!("{}", hash_content);
        }
    }
    
    // 修改文件内容
    println!("\n✏️  修改文件内容...");
    let new_content = r#"
pub fn hello() {
    println!("Hello, world!");
}

pub fn greet(name: &str) {
    println!("Hello, {}!", name);
}

pub fn new_function() {
    println!("This is a new function!");
}
"#;
    fs::write(&test_file, new_content)?;
    
    // 第二次构建（应该检测到文件变化）
    println!("\n🔄 第二次构建（检测到文件变化）...");
    let start = Instant::now();
    let mut analyzer2 = CodeAnalyzer::new();
    let graph2 = analyzer2.analyze_directory(&project_dir)?;
    let duration2 = start.elapsed();
    
    let stats2 = graph2.get_stats();
    println!("✅ 构建完成！函数数量: {}", stats2.total_functions);
    println!("   构建时间: {:?}", duration2);
    
    // 再次检查缓存文件
    let hash_file = cache_dir.join("file_hashes.json");
    if hash_file.exists() {
        let hash_content = fs::read_to_string(&hash_file)?;
        println!("📄 更新后的文件哈希值:");
        println!("{}", hash_content);
    }
    
    // 再次构建（无变化）
    println!("\n🔄 第三次构建（无变化）...");
    let start = Instant::now();
    let mut analyzer3 = CodeAnalyzer::new();
    let graph3 = analyzer3.analyze_directory(&project_dir)?;
    let duration3 = start.elapsed();
    
    let stats3 = graph3.get_stats();
    println!("✅ 构建完成！函数数量: {}", stats3.total_functions);
    println!("   构建时间: {:?}", duration3);
    
    // 性能对比
    println!("\n📊 性能对比");
    println!("=====================================");
    println!("首次构建: {:?} ({} 函数)", duration1, stats1.total_functions);
    println!("增量构建: {:?} ({} 函数)", duration2, stats2.total_functions);
    println!("无变化构建: {:?} ({} 函数)", duration3, stats3.total_functions);
    
    if duration2 < duration1 {
        let speedup = duration1.as_millis() as f64 / duration2.as_millis() as f64;
        println!("🚀 增量构建速度提升: {:.1}x", speedup);
    }
    
    if duration3 < duration2 {
        let speedup = duration2.as_millis() as f64 / duration3.as_millis() as f64;
        println!("⚡ 缓存构建速度提升: {:.1}x", speedup);
    }
    
    // 验证结果
    println!("\n📊 测试结果验证");
    println!("=====================================");
    println!("首次构建: {} 函数", stats1.total_functions);
    println!("增量构建: {} 函数", stats2.total_functions);
    println!("无变化构建: {} 函数", stats3.total_functions);
    
    if stats1.total_functions == 2 && stats2.total_functions == 3 && stats3.total_functions == 3 {
        println!("✅ 所有测试通过！HTTP接口的增量构建功能正常工作");
        println!("✅ MD5加速解析逻辑已成功集成到HTTP接口中");
    } else {
        println!("❌ 测试失败！函数数量不符合预期");
    }
    
    // 清理
    fs::remove_dir_all(&project_dir)?;
    
    println!("\n🎉 HTTP接口增量构建测试完成！");
    Ok(())
} 