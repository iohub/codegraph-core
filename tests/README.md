# 函数调用关系图功能测试

本目录包含了针对函数调用关系图系统的全面功能测试，主要测试 `build_graph` 和 `query_call_graph` 的顶层功能。

## 测试结构

### 测试仓库

我们创建了三个不同语言的测试项目来验证系统的多语言支持：

#### 1. Rust测试项目 (`test_repos/simple_rust_project/`)
- **main.rs**: 包含主函数和多个函数调用链
- **lib.rs**: 库函数和模块结构
- **utils.rs**: 工具函数模块
- **math.rs**: 数学运算模块

**函数调用关系示例**:
```
main() → process_data() → fetch_data() → get_raw_data()
                    → transform_data() → double_value()
                                    → add_offset()
                    → validate_data() → default_value()
```

#### 2. Python测试项目 (`test_repos/simple_python_project/`)
- **main.py**: 主程序入口
- **data_processor.py**: 数据处理类，包含复杂的调用链
- **math_utils.py**: 数学工具类
- **string_utils.py**: 字符串处理类

**函数调用关系示例**:
```
main() → DataProcessor.process_data() → _clean_input()
                                → _validate_data()
                                → _transform_data() → _step1_transform()
                                                → _step2_transform()
                                → _finalize_result()
```

#### 3. JavaScript测试项目 (`test_repos/simple_js_project/`)
- **index.js**: 主程序入口
- **dataProcessor.js**: 数据处理类
- **mathUtils.js**: 数学工具类
- **stringUtils.js**: 字符串工具类

## 测试用例

### 1. 构建图功能测试 (`test_build_graph_functionality`)

测试 `build_graph` 的核心功能：

- **多语言项目分析**: 验证系统能正确分析Rust、Python、JavaScript项目
- **函数识别**: 确保能正确识别各种函数定义
- **调用关系解析**: 验证函数调用关系的正确解析
- **统计信息**: 检查文件数量、函数数量等统计信息的准确性
- **图转换**: 测试从CodeGraph到PetCodeGraph的转换

### 2. 查询调用图功能测试 (`test_query_call_graph_functionality`)

测试 `query_call_graph` 的查询功能：

- **函数名查询**: 根据函数名查找函数及其调用关系
- **文件查询**: 查询特定文件中的所有函数
- **调用链扩展**: 测试多层级调用链的扩展功能
- **调用者/被调用者**: 验证调用关系的双向查询

### 3. 完整工作流测试 (`test_complete_build_and_query_workflow`)

测试完整的构建和查询流程：

- **端到端流程**: 从项目分析到图构建到查询的完整流程
- **持久化**: 测试图的保存和加载功能
- **数据一致性**: 验证保存和加载后数据的一致性
- **复杂查询**: 测试复杂的函数调用关系查询

## 运行测试

### 运行所有功能测试

```bash
# 使用测试脚本
./tests/run_functional_tests.sh

# 或手动运行
cargo test --test test_functional -- --nocapture
```

### 运行特定测试

```bash
# 只运行构建图测试
cargo test test_build_graph_functionality --test test_functional -- --nocapture

# 只运行查询测试
cargo test test_query_call_graph_functionality --test test_functional -- --nocapture

# 只运行完整工作流测试
cargo test test_complete_build_and_query_workflow --test test_functional -- --nocapture
```

## 测试验证点

### 构建图验证

- ✅ 能正确解析多语言项目
- ✅ 能识别函数定义和调用关系
- ✅ 能生成准确的统计信息
- ✅ 能正确转换图格式

### 查询功能验证

- ✅ 能按函数名查询函数
- ✅ 能按文件查询函数
- ✅ 能扩展调用链到指定深度
- ✅ 能正确显示调用者和被调用者关系

### 数据完整性验证

- ✅ 保存和加载的图数据一致
- ✅ 调用关系信息完整准确
- ✅ 统计信息正确更新

## 测试数据

测试项目包含以下典型的函数调用模式：

1. **线性调用链**: `main() → process() → helper()`
2. **分支调用**: 一个函数调用多个不同的函数
3. **递归调用**: 如阶乘、斐波那契数列函数
4. **类方法调用**: 面向对象编程中的方法调用
5. **模块间调用**: 跨文件的函数调用

这些测试用例覆盖了真实项目中常见的函数调用模式，确保系统能够正确处理各种复杂的代码结构。

## 注意事项

- 测试使用临时目录，不会影响实际项目
- 测试会输出详细的调试信息，便于问题诊断
- 如果测试失败，请检查测试仓库是否存在且完整
- 确保系统依赖（如tree-sitter解析器）已正确安装 