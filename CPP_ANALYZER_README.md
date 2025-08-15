# C++ 代码分析器

这个C++代码分析器基于Tree-sitter实现，能够分析C++源代码并提取函数定义、函数调用关系、类定义、命名空间等信息。

## 功能特性

### 1. 代码片段索引
- **函数定义索引**: 提取所有函数定义，包括普通函数、成员函数、模板函数
- **类定义索引**: 提取类、结构体、模板类定义
- **命名空间索引**: 提取命名空间定义和声明
- **变量声明索引**: 提取变量声明和成员变量

### 2. 函数片段索引
- **作用域追踪**: 精确记录每个函数体的起止位置
- **参数解析**: 提取函数参数信息
- **返回类型识别**: 识别函数返回类型
- **命名空间归属**: 确定函数所属的命名空间
- **类归属**: 确定成员函数所属的类

### 3. 调用关系建立
- **函数调用识别**: 识别所有函数调用表达式
- **作用域归属**: 根据调用位置确定调用者函数
- **全局调用处理**: 单独收集不在函数内部的调用
- **调用链追踪**: 建立完整的函数调用关系图

## 核心设计

### 三重解析架构

1. **第一遍**: 收集函数定义和作用域信息
   ```rust
   (function_definition
     declarator: (function_declarator
       declarator: (_) @function.name
       parameters: (parameter_list) @function.params
     )
     body: (compound_statement) @function.body
   ) @function.def
   ```

2. **第二遍**: 收集所有调用表达式（不区分位置）

3. **第三遍**: 基于位置判断调用的归属

### 函数体位置记录

```rust
struct FunctionScope {
    body_start: Point,  // 函数体起始位置
    body_end: Point,    // 函数体结束位置
}
```

使用Tree-sitter的Point类型记录每个函数体的行列坐标范围。

### 精确归属判断

```rust
// 检查调用是否在函数体内
if call.location.row >= scope.body_start.row &&
   call.location.row <= scope.body_end.row &&
   call.location.column >= scope.body_start.column &&
   call.location.column <= scope.body_end.column {
    // 归属此函数
}
```

## 使用方法

### 基本使用

```rust
use codegraph_cli::codegraph::CppAnalyzer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建C++分析器
    let mut analyzer = CppAnalyzer::new()?;
    
    // 分析目录
    let dir = Path::new("./src");
    analyzer.analyze_directory(dir)?;
    
    // 获取所有函数
    let functions = analyzer.get_all_functions();
    println!("Found {} functions", functions.len());
    
    // 查找特定函数
    let main_functions = analyzer.find_functions_by_name("main");
    
    // 生成报告
    let report = analyzer.generate_report();
    println!("{}", report);
    
    Ok(())
}
```

### 分析单个文件

```rust
let mut analyzer = CppAnalyzer::new()?;
let file_path = Path::new("example.cpp");
analyzer.analyze_file(file_path)?;

// 获取文件的函数列表
if let Some(functions) = analyzer.get_file_functions(file_path) {
    for function in functions {
        println!("Function: {} (lines {}-{})", 
            function.name, function.line_start, function.line_end);
    }
}
```

## 支持的C++特性

### 函数定义
- 普通函数定义
- 成员函数定义
- 模板函数定义
- 构造函数和析构函数
- 内联函数

### 类定义
- 类声明
- 结构体声明
- 模板类声明
- 继承关系

### 命名空间
- 命名空间定义
- 命名空间声明
- 嵌套命名空间

### 函数调用
- 普通函数调用
- 成员函数调用
- 模板函数调用
- 构造函数调用
- 析构函数调用

### 包含文件
- 系统头文件 (#include <...>)
- 用户头文件 (#include "...")

## 输出格式

### 函数信息
```rust
pub struct FunctionInfo {
    pub id: Uuid,
    pub name: String,
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub namespace: String,
    pub language: String,
    pub signature: Option<String>,
    pub return_type: Option<String>,
    pub parameters: Vec<ParameterInfo>,
}
```

### 调用关系
```rust
pub struct CppFunctionCall {
    pub caller_name: String,
    pub called_name: String,
    pub caller_location: (usize, usize), // (line, column)
    pub called_location: (usize, usize),
    pub caller_file: String,
    pub called_file: Option<String>,
    pub is_resolved: bool,
    pub namespace: Option<String>,
    pub class_name: Option<String>,
}
```

## 性能优化

### 空间换时间
- 使用区间树存储函数作用域位置，加速归属判断
- 缓存解析结果，避免重复解析

### 并行处理
- 对大型代码库可将不同查询放在不同线程执行
- 支持增量解析，对编辑场景使用parser.parse_edits()进行部分更新

## 测试用例

### 测试文件示例
```cpp
#include <iostream>
#include "utils.h"

void helper() {
    std::cout << "Helper called" << std::endl;
}

int main() {
    helper();
    return 0;
}

// 函数外部的调用
initializeSystem();  // 应该被识别为全局调用
```

### 预期输出
```
Include Paths:
- iostream
- utils.h

Function Definitions and Calls:
Function: helper
  Params: 
  Calls: std::cout, std::endl
----------------------------------
Function: main
  Params: 
  Calls: helper
----------------------------------

Global Scope Calls:
- initializeSystem
```

## 扩展性

### 自定义查询
可以通过修改`CppQueries`结构体来添加新的Tree-sitter查询：

```rust
pub struct CppQueries {
    pub function_definition: Query,
    pub function_call: Query,
    pub class_definition: Query,
    // 添加新的查询...
    pub custom_query: Query,
}
```

### 新的代码片段类型
可以扩展`CppSnippetType`枚举来支持新的代码片段类型：

```rust
pub enum CppSnippetType {
    Function,
    Class,
    Namespace,
    Variable,
    Include,
    Method,
    Constructor,
    Destructor,
    // 添加新的类型...
    Macro,
    Enum,
}
```

## 错误处理

分析器提供了完善的错误处理机制：

- 文件读取错误
- 语法解析错误
- Tree-sitter查询错误
- 内存分配错误

所有错误都会通过`Result<T, String>`类型返回，便于调试和处理。

## 依赖项

- `tree-sitter`: C++语法解析
- `tree-sitter-cpp`: C++语言支持
- `tracing`: 日志记录
- `uuid`: 唯一标识符生成
- `serde`: 序列化支持

## 许可证

本项目采用MIT许可证，详见LICENSE文件。 