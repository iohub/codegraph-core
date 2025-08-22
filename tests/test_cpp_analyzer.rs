use std::path::Path;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::analyzers::CppAnalyzer;

#[test]
fn test_cpp_analyzer_new() {
    let analyzer = CppAnalyzer::new();
    assert!(analyzer.is_ok());
}

#[test]
fn test_cpp_analyzer_analyze_file_with_simple_function() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时C++文件
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("simple_function.cpp");
    
    let cpp_content = r#"
        #include <iostream>
        #include <string>
        
        int add(int a, int b) {
            return a + b;
        }
        
        void printMessage(const std::string& message) {
            std::cout << message << std::endl;
        }
        
        int main() {
            int result = add(5, 3);
            printMessage("Hello, World!");
            return 0;
        }
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&cpp_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Found {} functions: {:?}", functions.len(), 
        functions.iter().map(|f| &f.name).collect::<Vec<_>>());
    assert!(functions.len() >= 3); // 3个函数
    
    // 验证特定函数
    let add_functions = analyzer.find_functions_by_name("add");
    assert!(!add_functions.is_empty());
    
    let main_functions = analyzer.find_functions_by_name("main");
    assert!(!main_functions.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_analyze_file_with_class() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时C++文件
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("simple_class.cpp");
    
    let cpp_content = r#"
        #include <string>
        
        class Calculator {
        private:
            int value;
            
        public:
            Calculator(int initial_value) : value(initial_value) {}
            
            int add(int x) {
                value += x;
                return value;
            }
            
            int subtract(int x) {
                value -= x;
                return value;
            }
            
            int getValue() const {
                return value;
            }
        };
        
        int main() {
            Calculator calc(10);
            calc.add(5);
            calc.subtract(2);
            return calc.getValue();
        }
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&cpp_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Found {} functions: {:?}", functions.len(), 
        functions.iter().map(|f| &f.name).collect::<Vec<_>>());
    assert!(functions.len() >= 5); // 构造函数 + 3个方法 + main函数
    
    // 验证特定函数
    let add_functions = analyzer.find_functions_by_name("add");
    assert!(!add_functions.is_empty());
    
    let constructor_functions = analyzer.find_functions_by_name("Calculator");
    assert!(!constructor_functions.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_analyze_file_with_namespace() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时C++文件
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("namespace_test.cpp");
    
    let cpp_content = r#"
        #include <iostream>
        
        namespace Math {
            int add(int a, int b) {
                return a + b;
            }
            
            int multiply(int a, int b) {
                return a * b;
            }
            
            namespace Utils {
                void printResult(int result) {
                    std::cout << "Result: " << result << std::endl;
                }
            }
        }
        
        int main() {
            int sum = Math::add(5, 3);
            int product = Math::multiply(4, 6);
            Math::Utils::printResult(sum);
            Math::Utils::printResult(product);
            return 0;
        }
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&cpp_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Found {} functions: {:?}", functions.len(), 
        functions.iter().map(|f| &f.name).collect::<Vec<_>>());
    assert!(functions.len() >= 4); // 4个函数
    
    // 验证特定函数
    let add_functions = analyzer.find_functions_by_name("add");
    assert!(!add_functions.is_empty());
    
    let multiply_functions = analyzer.find_functions_by_name("multiply");
    assert!(!multiply_functions.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_analyze_file_with_templates() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时C++文件
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("template_test.cpp");
    
    let cpp_content = r#"
        #include <iostream>
        
        template<typename T>
        T max(T a, T b) {
            return (a > b) ? a : b;
        }
        
        template<typename T>
        class Container {
        private:
            T data;
            
        public:
            Container(T value) : data(value) {}
            
            T getValue() const {
                return data;
            }
            
            void setValue(T value) {
                data = value;
            }
        };
        
        int main() {
            int max_int = max(5, 10);
            double max_double = max(3.14, 2.71);
            
            Container<int> int_container(42);
            Container<std::string> string_container("Hello");
            
            return 0;
        }
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&cpp_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Found {} functions: {:?}", functions.len(), 
        functions.iter().map(|f| &f.name).collect::<Vec<_>>());
    assert!(functions.len() >= 4); // 至少4个函数
    
    // 验证特定函数
    let max_functions = analyzer.find_functions_by_name("max");
    assert!(!max_functions.is_empty());
    
    let get_value_functions = analyzer.find_functions_by_name("getValue");
    assert!(!get_value_functions.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_analyze_file_with_struct() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时C++文件
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("struct_test.cpp");
    
    let cpp_content = r#"
        #include <string>
        
        struct Point {
            int x;
            int y;
            
            Point(int x, int y) : x(x), y(y) {}
            
            int distance() const {
                return x * x + y * y;
            }
        };
        
        struct Rectangle {
            Point topLeft;
            Point bottomRight;
            
            Rectangle(Point tl, Point br) : topLeft(tl), bottomRight(br) {}
            
            int area() const {
                int width = bottomRight.x - topLeft.x;
                int height = bottomRight.y - topLeft.y;
                return width * height;
            }
        };
        
        int main() {
            Point p1(0, 0);
            Point p2(3, 4);
            Rectangle rect(p1, p2);
            return rect.area();
        }
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&cpp_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Found {} functions: {:?}", functions.len(), 
        functions.iter().map(|f| &f.name).collect::<Vec<_>>());
    assert!(functions.len() >= 4); // 至少4个函数
    
    // 验证特定函数
    let distance_functions = analyzer.find_functions_by_name("distance");
    assert!(!distance_functions.is_empty());
    
    let area_functions = analyzer.find_functions_by_name("area");
    assert!(!area_functions.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_analyze_directory() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时目录和多个C++文件
    let temp_dir = TempDir::new().unwrap();
    
    let file1 = temp_dir.path().join("file1.cpp");
    let file2 = temp_dir.path().join("file2.cpp");
    let file3 = temp_dir.path().join("subdir").join("file3.cpp");
    
    // 创建子目录
    fs::create_dir_all(temp_dir.path().join("subdir")).unwrap();
    
    let content1 = r#"
        int function1() { return 1; }
        int function2() { return 2; }
    "#;
    
    let content2 = r#"
        int function3() { return 3; }
        int function4() { return 4; }
    "#;
    
    let content3 = r#"
        int function5() { return 5; }
        int function6() { return 6; }
    "#;
    
    fs::write(&file1, content1).unwrap();
    fs::write(&file2, content2).unwrap();
    fs::write(&file3, content3).unwrap();
    
    // 分析目录
    let result = analyzer.analyze_directory(temp_dir.path());
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Found {} functions in directory", functions.len());
    assert!(functions.len() >= 6); // 至少6个函数
    
    // 验证特定文件
    let file1_functions = analyzer.get_file_functions(&file1);
    assert!(file1_functions.is_some());
    assert!(file1_functions.unwrap().len() >= 2);
    
    let file2_functions = analyzer.get_file_functions(&file2);
    assert!(file2_functions.is_some());
    assert!(file2_functions.unwrap().len() >= 2);
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_generate_report() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时C++文件
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("report_test.cpp");
    
    let cpp_content = r#"
        #include <iostream>
        
        void testFunction1() {
            std::cout << "Test 1" << std::endl;
        }
        
        void testFunction2() {
            std::cout << "Test 2" << std::endl;
        }
        
        int main() {
            testFunction1();
            testFunction2();
            return 0;
        }
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&cpp_file);
    assert!(result.is_ok());
    
    // 生成报告
    let report = analyzer.generate_report();
    println!("Generated report:\n{}", report);
    
    // 验证报告内容
    assert!(report.contains("Total Functions: 3"));
    assert!(report.contains("testFunction1"));
    assert!(report.contains("testFunction2"));
    assert!(report.contains("main"));
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_error_handling() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 测试分析不存在的文件
    let result = analyzer.analyze_file(Path::new("nonexistent_file.cpp"));
    assert!(result.is_err());
    
    // 测试分析空文件
    let temp_dir = TempDir::new().unwrap();
    let empty_file = temp_dir.path().join("empty.cpp");
    fs::write(&empty_file, "").unwrap();
    
    let result = analyzer.analyze_file(&empty_file);
    // 空文件应该能解析，但可能没有函数
    assert!(result.is_ok());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_cpp_analyzer_advanced_features() {
    let mut analyzer = CppAnalyzer::new().unwrap();
    
    // 创建临时C++文件
    let temp_dir = TempDir::new().unwrap();
    let cpp_file = temp_dir.path().join("advanced_features.cpp");
    
    let cpp_content = r#"
        #include <iostream>
        #include <vector>
        #include <algorithm>
        
        // Lambda 表达式
        auto lambda_function = [](int x) -> int {
            return x * 2;
        };
        
        // 运算符重载
        class Complex {
        private:
            double real, imag;
            
        public:
            Complex(double r, double i) : real(r), imag(i) {}
            
            Complex operator+(const Complex& other) const {
                return Complex(real + other.real, imag + other.imag);
            }
            
            Complex operator-(const Complex& other) const {
                return Complex(real - other.real, imag - other.imag);
            }
            
            friend std::ostream& operator<<(std::ostream& os, const Complex& c) {
                os << c.real << " + " << c.imag << "i";
                return os;
            }
        };
        
        // 模板特化
        template<typename T>
        T max_value(T a, T b) {
            return (a > b) ? a : b;
        }
        
        template<>
        int max_value<int>(int a, int b) {
            return (a > b) ? a : b;
        }
        
        // 移动语义
        class Resource {
        private:
            int* data;
            
        public:
            Resource(int size) : data(new int[size]) {}
            
            Resource(Resource&& other) noexcept : data(other.data) {
                other.data = nullptr;
            }
            
            Resource& operator=(Resource&& other) noexcept {
                if (this != &other) {
                    delete[] data;
                    data = other.data;
                    other.data = nullptr;
                }
                return *this;
            }
            
            ~Resource() {
                delete[] data;
            }
        };
        
        // 变参模板
        template<typename... Args>
        void print_all(Args... args) {
            (std::cout << ... << args) << std::endl;
        }
        
        int main() {
            // 使用 lambda
            std::vector<int> numbers = {1, 2, 3, 4, 5};
            std::transform(numbers.begin(), numbers.end(), numbers.begin(), lambda_function);
            
            // 使用运算符重载
            Complex c1(1.0, 2.0);
            Complex c2(3.0, 4.0);
            Complex c3 = c1 + c2;
            std::cout << c3 << std::endl;
            
            // 使用模板特化
            int max_int = max_value(5, 10);
            double max_double = max_value(3.14, 2.71);
            
            // 使用变参模板
            print_all("Hello", " ", "World", " ", 42);
            
            return 0;
        }
    "#;
    
    fs::write(&cpp_file, cpp_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&cpp_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Found {} functions in advanced features test: {:?}", 
        functions.len(), 
        functions.iter().map(|f| &f.name).collect::<Vec<_>>());
    
    // 应该能识别出多个函数
    assert!(functions.len() >= 8); // 至少8个函数
    
    // 验证特定函数
    let main_functions = analyzer.find_functions_by_name("main");
    assert!(!main_functions.is_empty());
    
    let max_functions = analyzer.find_functions_by_name("max_value");
    assert!(!max_functions.is_empty());
    
    let print_functions = analyzer.find_functions_by_name("print_all");
    assert!(!print_functions.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}
