# Python代码分析器

本项目为Python语言添加了完整的代码分析功能，包括语法解析、函数提取、调用关系分析等。

## 功能特性

### 1. 代码结构分析
- **函数定义识别**：自动识别Python函数定义，包括参数和函数体
- **类定义识别**：识别类定义及其结构
- **导入语句分析**：分析import和from import语句
- **变量赋值跟踪**：识别变量赋值操作
- **装饰器支持**：支持Python装饰器语法

### 2. 代码片段提取
- 函数代码片段
- 类代码片段
- 方法代码片段
- 导入语句
- 变量声明

### 3. 调用关系分析
- 函数调用关系
- 方法调用关系
- 作用域分析
- 命名空间解析

## 文件结构

```
src/codegraph/
├── treesitter/
│   └── queries/
│       ├── cpp.rs          # C++查询定义
│       └── python.rs       # Python查询定义 (新增)
├── cpp_analyzer.rs         # C++分析器
└── python_analyzer.rs      # Python分析器 (新增)
```

## 核心组件

### PythonQueries
定义了Python语言的Tree-sitter查询模式：

```rust
pub struct PythonQueries {
    pub function_definition: Query,    // 函数定义查询
    pub function_call: Query,          // 函数调用查询
    pub class_definition: Query,       // 类定义查询
    pub import_statement: Query,       // 导入语句查询
    pub variable_assignment: Query,    // 变量赋值查询
    pub decorator: Query,              // 装饰器查询
}
```

### PythonAnalyzer
主要的Python代码分析器：

```rust
pub struct PythonAnalyzer {
    parser: Parser,
    language: Language,
    queries: PythonQueries,
    function_registry: HashMap<String, FunctionInfo>,
    file_functions: HashMap<PathBuf, Vec<FunctionInfo>>,
}
```

## 使用方法

### 基本用法

```rust
use codegraph_cli::codegraph::PythonAnalyzer;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建Python分析器
    let mut analyzer = PythonAnalyzer::new()?;
    
    // 分析目录
    analyzer.analyze_directory(Path::new("."))?;
    
    // 获取分析结果
    let functions = analyzer.get_all_functions();
    let report = analyzer.generate_report();
    
    println!("{}", report);
    Ok(())
}
```

### 分析单个文件

```rust
// 分析单个Python文件
analyzer.analyze_file(Path::new("example.py"))?;
```

### 查找特定函数

```rust
// 根据名称查找函数
let main_functions = analyzer.find_functions_by_name("main");
```

## 支持的Python语法

### 函数定义
```python
def simple_function():
    pass

def function_with_params(name: str, age: int = 18) -> str:
    return f"Name: {name}, Age: {age}"
```

### 类定义
```python
class TestClass:
    def __init__(self, name: str):
        self.name = name
    
    @classmethod
    def create_default(cls) -> 'TestClass':
        return cls("default")
```

### 导入语句
```python
import os
from typing import List, Dict
from dataclasses import dataclass
```

### 装饰器
```python
@dataclass
class TestData:
    name: str
    value: int
```

## 分析结果

分析器会生成以下信息：

1. **代码片段**：提取的函数、类、方法等代码片段
2. **函数调用关系**：函数之间的调用关系图
3. **作用域信息**：代码的作用域层次结构
4. **导入依赖**：模块间的导入依赖关系

## 示例输出

```
=== Python Code Analysis Report ===

Total Functions: 10
Total Files: 1

Functions by File:
  ./test_python.py: 11 functions

Function List:
  - __init__ (./test_python.py:67-68)
  - main (./test_python.py:74-111)
  - function_with_decorators (./test_python.py:29-35)
  - create_default (./test_python.py:55-56)
  - validate_name (./test_python.py:60-61)
  - simple_function (./test_python.py:18-20)
  - add_item (./test_python.py:45-47)
  - function_with_params (./test_python.py:23-26)
  - get_items (./test_python.py:50-51)
  - get_extra (./test_python.py:71-71)
```

## 依赖要求

- `tree-sitter-python`: Python语言的Tree-sitter语法支持
- `tree-sitter`: 核心的Tree-sitter库
- `tracing`: 日志记录
- `uuid`: 唯一标识符生成

## 扩展性

该分析器设计为可扩展的架构，可以轻松添加：

- 新的查询模式
- 更多的代码结构分析
- 自定义的分析规则
- 其他编程语言支持

## 与C++分析器的对比

| 特性 | C++分析器 | Python分析器 |
|------|-----------|--------------|
| 函数定义 | ✅ | ✅ |
| 类定义 | ✅ | ✅ |
| 命名空间 | ✅ | 模块系统 |
| 模板支持 | ✅ | 泛型支持 |
| 装饰器 | ❌ | ✅ |
| 动态类型 | ❌ | ✅ |

## 未来改进

1. **类型注解支持**：增强对Python类型注解的解析
2. **异步函数**：支持async/await语法
3. **列表推导式**：分析列表、字典、集合推导式
4. **异常处理**：分析try-except语句结构
5. **循环结构**：分析for、while循环
6. **条件语句**：分析if-elif-else结构

## 贡献

欢迎提交Issue和Pull Request来改进Python分析器的功能！ 