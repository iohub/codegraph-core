# 测试代码结构

本目录包含了codegraph-core项目的所有测试代码，按照功能模块进行了组织。

## 测试文件组织

### 核心分析器测试
- `test_rust_analyzer.rs` - Rust分析器的单元测试
- `test_javascript_analyzer.rs` - JavaScript分析器的单元测试  
- `test_typescript_analyzer.rs` - TypeScript分析器的单元测试
- `test_python_analyzer.rs` - Python分析器的单元测试
- `test_cpp_analyzer.rs` - C++分析器的单元测试
- `test_java_analyzer.rs` - Java分析器的单元测试

### 集成测试
- `test_integration.rs` - 多个分析器协同工作的集成测试

## 测试覆盖范围

每个分析器的测试都包含以下方面：

1. **基本功能测试**
   - 分析器创建
   - 单个文件分析
   - 目录分析

2. **代码结构识别测试**
   - 函数定义识别
   - 类/结构体识别
   - 导入/导出识别
   - 命名空间识别

3. **边界情况测试**
   - 错误处理
   - 空文件/目录处理
   - 性能测试

## 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定分析器的测试
cargo test test_rust_analyzer

# 运行集成测试
cargo test test_integration

# 运行测试并显示输出
cargo test -- --nocapture
```

## 测试数据

测试使用临时文件和目录，确保测试的隔离性和可重复性。每个测试都会：

1. 创建临时测试环境
2. 生成测试代码文件
3. 执行分析操作
4. 验证分析结果
5. 清理临时资源

## 注意事项

- 所有测试都应该是独立的，不依赖外部状态
- 使用`tempfile`库创建临时文件，确保测试后自动清理
- 测试代码应该覆盖正常情况和边界情况
- 避免在测试中硬编码路径，使用相对路径或临时路径 