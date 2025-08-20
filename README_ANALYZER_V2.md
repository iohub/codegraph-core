# CodeGraph Analyzer V2 实现

## 概述

根据 `planning.md` 的计划，我们成功实现了基于 `analyzers` 的统一解析与 CodeGraph 构建系统。这个新系统提供了一个统一的接口来解析多种编程语言，并生成标准化的代码图。

## 主要功能

### 1. 统一语言分析器接口

- **LanguageAnalyzer trait**: 为所有支持的语言提供统一的解析接口
- **支持的语言**: Rust、Java、Python、C++、TypeScript、JavaScript
- **核心方法**:
  - `parse_file()`: 解析单个文件
  - `extract_functions()`: 提取函数信息
  - `extract_calls()`: 提取调用关系
  - `extract_snippets()`: 提取代码片段

### 2. 分析器注册表

- **AnalyzerRegistry**: 管理所有语言分析器的注册和查找
- **自动注册**: 支持的语言分析器在创建时自动注册
- **线程安全**: 使用 Arc 实现线程安全的共享

### 3. 编排器

- **AnalyzerOrchestrator**: 协调整个分析流程
- **并发处理**: 支持多线程并发解析文件
- **文件发现**: 自动扫描和识别支持的文件类型
- **结果聚合**: 将多个文件的分析结果聚合为统一的 CodeGraph

### 4. CLI 命令

新增了 `analyze-v2` 命令，提供以下功能：

```bash
cargo run -- analyze-v2 --help
```

**主要选项**:
- `--root`: 指定要分析的根目录 (默认: ".")
- `--languages`: 指定要分析的语言 (逗号分隔: rust,java,python,cpp,typescript,javascript)
- `--output-dir`: 指定输出目录 (默认: "target/codegraph")
- `--formats`: 指定输出格式 (逗号分隔: json,mermaid,dot)
- `--workers`: 指定最大工作线程数
- `--include-tests`: 包含测试文件
- `--follow-symlinks`: 跟随符号链接

## 输出文件

分析完成后，会在指定的输出目录生成以下文件：

1. **graph.json**: 序列化的 CodeGraph 结构
2. **graph.mmd**: Mermaid 格式的图表
3. **graph.dot**: DOT 格式的图表
4. **snippets.json**: 代码片段索引

## 使用示例

### 基本用法

```bash
# 分析当前目录的所有支持语言
cargo run -- analyze-v2

# 分析指定目录的 Rust 文件
cargo run -- analyze-v2 --root examples --languages rust --output-dir test_output

# 分析多种语言
cargo run -- analyze-v2 --languages rust,java,python --include-tests
```

### 输出示例

生成的 `graph.json` 包含：
- 函数信息映射
- 调用关系列表
- 统计信息
- 语言分布

生成的 `graph.mmd` 可以用于：
- Mermaid 在线编辑器
- GitHub/GitLab 的 Mermaid 渲染
- 文档生成

生成的 `graph.dot` 可以用于：
- Graphviz 可视化
- 生成 PNG/SVG 图片

## 架构设计

### 核心组件

1. **LanguageAnalyzer trait**: 统一接口
2. **AnalyzerRegistry**: 分析器管理
3. **AnalyzerOrchestrator**: 流程编排
4. **ParsedUnit**: 解析单元
5. **Snippet**: 代码片段

### 数据流

1. **发现阶段**: 扫描目录，识别文件类型
2. **解析阶段**: 并发解析文件
3. **聚合阶段**: 收集结果，构建 CodeGraph
4. **导出阶段**: 生成多种格式的输出

### 并发处理

- 文件级并发解析
- 可配置的工作线程数
- 线程安全的结果聚合

## 扩展性

### 添加新语言

1. 实现 `LanguageAnalyzer` trait
2. 在 `AnalyzerRegistry::new()` 中注册
3. 更新语言识别逻辑

### 自定义输出格式

1. 在 `CodeGraph` 中添加新的导出方法
2. 在 `AnalyzerOrchestrator::export_results()` 中调用

## 当前状态

### 已完成

- ✅ 统一接口设计
- ✅ 分析器注册表
- ✅ 编排器实现
- ✅ CLI 命令
- ✅ 多格式输出
- ✅ 并发处理
- ✅ 错误处理

### 待完善

- 🔄 真正的函数提取实现
- 🔄 调用关系解析
- 🔄 代码片段生成
- 🔄 跨文件解析
- 🔄 性能优化

## 下一步计划

1. **M1 里程碑**: 实现基本的函数提取和调用关系解析
2. **M2 里程碑**: 完善所有语言的解析器
3. **M3 里程碑**: 优化解析精度和性能

## 技术栈

- **Rust**: 主要开发语言
- **Tree-sitter**: AST 解析
- **anyhow**: 错误处理
- **serde**: 序列化
- **clap**: CLI 参数解析
- **tracing**: 日志记录
- **walkdir**: 文件遍历

## 贡献指南

1. 遵循现有的代码风格
2. 添加适当的测试
3. 更新文档
4. 确保所有测试通过

## 许可证

本项目遵循项目的整体许可证。 