use std::path::Path;
use tempfile::TempDir;
use std::fs;
use codegraph_cli::codegraph::JavaScriptAnalyzer;

#[test]
fn test_javascript_analyzer_new() {
    let analyzer = JavaScriptAnalyzer::new();
    assert!(analyzer.is_ok());
}

#[test]
fn test_javascript_analyzer_analyze_file() {
    let mut analyzer = JavaScriptAnalyzer::new().unwrap();
    
    // 创建临时JavaScript文件
    let temp_dir = TempDir::new().unwrap();
    let js_file = temp_dir.path().join("test.js");
    
    let js_content = r#"
        // 函数声明
        function greet(name) {
            return `Hello, ${name}!`;
        }
        
        // 箭头函数
        const multiply = (a, b) => a * b;
        
        // 类定义
        class UserService {
            constructor(apiUrl) {
                this.apiUrl = apiUrl;
            }
            
            async getUsers() {
                const response = await fetch(this.apiUrl);
                return response.json();
            }
        }
        
        // 对象字面量
        const utils = {
            formatDate(date) {
                return date.toISOString();
            }
        };
    "#;
    
    fs::write(&js_file, js_content).unwrap();
    
    // 分析文件
    let result = analyzer.analyze_file(&js_file);
    assert!(result.is_ok());
    
    // 添加调试信息
    let result = analyzer.get_analysis_result();
    println!("Debug: Total snippets found: {}", result.snippets.len());
    println!("Debug: Snippets: {:?}", result.snippets.iter().map(|s| (s.name.clone(), format!("{:?}", s.snippet_type))).collect::<Vec<_>>());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    println!("Debug: Functions found: {}", functions.len());
    assert!(functions.len() >= 3); // 至少应该有3个函数
    
    let classes = analyzer.get_all_classes();
    assert!(classes.len() >= 1); // 至少应该有1个类
    
    let objects = analyzer.get_all_objects();
    assert!(objects.len() >= 1); // 至少应该有1个对象
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_javascript_analyzer_analyze_directory() {
    let mut analyzer = JavaScriptAnalyzer::new().unwrap();
    
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();
    let js_dir = temp_dir.path().join("js");
    fs::create_dir(&js_dir).unwrap();
    
    // 创建多个JavaScript文件
    let js_file1 = js_dir.join("file1.js");
    let js_file2 = js_dir.join("file2.js");
    
    let js_content1 = r#"
        function helper() {
            return "help";
        }
    "#;
    
    let js_content2 = r#"
        class Helper {
            static getValue() {
                return "value";
            }
        }
    "#;
    
    fs::write(&js_file1, js_content1).unwrap();
    fs::write(&js_file2, js_content2).unwrap();
    
    // 分析目录
    let result = analyzer.analyze_directory(temp_dir.path());
    assert!(result.is_ok());
    
    // 检查分析结果
    let functions = analyzer.get_all_functions();
    assert!(functions.len() >= 2); // 至少应该有2个函数
    
    let classes = analyzer.get_all_classes();
    assert!(classes.len() >= 1); // 至少应该有1个类
    
    // 清理
    temp_dir.close().unwrap();
}

#[test]
fn test_javascript_analyzer_function_types() {
    let mut analyzer = JavaScriptAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let js_file = temp_dir.path().join("functions.js");
    
    let js_content = r#"
        // 函数声明
        function declaredFunction() {}
        
        // 函数表达式
        const expressionFunction = function() {};
        
        // 箭头函数
        const arrowFunction = () => {};
        
        // 对象方法
        const obj = {
            method() {},
            arrowMethod: () => {}
        };
        
        // 类方法
        class TestClass {
            constructor() {}
            method() {}
            static staticMethod() {}
        }
    "#;
    
    fs::write(&js_file, js_content).unwrap();
    
    analyzer.analyze_file(&js_file).unwrap();
    
    let functions = analyzer.get_all_functions();
    assert!(functions.len() >= 7); // 应该有多个函数
    
    // 检查不同类型的函数
    let has_declared = functions.iter().any(|f| f.name == "declaredFunction");
    let has_expression = functions.iter().any(|f| f.name == "expressionFunction");
    let has_arrow = functions.iter().any(|f| f.name == "arrowFunction");
    
    assert!(has_declared);
    assert!(has_expression);
    assert!(has_arrow);
    
    temp_dir.close().unwrap();
}

#[test]
fn test_javascript_analyzer_imports_exports() {
    let mut analyzer = JavaScriptAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let js_file = temp_dir.path().join("modules.js");
    
    let js_content = r#"
        // 导入语句
        import { Component } from 'react';
        import * as React from 'react';
        import { useState, useEffect } from 'react';
        
        // 导出语句
        export function exportedFunction() {}
        export class ExportedClass {}
        export default class DefaultClass {}
        
        // 命名导出
        export { exportedFunction, ExportedClass };
    "#;
    
    fs::write(&js_file, js_content).unwrap();
    
    analyzer.analyze_file(&js_file).unwrap();
    
    let result = analyzer.get_analysis_result();
    assert!(result.imports.len() >= 3); // 至少应该有3个导入
    assert!(result.exports.len() >= 3); // 至少应该有3个导出
    
    temp_dir.close().unwrap();
}

#[test]
fn test_javascript_analyzer_generate_report() {
    let mut analyzer = JavaScriptAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let js_file = temp_dir.path().join("report.js");
    
    let js_content = r#"
        function testFunction() {
            return "test";
        }
        
        class TestClass {
            constructor() {}
        }
    "#;
    
    fs::write(&js_file, js_content).unwrap();
    
    analyzer.analyze_file(&js_file).unwrap();
    
    let report = analyzer.generate_report();
    
    // 检查报告内容
    assert!(report.contains("JavaScript Code Analysis Report"));
    assert!(report.contains("Total Snippets:"));
    assert!(report.contains("=== Functions ==="));
    assert!(report.contains("=== Classes ==="));
    
    temp_dir.close().unwrap();
}

#[test]
fn test_javascript_analyzer_skip_ignored_directories() {
    let mut analyzer = JavaScriptAnalyzer::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    
    // 创建node_modules目录
    let node_modules = temp_dir.path().join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    
    // 在node_modules中创建JavaScript文件
    let js_file = node_modules.join("ignored.js");
    let js_content = "function ignored() {}";
    fs::write(&js_file, js_content).unwrap();
    
    // 在根目录创建JavaScript文件
    let root_js = temp_dir.path().join("root.js");
    let root_content = "function root() {}";
    fs::write(&root_js, root_content).unwrap();
    
    // 分析目录
    analyzer.analyze_directory(temp_dir.path()).unwrap();
    
    // 检查结果 - node_modules中的文件应该被忽略
    let functions = analyzer.get_all_functions();
    
    // 应该只找到root.js中的函数，忽略node_modules中的
    let root_function = functions.iter().find(|f| f.name == "root");
    let ignored_function = functions.iter().find(|f| f.name == "ignored");
    
    assert!(root_function.is_some());
    assert!(ignored_function.is_none());
    
    temp_dir.close().unwrap();
} 