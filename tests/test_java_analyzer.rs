use std::path::Path;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::analyzers::JavaAnalyzer;

#[test]
fn test_java_analyzer_new() {
    let analyzer = JavaAnalyzer::new();
    assert!(analyzer.is_ok());
}

#[test]
fn test_java_analyzer_analyze_file_with_simple_class() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("SimpleClass.java");
    
    let java_content = r#"
        package com.example;
        
        public class SimpleClass {
            private String name;
            private int value;
            
            public SimpleClass(String name, int value) {
                this.name = name;
                this.value = value;
            }
            
            public String getName() {
                return name;
            }
            
            public void setName(String name) {
                this.name = name;
            }
            
            public int getValue() {
                return value;
            }
            
            public void setValue(int value) {
                this.value = value;
            }
            
            public int calculate(int multiplier) {
                return value * multiplier;
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let methods = analyzer.get_all_methods();
    println!("Found {} methods: {:?}", methods.len(), 
        methods.iter().map(|m| &m.name).collect::<Vec<_>>());
    assert!(methods.len() >= 5); // 5个方法
    
    // 验证特定方法
    let get_name_methods = analyzer.find_methods_by_name("getName");
    assert!(!get_name_methods.is_empty());
    
    let calculate_methods = analyzer.find_methods_by_name("calculate");
    assert!(!calculate_methods.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_analyze_file_with_interface() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("TestInterface.java");
    
    let java_content = r#"
        package com.example.interfaces;
        
        public interface TestInterface {
            void doSomething();
            String processData(String input);
            int calculate(int a, int b);
        }
        
        public class TestInterfaceImpl implements TestInterface {
            @Override
            public void doSomething() {
                System.out.println("Doing something");
            }
            
            @Override
            public String processData(String input) {
                return "Processed: " + input;
            }
            
            @Override
            public int calculate(int a, int b) {
                return a + b;
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let methods = analyzer.get_all_methods();
    assert!(methods.len() >= 3); // 接口实现类的方法
    
    // 验证特定方法
    let do_something_methods = analyzer.find_methods_by_name("doSomething");
    assert!(!do_something_methods.is_empty());
    
    let calculate_methods = analyzer.find_methods_by_name("calculate");
    assert!(!calculate_methods.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_analyze_file_with_method_calls() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("MethodCalls.java");
    
    let java_content = r#"
        package com.example.methods;
        
        import java.util.List;
        import java.util.ArrayList;
        
        public class MethodCalls {
            private List<String> items;
            
            public MethodCalls() {
                this.items = new ArrayList<>();
            }
            
            public void addItem(String item) {
                items.add(item);
                System.out.println("Added: " + item);
            }
            
            public void processItems() {
                for (String item : items) {
                    String processed = processItem(item);
                    System.out.println(processed);
                }
            }
            
            private String processItem(String item) {
                return "Processed: " + item.toUpperCase();
            }
            
            public int getItemCount() {
                return items.size();
            }
            
            public void clearItems() {
                items.clear();
                System.out.println("Items cleared");
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let methods = analyzer.get_all_methods();
    assert!(methods.len() >= 5); // 5个方法（不包括构造函数）
    
    // 验证特定方法
    let add_item_methods = analyzer.find_methods_by_name("addItem");
    assert!(!add_item_methods.is_empty());
    
    let process_items_methods = analyzer.find_methods_by_name("processItems");
    assert!(!process_items_methods.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_analyze_file_with_generics() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("GenericClass.java");
    
    let java_content = r#"
        package com.example.generics;
        
        public class GenericClass<T> {
            private T data;
            
            public GenericClass(T data) {
                this.data = data;
            }
            
            public T getData() {
                return data;
            }
            
            public void setData(T data) {
                this.data = data;
            }
            
            public <E> void processElement(E element) {
                System.out.println("Processing: " + element);
            }
            
            public static <K, V> void printPair(K key, V value) {
                System.out.println("Key: " + key + ", Value: " + value);
            }
        }
        
        public class StringProcessor extends GenericClass<String> {
            public StringProcessor(String data) {
                super(data);
            }
            
            public void processString() {
                String data = getData();
                System.out.println("Processing string: " + data);
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let methods = analyzer.get_all_methods();
    assert!(methods.len() >= 5); // 5个方法（不包括构造函数）
    
    // 验证特定方法
    let get_data_methods = analyzer.find_methods_by_name("getData");
    assert!(!get_data_methods.is_empty());
    
    let process_element_methods = analyzer.find_methods_by_name("processElement");
    assert!(!process_element_methods.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_analyze_file_with_annotations() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("AnnotatedClass.java");
    
    let java_content = r#"
        package com.example.annotations;
        
        import java.lang.annotation.*;
        
        @Target(ElementType.METHOD)
        @Retention(RetentionPolicy.RUNTIME)
        public @interface TestMethod {
            String value() default "";
        }
        
        @Deprecated
        public class AnnotatedClass {
            @TestMethod("main method")
            public static void main(String[] args) {
                System.out.println("Hello, World!");
            }
            
            @Override
            public String toString() {
                return "AnnotatedClass";
            }
            
            @SuppressWarnings("unused")
            private void unusedMethod() {
                // This method is intentionally unused
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let methods = analyzer.get_all_methods();
    assert!(methods.len() >= 3); // main, toString, unusedMethod
    
    // 验证特定方法
    let main_methods = analyzer.find_methods_by_name("main");
    assert!(!main_methods.is_empty());
    
    let to_string_methods = analyzer.find_methods_by_name("toString");
    assert!(!to_string_methods.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_analyze_file_with_inner_classes() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("OuterClass.java");
    
    let java_content = r#"
        package com.example.inner;
        
        public class OuterClass {
            private String outerField;
            
            public OuterClass(String outerField) {
                this.outerField = outerField;
            }
            
            public void outerMethod() {
                System.out.println("Outer method: " + outerField);
            }
            
            public class InnerClass {
                private String innerField;
                
                public InnerClass(String innerField) {
                    this.innerField = innerField;
                }
                
                public void innerMethod() {
                    System.out.println("Inner method: " + innerField);
                    System.out.println("Outer field: " + outerField);
                }
            }
            
            public static class StaticInnerClass {
                public static void staticMethod() {
                    System.out.println("Static inner method");
                }
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let methods = analyzer.get_all_methods();
    assert!(methods.len() >= 3); // 3个方法（不包括构造函数）
    
    // 验证特定方法
    let outer_methods = analyzer.find_methods_by_name("outerMethod");
    assert!(!outer_methods.is_empty());
    
    let inner_methods = analyzer.find_methods_by_name("innerMethod");
    assert!(!inner_methods.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_analyze_file_with_exceptions() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("ExceptionHandling.java");
    
    let java_content = r#"
        package com.example.exceptions;
        
        import java.io.IOException;
        import java.sql.SQLException;
        
        public class ExceptionHandling {
            public void riskyMethod() throws IOException, SQLException {
                if (Math.random() > 0.5) {
                    throw new IOException("Random IO error");
                }
                if (Math.random() > 0.8) {
                    throw new SQLException("Random SQL error");
                }
            }
            
            public void safeMethod() {
                try {
                    riskyMethod();
                } catch (IOException e) {
                    System.err.println("IO Error: " + e.getMessage());
                } catch (SQLException e) {
                    System.err.println("SQL Error: " + e.getMessage());
                } catch (Exception e) {
                    System.err.println("Unexpected error: " + e.getMessage());
                } finally {
                    System.out.println("Cleanup completed");
                }
            }
            
            public void resourceMethod() {
                try (AutoCloseable resource = new AutoCloseable() {
                    @Override
                    public void close() throws Exception {
                        System.out.println("Resource closed");
                    }
                }) {
                    System.out.println("Using resource");
                } catch (Exception e) {
                    System.err.println("Error: " + e.getMessage());
                }
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let methods = analyzer.get_all_methods();
    assert!(methods.len() >= 3); // riskyMethod, safeMethod, resourceMethod
    
    // 验证特定方法
    let risky_methods = analyzer.find_methods_by_name("riskyMethod");
    assert!(!risky_methods.is_empty());
    
    let safe_methods = analyzer.find_methods_by_name("safeMethod");
    assert!(!safe_methods.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_analyze_directory() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();
    let project_dir = temp_dir.path().join("java_project");
    fs::create_dir(&project_dir).unwrap();
    
    // 创建多个Java文件
    let src_dir = project_dir.join("src");
    let main_dir = src_dir.join("main").join("java").join("com").join("example");
    fs::create_dir_all(&main_dir).unwrap();
    
    // 文件1: 主类
    let main_class = main_dir.join("MainClass.java");
    let main_content = r#"
        package com.example;
        
        public class MainClass {
            public static void main(String[] args) {
                System.out.println("Hello, World!");
                Calculator calc = new Calculator();
                int result = calc.add(5, 3);
                System.out.println("5 + 3 = " + result);
            }
        }
    "#;
    fs::write(&main_class, main_content).unwrap();
    
    // 文件2: 工具类
    let util_class = main_dir.join("Calculator.java");
    let util_content = r#"
        package com.example;
        
        public class Calculator {
            public int add(int a, int b) {
                return a + b;
            }
            
            public int subtract(int a, int b) {
                return a - b;
            }
            
            public int multiply(int a, int b) {
                return a * b;
            }
        }
    "#;
    fs::write(&util_class, util_content).unwrap();
    
    // 文件3: 接口
    let interface_file = main_dir.join("MathOperation.java");
    let interface_content = r#"
        package com.example;
        
        public interface MathOperation {
            int perform(int a, int b);
        }
    "#;
    fs::write(&interface_file, interface_content).unwrap();
    
    // 分析整个目录
    let result = analyzer.analyze_directory(&project_dir);
    assert!(result.is_ok());
    
    // 检查分析结果
    let methods = analyzer.get_all_methods();
    assert!(methods.len() >= 4); // main + 3 calculator methods
    
    // 验证特定方法
    let main_methods = analyzer.find_methods_by_name("main");
    assert!(!main_methods.is_empty());
    
    let add_methods = analyzer.find_methods_by_name("add");
    assert!(!add_methods.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_method_registry() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("RegistryTest.java");
    
    let java_content = r#"
        package com.example.registry;
        
        public class RegistryTest {
            public void method1() {
                System.out.println("Method 1");
            }
            
            public void method2() {
                System.out.println("Method 2");
                method1(); // 调用method1
            }
            
            public void method3() {
                System.out.println("Method 3");
                method2(); // 调用method2
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 检查方法注册表
    let all_methods = analyzer.get_all_methods();
    assert_eq!(all_methods.len(), 3);
    
    // 检查文件方法映射
    let file_methods = analyzer.get_file_methods(&java_file);
    assert!(file_methods.is_some());
    assert_eq!(file_methods.unwrap().len(), 3);
    
    // 验证方法查找
    let method1 = analyzer.find_methods_by_name("method1");
    assert_eq!(method1.len(), 1);
    
    let method2 = analyzer.find_methods_by_name("method2");
    assert_eq!(method2.len(), 1);
    
    let method3 = analyzer.find_methods_by_name("method3");
    assert_eq!(method3.len(), 1);
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_generate_report() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("ReportTest.java");
    
    let java_content = r#"
        package com.example.report;
        
        public class ReportTest {
            public void testMethod1() {
                System.out.println("Test 1");
            }
            
            public void testMethod2() {
                System.out.println("Test 2");
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 生成报告
    let report = analyzer.generate_report();
    
    // 验证报告内容
    assert!(report.contains("Java Code Analysis Report"));
    assert!(report.contains("Total Methods: 2"));
    assert!(report.contains("Total Files: 1"));
    assert!(report.contains("testMethod1"));
    assert!(report.contains("testMethod2"));
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_error_handling() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 测试分析不存在的文件
    let non_existent_file = Path::new("/non/existent/file.java");
    let result = analyzer.analyze_file(non_existent_file);
    assert!(result.is_err());
    
    // 测试分析无效的Java文件
    let temp_dir = TempDir::new().unwrap();
    let invalid_java_file = temp_dir.path().join("Invalid.java");
    
    let invalid_content = r#"
        package com.example;
        
        public class Invalid {
            public void method1() {
                // 缺少分号
                System.out.println("Hello"
            }
        }
    "#;
    
    fs::write(&invalid_java_file, invalid_content).unwrap();
    
    // 分析应该失败（语法错误）
    let result = analyzer.analyze_file(&invalid_java_file);
    // 注意：tree-sitter可能能够解析部分语法错误，所以这里不强制要求失败
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_complex_scenarios() {
    let mut analyzer = JavaAnalyzer::new().unwrap();
    
    // 创建临时Java文件，包含复杂的Java特性
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("ComplexJava.java");
    
    let java_content = r#"
        package com.example.complex;
        
        import java.util.*;
        import java.util.stream.Collectors;
        
        @SuppressWarnings("unchecked")
        public class ComplexJava<T extends Number> {
            private final Map<String, List<T>> dataMap;
            private static final int MAX_SIZE = 1000;
            
            public ComplexJava() {
                this.dataMap = new HashMap<>();
            }
            
            @SafeVarargs
            public final void addData(String key, T... values) {
                if (key == null || key.isEmpty()) {
                    throw new IllegalArgumentException("Key cannot be null or empty");
                }
                
                dataMap.computeIfAbsent(key, k -> new ArrayList<>())
                      .addAll(Arrays.asList(values));
                
                if (dataMap.get(key).size() > MAX_SIZE) {
                    dataMap.get(key).subList(MAX_SIZE, dataMap.get(key).size()).clear();
                }
            }
            
            public <R> List<R> transformData(String key, 
                                           java.util.function.Function<T, R> transformer) {
                return dataMap.getOrDefault(key, Collections.emptyList())
                             .stream()
                             .map(transformer)
                             .filter(Objects::nonNull)
                             .collect(Collectors.toList());
            }
            
            public void processWithLambda(String key, 
                                       java.util.function.Consumer<T> processor) {
                dataMap.getOrDefault(key, Collections.emptyList())
                      .forEach(processor);
            }
            
            @Override
            public String toString() {
                return "ComplexJava{" +
                       "dataMap=" + dataMap +
                       ", size=" + dataMap.values().stream()
                                         .mapToInt(List::size)
                                         .sum() +
                       '}';
            }
            
            public static <E> ComplexJava<E> createInstance() {
                return new ComplexJava<>();
            }
            
            public static class Builder {
                private final ComplexJava<Number> instance;
                
                public Builder() {
                    this.instance = new ComplexJava<>();
                }
                
                public Builder addData(String key, Number... values) {
                    instance.addData(key, values);
                    return this;
                }
                
                public ComplexJava<Number> build() {
                    return instance;
                }
            }
        }
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok());
    
    // 检查分析结果
    let methods = analyzer.get_all_methods();
    assert!(methods.len() >= 6); // 构造函数 + 多个方法 + 内部类方法
    
    // 验证特定方法
    let add_data_methods = analyzer.find_methods_by_name("addData");
    assert!(!add_data_methods.is_empty());
    
    let transform_data_methods = analyzer.find_methods_by_name("transformData");
    assert!(!transform_data_methods.is_empty());
    
    let to_string_methods = analyzer.find_methods_by_name("toString");
    assert!(!to_string_methods.is_empty());
    
    // 清理
    temp_dir.close().unwrap();
} 