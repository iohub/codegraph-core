use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::JavaAnalyzer;

#[test]
fn test_java_analyzer_creation() {
    let analyzer = JavaAnalyzer::new();
    assert!(analyzer.is_ok(), "Failed to create Java analyzer: {:?}", analyzer.err());
}

#[test]
fn test_java_analyzer_analyze_file() {
    let mut analyzer = JavaAnalyzer::new().expect("Failed to create Java analyzer");
    
    // 创建临时Java文件
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("Calculator.java");
    
    let java_content = r#"
package com.example.calculator;

import java.util.List;
import java.util.ArrayList;

/**
 * 简单的计算器类
 */
public class Calculator {
    private double result;
    private List<String> history;
    
    public Calculator() {
        this.result = 0.0;
        this.history = new ArrayList<>();
    }
    
    /**
     * 加法运算
     */
    public double add(double a, double b) {
        this.result = a + b;
        this.history.add(String.format("add(%.2f, %.2f) = %.2f", a, b, this.result));
        return this.result;
    }
    
    /**
     * 减法运算
     */
    public double subtract(double a, double b) {
        this.result = a - b;
        this.history.add(String.format("subtract(%.2f, %.2f) = %.2f", a, b, this.result));
        return this.result;
    }
    
    /**
     * 获取计算历史
     */
    public List<String> getHistory() {
        return new ArrayList<>(this.history);
    }
}
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok(), "Failed to analyze Java file: {:?}", result.err());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_analyze_directory() {
    let mut analyzer = JavaAnalyzer::new().expect("Failed to create Java analyzer");
    
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();
    let java_dir = temp_dir.path().join("java");
    fs::create_dir(&java_dir).unwrap();
    
    // 创建多个Java文件
    let java_file1 = java_dir.join("Main.java");
    let java_file2 = java_dir.join("Utils.java");
    
    let java_content1 = r#"
package com.example;

public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}
    "#;
    
    let java_content2 = r#"
package com.example;

public class Utils {
    public static String format(String input) {
        return input.trim().toLowerCase();
    }
}
    "#;
    
    fs::write(&java_file1, java_content1).unwrap();
    fs::write(&java_file2, java_content2).unwrap();
    
    // 分析目录
    let result = analyzer.analyze_directory(&java_dir);
    assert!(result.is_ok(), "Failed to analyze Java directory: {:?}", result.err());
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_java_analyzer_complex_code() {
    let mut analyzer = JavaAnalyzer::new().expect("Failed to create Java analyzer");
    
    let temp_dir = TempDir::new().unwrap();
    let java_file = temp_dir.path().join("ComplexTest.java");
    
    let java_content = r#"
package com.example.test;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

/**
 * Complex test class with multiple methods and nested classes
 */
public class ComplexTestClass<T extends Comparable<T>> {
    
    private static final String DEFAULT_NAME = "default";
    private final String name;
    private final List<T> items;
    private Map<String, Object> config;
    
    /**
     * Constructor with parameters
     */
    public ComplexTestClass(String name, List<T> items) {
        this.name = name != null ? name : DEFAULT_NAME;
        this.items = items != null ? items : new ArrayList<>();
        this.config = new HashMap<>();
    }
    
    /**
     * Default constructor
     */
    public ComplexTestClass() {
        this(DEFAULT_NAME, null);
    }
    
    /**
     * Get the name of this instance
     */
    public String getName() {
        return this.name;
    }
    
    /**
     * Add item to the list
     */
    public void addItem(T item) {
        if (item != null) {
            this.items.add(item);
        }
    }
    
    /**
     * Process items with stream operations
     */
    public List<T> processItems() {
        return this.items.stream()
            .filter(item -> item != null)
            .sorted()
            .collect(Collectors.toList());
    }
    
    /**
     * Static factory method
     */
    public static <E> ComplexTestClass<E> createDefault() {
        return new ComplexTestClass<>();
    }
}
    "#;
    
    fs::write(&java_file, java_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&java_file);
    assert!(result.is_ok(), "Failed to analyze complex Java file: {:?}", result.err());
    
    // 清理
    temp_dir.close().unwrap();
} 