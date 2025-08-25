use std::path::Path;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::TypeScriptAnalyzer;

#[test]
fn test_typescript_analyzer_new() {
    let analyzer = TypeScriptAnalyzer::new();
    assert!(analyzer.is_ok());
}

#[test]
fn test_typescript_analyzer_analyze_file() {
    let mut analyzer = TypeScriptAnalyzer::new().unwrap();
    
    // 创建临时TypeScript文件
    let temp_dir = TempDir::new().unwrap();
    let ts_file = temp_dir.path().join("test.ts");
    
    let ts_content = r#"
        // 接口定义
        interface User {
            id: number;
            name: string;
            email: string;
        }
        
        // 函数声明
        function greet(name: string): string {
            return `Hello, ${name}!`;
        }
        
        // 箭头函数
        const multiply = (a: number, b: number): number => a * b;
        
        // 类定义
        class UserService {
            private apiUrl: string;
            
            constructor(apiUrl: string) {
                this.apiUrl = apiUrl;
            }
            
            async getUsers(): Promise<User[]> {
                const response = await fetch(this.apiUrl);
                return response.json();
            }
        }
        
        // 类型别名
        type Status = 'active' | 'inactive' | 'pending';
        
        // 泛型函数
        function identity<T>(arg: T): T {
            return arg;
        }
    "#;
    
    fs::write(&ts_file, ts_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&ts_file);
    println!("Analysis result: {:?}", result);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Found {} functions:", functions.len());
    for (i, func) in functions.iter().enumerate() {
        println!("  {}: {} ({}:{}-{})", i + 1, func.name, func.file_path.display(), func.line_start, func.line_end);
    }
    
    assert!(functions.len() >= 3); // 至少应该有3个函数
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_typescript_analyzer_analyze_directory() {
    let mut analyzer = TypeScriptAnalyzer::new().unwrap();
    
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();
    let ts_dir = temp_dir.path().join("ts");
    fs::create_dir(&ts_dir).unwrap();
    
    // 创建多个TypeScript文件
    let ts_file1 = ts_dir.join("file1.ts");
    let ts_file2 = ts_dir.join("file2.ts");
    
    let ts_content1 = r#"
        export function helper(): string {
            return "help";
        }
    "#;
    
    let ts_content2 = r#"
        export class Helper {
            static getValue(): string {
                return "value";
            }
        }
    "#;
    
    fs::write(&ts_file1, ts_content1).unwrap();
    fs::write(&ts_file2, ts_content2).unwrap();
    
    // 分析目录
    let result = analyzer.analyze_directory(temp_dir.path());
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    assert!(functions.len() >= 2); // 至少应该有2个函数
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_typescript_analyzer_generate_report() {
    let mut analyzer = TypeScriptAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let ts_file = temp_dir.path().join("report.ts");
    
    let ts_content = r#"
        function testFunction(): string {
            return "test";
        }
        
        class TestClass {
            constructor() {}
        }
    "#;
    
    fs::write(&ts_file, ts_content).unwrap();
    
    analyzer.analyze_file(&ts_file).unwrap();
    
    let report = analyzer.generate_report();
    
    // 检查报告内容
    assert!(report.contains("TypeScript Code Analysis Report"));
    assert!(report.contains("Total Snippets:"));
    assert!(report.contains("=== Functions ==="));
    assert!(report.contains("=== Classes ==="));
    
    temp_dir.close().unwrap();
} 

#[test]
fn test_typescript_analyzer_minimal() {
    let mut analyzer = TypeScriptAnalyzer::new().unwrap();
    
    // 创建临时TypeScript文件
    let temp_dir = TempDir::new().unwrap();
    let ts_file = temp_dir.path().join("test.ts");
    
    let ts_content = r#"
        function greet(name: string): string {
            return `Hello, ${name}!`;
        }
    "#;
    
    fs::write(&ts_file, ts_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&ts_file);
    println!("Analysis result: {:?}", result);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Found {} functions:", functions.len());
    for (i, func) in functions.iter().enumerate() {
        println!("  {}: {} ({}:{}-{})", i + 1, func.name, func.file_path.display(), func.line_start, func.line_end);
    }
    
    assert!(functions.len() >= 1); // 至少应该有1个函数
    
    // 清理
    temp_dir.close().unwrap();
} 