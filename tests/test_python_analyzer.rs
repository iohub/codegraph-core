use std::path::Path;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::PythonAnalyzer;

#[test]
fn test_python_analyzer_new() {
    let analyzer = PythonAnalyzer::new();
    assert!(analyzer.is_ok());
}

#[test]
fn test_python_analyzer_analyze_file() {
    let mut analyzer = PythonAnalyzer::new().unwrap();
    
    // 创建临时Python文件
    let temp_dir = TempDir::new().unwrap();
    let py_file = temp_dir.path().join("test.py");
    
    let py_content = r#"
        # 函数定义
        def greet(name: str) -> str:
            return f"Hello, {name}!"
        
        # 类定义
        class UserService:
            def __init__(self, api_url: str):
                self.api_url = api_url
            
            async def get_users(self):
                import aiohttp
                async with aiohttp.ClientSession() as session:
                    async with session.get(self.api_url) as response:
                        return await response.json()
        
        # 装饰器函数
        def log_function(func):
            def wrapper(*args, **kwargs):
                print(f"Calling {func.__name__}")
                return func(*args, **kwargs)
            return wrapper
        
        # 使用装饰器
        @log_function
        def multiply(a: int, b: int) -> int:
            return a * b
        
        # 类型别名
        from typing import List, Dict, Optional
        
        UserList = List[Dict[str, str]]
        
        def process_users(users: UserList) -> List[str]:
            return [user.get('name', 'Unknown') for user in users]
    "#;
    
    fs::write(&py_file, py_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&py_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    assert!(functions.len() >= 4); // 至少应该有4个函数
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_python_analyzer_analyze_directory() {
    let mut analyzer = PythonAnalyzer::new().unwrap();
    
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();
    let py_dir = temp_dir.path().join("python");
    fs::create_dir(&py_dir).unwrap();
    
    // 创建多个Python文件
    let py_file1 = py_dir.join("main.py");
    let py_file2 = py_dir.join("utils.py");
    
    let py_content1 = r#"
        def main():
            print("Hello, World!")
        
        if __name__ == "__main__":
            main()
    "#;
    
    let py_content2 = r#"
        def helper():
            return "help"
        
        class Helper:
            @staticmethod
            def get_value():
                return "value"
    "#;
    
    fs::write(&py_file1, py_content1).unwrap();
    fs::write(&py_file2, py_content2).unwrap();
    
    // 分析目录
    let result = analyzer.analyze_directory(temp_dir.path());
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    assert!(functions.len() >= 3); // 至少应该有3个函数
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_python_analyzer_generate_report() {
    let mut analyzer = PythonAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let py_file = temp_dir.path().join("report.py");
    
    let py_content = r#"
        def test_function():
            return "test"
        
        class TestClass:
            def __init__(self):
                pass
    "#;
    
    fs::write(&py_file, py_content).unwrap();
    
    analyzer.analyze_file(&py_file).unwrap();
    
    let report = analyzer.generate_report();
    
    // 检查报告内容
    assert!(report.contains("Python Code Analysis Report"));
    assert!(report.contains("Total Snippets:"));
    assert!(report.contains("=== Functions ==="));
    assert!(report.contains("=== Classes ==="));
    
    temp_dir.close().unwrap();
} 