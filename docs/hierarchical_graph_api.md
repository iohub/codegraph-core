# 层次化函数调用关系图API

## 概述

层次化函数调用关系图API提供了一个新的端点，用于以树形结构的形式展示函数之间的调用关系。这种表达方式特别适合大模型理解，因为它提供了清晰的层次结构和父子关系。

## API端点

### POST /query_hierarchical_graph

获取层次化的函数调用关系图，支持两种模式：
1. **默认模式**：显示整个项目的函数结构，按文件分组
2. **根函数模式**：从指定的函数开始，递归显示调用链

## 请求参数

```json
{
  "project_id": "string",           // 可选：项目ID，如果不提供则使用第一个可用项目
  "root_function": "string",        // 可选：根函数名称，如果不提供则显示整个项目结构
  "max_depth": 5,                   // 可选：最大递归深度，默认5
  "include_file_info": true         // 可选：是否包含文件信息，默认true
}
```

### 参数说明

- `project_id`: 项目的唯一标识符
- `root_function`: 作为根节点的函数名称
- `max_depth`: 控制递归调用的最大深度，防止过深的调用链
- `include_file_info`: 是否在输出中包含文件路径、行号等详细信息

## 响应格式

```json
{
  "success": true,
  "data": {
    "project_id": "string",
    "root_function": "string",
    "max_depth": 5,
    "tree_structure": {
      "name": "string",
      "function_id": "string",
      "file_path": "string",
      "line_start": 123,
      "line_end": 145,
      "children": [...],
      "call_type": "string"
    },
    "total_functions": 100,
    "total_relations": 250
  }
}
```

### 响应字段说明

- `project_id`: 项目ID
- `root_function`: 根函数名称（如果指定）
- `max_depth`: 实际使用的最大深度
- `tree_structure`: 层次化树结构
- `total_functions`: 项目中的总函数数
- `total_relations`: 项目中的总调用关系数

### 树节点结构

每个树节点包含以下字段：

- `name`: 节点名称（函数名或文件/目录名）
- `function_id`: 函数ID（仅函数节点）
- `file_path`: 文件路径（如果include_file_info为true）
- `line_start`: 起始行号（如果include_file_info为true）
- `line_end`: 结束行号（如果include_file_info为true）
- `children`: 子节点列表
- `call_type`: 节点类型（"function", "max_depth"等）

## 使用示例

### 1. 获取整个项目的层次化结构

```bash
curl -X POST http://localhost:3000/query_hierarchical_graph \
  -H "Content-Type: application/json" \
  -d '{
    "max_depth": 3,
    "include_file_info": true
  }'
```

### 2. 从特定函数开始的层次化结构

```bash
curl -X POST http://localhost:3000/query_hierarchical_graph \
  -H "Content-Type: application/json" \
  -d '{
    "root_function": "main",
    "max_depth": 4,
    "include_file_info": true
  }'
```

### 3. 指定项目ID

```bash
curl -X POST http://localhost:3000/query_hierarchical_graph \
  -H "Content-Type: application/json" \
  -d '{
    "project_id": "abc123def456",
    "max_depth": 5,
    "include_file_info": false
  }'
```

## 输出示例

### 默认模式输出

```
Project Functions
├── 📁 main.rs
│   ├── main [function]
│   │   📁 src/main.rs
│   │   📍 行 1-26
│   ├── initialize_system [function]
│   │   📁 src/main.rs
│   │   📍 行 15-20
│   └── cleanup [function]
│       📁 src/main.rs
│       📍 行 22-25
├── 📁 lib.rs
│   └── lib_function [function]
│       📁 src/lib.rs
│       📍 行 1-5
```

### 根函数模式输出

```
main [function]
├── initialize_system [function]
│   ├── load_config [function]
│   ├── setup_database [function]
│   └── validate_permissions [function]
├── process_request [function]
│   ├── parse_input [function]
│   ├── validate_data [function]
│   └── execute_business_logic [function]
│       ├── calculate_result [function]
│       └── apply_rules [function]
└── cleanup [function]
    ├── close_connections [function]
    └── log_statistics [function]
```

## 特点

1. **层次清晰**: 使用树形结构展示函数调用关系
2. **易于理解**: 大模型可以轻松理解父子关系和调用层次
3. **灵活配置**: 支持多种参数配置，满足不同需求
4. **信息丰富**: 可选择包含文件路径、行号等详细信息
5. **防止循环**: 自动处理循环调用，避免无限递归

## 错误处理

- `400 Bad Request`: 请求参数无效
- `404 Not Found`: 项目或函数不存在
- `500 Internal Server Error`: 服务器内部错误

## 性能考虑

- 建议将`max_depth`控制在合理范围内（建议不超过10）
- 对于大型项目，建议先使用较小的深度进行测试
- 包含文件信息会增加响应大小，如不需要可设置为false 