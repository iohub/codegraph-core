# Java语言解析器使用指南

## 概述

本指南介绍如何使用CodeGraph Core项目中的Java语言解析器，包括Tree-sitter查询、AST解析和调用图构建功能。

## 功能特性

### 1. 代码结构解析
- **类定义**: 支持普通类、抽象类、内部类
- **接口定义**: 支持接口和注解接口
- **方法定义**: 支持实例方法、静态方法、抽象方法
- **构造函数**: 支持构造函数定义和调用
- **字段和变量**: 支持成员变量、局部变量、参数
- **枚举**: 支持枚举类型定义
- **注解**: 支持注解声明和使用

### 2. 包和导入管理
- **包声明**: 解析package语句
- **导入语句**: 解析import语句，包括静态导入
- **命名空间**: 支持Java包命名空间

### 3. 调用图分析
- **方法调用**: 识别方法调用关系
- **构造函数调用**: 识别对象创建和数组创建
- **字段访问**: 识别字段读写操作
- **依赖关系**: 构建类和方法之间的依赖图

## 使用方法

### 1. 基本解析

```rust
use std::path::PathBuf;
use codegraph_core::codegraph::analyzers::JavaParser;

fn main() {
    let java_code = r#"
    package com.example;
    
    public class HelloWorld {
        public static void main(String[] args) {
            System.out.println("Hello, World!");
        }
    }
    "#;
    
    let path = PathBuf::from("HelloWorld.java");
    
    match JavaParser::new() {
        Ok(mut parser) => {
            let symbols = parser.parse(java_code, &path);
            println!("Parsed {} symbols", symbols.len());
        }
        Err(e) => {
            eprintln!("Failed to create Java parser: {}", e.message);
        }
    }
}
```

### 2. 使用查询功能

```rust
use codegraph_core::codegraph::treesitter::queries::java::JavaQueries;
use tree_sitter_java::language;

fn main() {
    let language = language();
    match JavaQueries::new(language) {
        Ok(queries) => {
            // 使用各种查询
            // queries.method_definition - 方法定义查询
            // queries.class_definition - 类定义查询
            // queries.method_call - 方法调用查询
            // 等等...
        }
        Err(e) => {
            eprintln!("Failed to create Java queries: {}", e);
        }
    }
}
```

### 3. 调用图分析

```rust
use codegraph_core::codegraph::treesitter::analyzers::java_callgraph::JavaCallGraphAnalyzer;
use codegraph_core::codegraph::treesitter::queries::java::JavaQueries;

fn main() {
    let language = tree_sitter_java::language();
    let queries = JavaQueries::new(language).unwrap();
    let analyzer = JavaCallGraphAnalyzer::new(queries);
    
    let java_code = "..."; // 你的Java代码
    let result = analyzer.analyze(java_code, "Example.java");
    
    println!("Found {} snippets", result.snippets.len());
    println!("Found {} method calls", result.method_calls.len());
    println!("Found {} classes", result.classes.len());
    println!("Found {} interfaces", result.interfaces.len());
}
```

## 查询模式说明

### 方法定义查询

```scheme
(method_declaration
  name: (identifier) @method.name
  parameters: (formal_parameters) @method.params
  body: (block) @method.body
) @method.def
```

### 类定义查询

```scheme
(class_declaration
  name: (identifier) @class.name
  body: (class_body) @class.body
) @class.def
```

### 方法调用查询

```scheme
(method_invocation
  name: (identifier) @method.called
  arguments: (argument_list) @method.args
) @method.call
```

## 数据结构

### JavaSnippet

```rust
pub struct JavaSnippet {
    pub snippet_type: JavaSnippetType,
    pub name: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub file_path: String,
    pub package_name: Option<String>,
    pub class_name: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub modifiers: Vec<String>,
}
```

### JavaMethodCall

```rust
pub struct JavaMethodCall {
    pub caller_name: String,
    pub called_name: String,
    pub caller_location: (usize, usize),
    pub called_location: (usize, usize),
    pub caller_file: String,
    pub called_file: Option<String>,
    pub is_resolved: bool,
    pub package_name: Option<String>,
    pub class_name: Option<String>,
    pub method_signature: Option<String>,
}
```

## 示例代码

项目包含一个完整的Java计算器示例，展示了各种Java语言特性：

- `examples/java_analysis_example.rs` - 生成示例Java代码
- `example_calculator.java` - 生成的Java示例文件

## 注意事项

1. **依赖要求**: 确保项目中包含`tree-sitter-java`依赖
2. **错误处理**: 解析器可能因为语法错误而失败，请妥善处理错误
3. **性能考虑**: 对于大型Java项目，解析可能需要较长时间
4. **内存使用**: AST解析会占用一定内存，特别是对于大型文件

## 扩展功能

### 自定义查询

你可以创建自定义的Tree-sitter查询来满足特定需求：

```rust
let custom_query = Query::new(
    language,
    r#"
    (method_declaration
      name: (identifier) @custom.method
      modifiers: (modifiers) @custom.modifiers
    ) @custom.def
    "#,
)?;
```

### 调用图可视化

结合其他工具，可以将调用图导出为图形格式（如DOT、SVG等）进行可视化分析。

## 故障排除

### 常见问题

1. **解析失败**: 检查Java代码语法是否正确
2. **查询不匹配**: 确认Tree-sitter查询语法正确
3. **内存不足**: 对于大型文件，考虑分段解析

### 调试技巧

1. 使用`tracing`日志查看解析过程
2. 检查AST节点类型和结构
3. 验证查询捕获组是否正确

## 贡献

欢迎提交Issue和Pull Request来改进Java解析器功能。请确保：

1. 添加适当的测试用例
2. 更新相关文档
3. 遵循项目的代码风格指南 