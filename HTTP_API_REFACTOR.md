# HTTP服务重构完成

## 概述

根据您提供的API文档，我已经成功重构了HTTP服务，实现了符合RESTful设计原则的代码知识库查询系统API。

## 重构内容

### 1. 路由结构重构

**新的API v1路由**:
- `GET /v1/search/code` - 通过名称查询代码片段
- `GET /v1/analysis/callgraph` - 查询函数调用关系图
- `GET /v1/symbol/{symbol_name}` - 获取符号的定义和引用位置
- `GET /v1/analysis/dependencies` - 获取项目的依赖分析


**保留的旧路由** (向后兼容):
- `POST /build_graph` - 构建代码图
- `POST /query_call_graph` - 查询调用图
- `POST /query_code_snippet` - 查询代码片段

### 2. 响应模型标准化

**新的响应格式**:
```json
{
  "data": {
    // 具体的数据内容
  }
}
```

**标准化的错误格式**:
```json
{
  "code": "ERROR_CODE",
  "message": "A human-readable description of the error."
}
```

### 3. 查询参数支持

**代码搜索** (`/v1/search/code`):
- `q` (必需): 查询关键词
- `type` (可选): 过滤类型 (function, class, struct)
- `limit` (可选): 结果数量限制，默认10，最大50
- `offset` (可选): 分页偏移量，默认0
- `fuzzy` (可选): 是否启用模糊匹配，默认true

**调用图分析** (`/v1/analysis/callgraph`):
- `function` (必需): 目标函数名
- `depth` (可选): 调用栈深度，默认3，最大10
- `direction` (可选): 遍历方向 (down, up, both)，默认down

**依赖分析** (`/v1/analysis/dependencies`):
- `type` (可选): 依赖类型 (external, internal)，默认internal

### 4. 功能特性

- **多语言支持**: 自动识别文件扩展名并返回对应的编程语言
- **分页支持**: 支持limit/offset分页参数
- **Mermaid图表**: 调用图接口返回Mermaid格式的图表定义，便于前端渲染
- **错误处理**: 标准化的HTTP状态码和错误信息
- **CORS支持**: 配置了宽松的CORS策略，支持跨域请求

## 技术实现

### 架构设计
- 使用Axum框架构建HTTP服务
- 采用RESTful API设计原则
- 支持查询参数和路径参数
- 统一的错误处理和响应格式

### 数据模型
- 重构了响应模型，符合API文档规范
- 支持代码片段、调用图、符号信息、依赖关系等数据结构
- 保持了与现有代码图的兼容性

### 处理器函数
- `search_code`: 实现代码搜索功能，支持类型过滤和分页
- `get_call_graph`: 生成函数调用关系图，支持深度和方向控制
- `get_symbol_info`: 获取符号定义和引用信息
- `get_dependencies`: 分析项目依赖关系


## 使用示例

### 1. 搜索代码片段
```bash
curl "http://localhost:3000/v1/search/code?q=UserController&type=class&limit=5"
```

### 2. 获取调用图
```bash
curl "http://localhost:3000/v1/analysis/callgraph?function=createUser&depth=3&direction=both"
```

### 3. 获取符号信息
```bash
curl "http://localhost:3000/v1/symbol/calculateTotal"
```

### 4. 获取依赖分析
```bash
curl "http://localhost:3000/v1/analysis/dependencies?type=internal"
```


