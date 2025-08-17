# Rust Analyzer

Rust分析器是基于Tree-sitter的Rust代码分析工具，能够解析Rust源代码并提取函数、结构体、枚举、trait等代码结构信息。

## ✅ 完成状态

Rust分析器已经成功实现并集成到codegraph-core项目中：

- ✅ **Tree-sitter查询系统** - 完整的Rust语法查询集合
- ✅ **RustAnalyzer实现** - 基于现有分析器架构的Rust专用分析器
- ✅ **模块集成** - 已集成到analyzers模块中
- ✅ **测试验证** - 通过所有单元测试
- ✅ **演示程序** - 提供完整的功能演示

## 功能特性

### 支持的Rust语法结构

1. **函数定义和调用**
   - 普通函数定义 (`fn`)
   - 函数签名（无实现）
   - 函数调用
   - 方法调用
   - 泛型函数调用

2. **结构体和枚举**
   - 结构体定义 (`struct`)
   - 元组结构体
   - 单元结构体
   - 枚举定义 (`enum`)

3. **Trait和Impl**
   - Trait定义 (`trait`)
   - Impl块 (`impl`)
   - Trait实现

4. **模块系统**
   - 模块定义 (`mod`)
   - 模块声明
   - Use声明 (`use`)

5. **类型系统**
   - 类型别名 (`type`)
   - 泛型参数
   - 生命周期
   - 类型参数

6. **宏系统**
   - 宏定义 (`macro_rules!`)
   - 宏调用

7. **其他**
   - 常量定义 (`const`)
   - 静态变量 (`static`)
   - 变量声明 (`let`)
   - 属性 (`#[...]`)

## 使用方法

### 基本用法

```rust
use codegraph_cli::codegraph::analyzers::{RustAnalyzer, CodeAnalyzer};
use std::path::PathBuf;

// 创建Rust分析器
let mut analyzer = RustAnalyzer::new()?;

// 分析单个文件
let file_path = PathBuf::from("src/main.rs");
analyzer.analyze_file(&file_path)?;

// 分析整个目录
let dir_path = PathBuf::from("src/");
analyzer.analyze_directory(&dir_path)?;
```

### 运行演示

```bash
# 运行Rust分析器演示
cargo run --example rust_analyzer_demo
```

### 运行测试

```bash
# 运行所有Rust分析器测试
cargo test test_rust_analyzer

# 运行特定测试
cargo test test_rust_analyzer_creation
cargo test test_rust_analyzer_analyze_file
cargo test test_rust_analyzer_analyze_directory
```

## 实现细节

### 文件结构

```
src/codegraph/
├── analyzers/
│   ├── rust_analyzer.rs      # Rust分析器主实现
│   ├── rust_parser.rs        # Rust解析器（兼容层）
│   └── mod.rs               # 分析器模块导出
└── treesitter/
    └── queries/
        ├── rust.rs          # Rust Tree-sitter查询
        └── mod.rs           # 查询模块导出
```

### 核心组件

1. **RustQueries** - Tree-sitter查询集合
   - 函数定义和调用查询
   - 结构体、枚举、trait查询
   - 模块、宏、类型查询

2. **RustAnalyzer** - 主要分析器
   - 基于Tree-sitter的语法解析
   - 代码结构提取
   - 作用域分析

3. **RustSnippet** - 代码片段表示
   - 支持多种Rust语法结构
   - 包含位置、作用域信息

## 技术特点

- **基于Tree-sitter** - 使用官方Rust语法树解析器
- **高性能** - 增量解析和缓存机制
- **可扩展** - 模块化设计，易于添加新功能
- **类型安全** - 完整的Rust类型系统支持
- **错误处理** - 健壮的错误处理和恢复机制

## 示例输出

分析器能够提取以下信息：

```rust
// 函数定义
Function: new() -> Calculator
Function: add(&mut self, x: i32) -> i32
Function: get_value(&self) -> i32

// 结构体定义
Struct: Calculator { value: i32 }

// Trait定义
Trait: MathOperations
  - multiply(&self, x: i32) -> i32
  - divide(&self, x: i32) -> Result<i32, String>

// Impl块
Impl: MathOperations for Calculator
```

## 集成状态

Rust分析器已完全集成到codegraph-core项目中：

- ✅ 在`get_code_analyzer`函数中注册
- ✅ 实现`CodeAnalyzer` trait
- ✅ 通过所有编译和测试
- ✅ 支持CLI工具链

## 未来改进

- [ ] 增强泛型支持
- [ ] 改进生命周期分析
- [ ] 添加更多Rust特定语法支持
- [ ] 优化性能
- [ ] 增加更多测试用例

## 贡献

欢迎贡献代码和改进建议！请确保：

1. 运行所有测试：`cargo test`
2. 检查代码风格：`cargo fmt`
3. 运行linter：`cargo clippy`

## 许可证

本项目采用MIT许可证。 