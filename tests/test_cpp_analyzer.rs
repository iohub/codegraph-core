use std::path::Path;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::CppAnalyzer;

#[test]
fn test_cpp_analyzer_new() {
    let analyzer = CppAnalyzer::new();
    assert!(analyzer.is_ok());
}

#[test]
fn test_cpp_analyzer_analyze_file() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时C++文件
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("test.cpp");
    
    let cpp_content = r#"
        #include <iostream>
        #include <string>
        #include <vector>
        
        // 命名空间
        namespace math {
            // 函数声明
            int add(int a, int b) {
                return a + b;
            }
            
            // 模板函数
            template<typename T>
            T multiply(T a, T b) {
                return a * b;
            }
        }
        
        // 类定义
        class Calculator {
        private:
            double result;
            std::vector<std::string> history;
            
        public:
            // 构造函数
            Calculator() : result(0.0) {}
            
            // 成员函数
            double add(double a, double b) {
                result = a + b;
                history.push_back("add operation");
                return result;
            }
            
            double subtract(double a, double b) {
                result = a - b;
                history.push_back("subtract operation");
                return result;
            }
            
            // 获取结果
            double getResult() const {
                return result;
            }
            
            // 获取历史记录
            const std::vector<std::string>& getHistory() const {
                return history;
            }
        };
        
        // 主函数
        int main() {
            Calculator calc;
            double sum = calc.add(10.5, 5.3);
            std::cout << "Sum: " << sum << std::endl;
            
            return 0;
        }
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&cpp_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    assert!(functions.len() >= 4); // 至少应该有4个函数
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_analyze_directory() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();
    let cpp_dir = temp_dir.path().join("cpp");
    fs::create_dir(&cpp_dir).unwrap();
    
    // 创建多个C++文件
    let cpp_file1 = cpp_dir.join("main.cpp");
    let cpp_file2 = cpp_dir.join("utils.cpp");
    
    let cpp_content1 = r#"
        #include <iostream>
        
        int main() {
            std::cout << "Hello, World!" << std::endl;
            return 0;
        }
    "#;
    
    let cpp_content2 = r#"
        #include <string>
        
        std::string helper() {
            return "help";
        }
        
        class Helper {
        public:
            static std::string getValue() {
                return "value";
            }
        };
    "#;
    
    fs::write(&cpp_file1, cpp_content1).unwrap();
    fs::write(&cpp_file2, cpp_content2).unwrap();
    
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
fn test_cpp_analyzer_find_functions_by_name() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("search.cpp");
    
    let cpp_content = r#"
        #include <iostream>
        
        void helper() {
            std::cout << "Helper function" << std::endl;
        }
        
        int main() {
            helper();
            return 0;
        }
        
        void another_helper() {
            std::cout << "Another helper" << std::endl;
        }
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    analyzer.analyze_file(&cpp_file).unwrap();
    
    // 查找特定函数
    let helper_functions = analyzer.find_functions_by_name("helper");
    assert!(helper_functions.len() >= 1);
    
    let main_functions = analyzer.find_functions_by_name("main");
    assert!(main_functions.len() >= 1);
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_generate_report() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("report.cpp");
    
    let cpp_content = r#"
        #include <iostream>
        
        void test_function() {
            std::cout << "test" << std::endl;
        }
        
        class TestClass {
        public:
            TestClass() {}
        };
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    analyzer.analyze_file(&cpp_file).unwrap();
    
    let report = analyzer.generate_report();
    
    // 检查报告内容
    assert!(report.contains("C++ Code Analysis Report"));
    assert!(report.contains("Total Snippets:"));
    assert!(report.contains("=== Functions ==="));
    assert!(report.contains("=== Classes ==="));
    
    temp_dir.close().unwrap();
} 