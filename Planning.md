# CodeGraph HTTP服务封装计划

## 项目概述
将现有的CodeGraph CLI工具封装成HTTP服务，提供RESTful API接口来构建代码图、查询调用关系和获取代码片段。

## 现有代码分析
- **核心功能**: 已实现代码解析、图构建、调用关系分析
- **支持语言**: C++, Java, Python, JavaScript, Rust, TypeScript等
- **图结构**: 基于petgraph的有向图，支持函数调用关系
- **存储**: JSON格式，支持多种导出格式

## 技术架构设计

### 1. HTTP服务框架选择
**推荐使用 `axum` + `tokio`**
- 轻量级，性能优秀
- 与现有tokio依赖兼容
- 支持异步处理
- 易于部署和分发

### 2. 项目结构规划
```
src/
├── main.rs                 # HTTP服务入口
├── lib.rs                  # 库入口
├── cli/                    # 保留原有CLI功能
├── codegraph/              # 保留原有核心功能
├── http/                   # 新增HTTP服务模块
│   ├── mod.rs             # HTTP模块入口
│   ├── server.rs          # HTTP服务器
│   ├── handlers/          # 请求处理器
│   │   ├── mod.rs
│   │   ├── build.rs       # 构建代码图
│   │   ├── query.rs       # 查询调用关系
│   │   └── snippet.rs     # 获取代码片段
│   ├── models/            # HTTP请求/响应模型
│   │   ├── mod.rs
│   │   ├── build.rs
│   │   ├── query.rs
│   │   └── snippet.rs
│   └── middleware/        # 中间件
│       ├── mod.rs
│       └── cors.rs        # CORS支持
└── storage/               # 新增存储管理模块
    ├── mod.rs
    ├── cache.rs           # 内存缓存
    ├── persistence.rs     # 持久化存储
    └── incremental.rs     # 增量更新逻辑
```

## 功能模块设计

### 1. 构建代码图 (POST /build_graph)
**请求格式**:
```json
{
  "project_dir": "/path/to/project",
  "force_rebuild": false,
  "exclude_patterns": ["node_modules", ".venv", "__pycache__", "target"]
}
```

**响应格式**:
```json
{
  "success": true,
  "data": {
    "project_id": "uuid",
    "total_files": 150,
    "total_functions": 1200,
    "build_time_ms": 2500,
    "cache_hit_rate": 0.85
  }
}
```

**核心逻辑**:
- 文件过滤：自动跳过依赖目录
- MD5校验：计算文件哈希，只处理变更文件
- 增量更新：复用已有图结构，只更新变更部分
- 并发处理：多线程解析多个文件

### 2. 查询调用关系 (POST /query_call_graph)
**请求格式**:
```json
{
  "filepath": "/path/to/file.cpp",
  "function_name": "optional_function_name",
  "max_depth": 3
}
```

**响应格式**:
```json
{
  "success": true,
  "data": {
    "filepath": "/path/to/file.cpp",
    "functions": [
      {
        "id": "uuid",
        "name": "main",
        "line_start": 1,
        "line_end": 50,
        "callers": [],
        "callees": [
          {
            "function_name": "process_data",
            "file_path": "/path/to/utils.cpp",
            "line_number": 25
          }
        ]
      }
    ]
  }
}
```

### 3. 获取代码片段 (POST /query_code_snippet)
**请求格式**:
```json
{
  "filepath": "/path/to/file.cpp",
  "function_name": "process_data",
  "include_context": true,
  "context_lines": 5
}
```

**响应格式**:
```json
{
  "success": true,
  "data": {
    "filepath": "/path/to/file.cpp",
    "function_name": "process_data",
    "code_snippet": "void process_data(int* data, size_t size) {\n    // ...\n}",
    "line_start": 25,
    "line_end": 45,
    "language": "cpp"
  }
}
```

## 增量更新机制

### 1. 文件变更检测
- 使用MD5/SHA256哈希算法
- 缓存文件哈希到内存和磁盘
- 支持文件修改时间检查（快速预筛选）

### 2. 图结构增量更新
- 删除变更文件的旧节点和边
- 重新解析变更文件
- 更新调用关系
- 保持图的一致性

### 3. 缓存策略
- 内存缓存：热点数据
- 磁盘缓存：持久化存储
- LRU淘汰策略

## 依赖包选择

### 核心HTTP依赖
```toml
[dependencies]
axum = "0.7"           # HTTP框架
tokio = { version = "1.43", features = ["full"] }  # 异步运行时
tower = "0.4"          # 中间件框架
tower-http = { version = "0.5", features = ["cors"] }  # HTTP中间件
```

### 文件处理依赖
```toml
[dependencies]
md-5 = "0.10"          # MD5哈希计算
notify = "6.1"          # 文件系统监控
```

### 序列化依赖
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## 性能优化策略

### 1. 并发处理
- 文件解析：多线程并行
- 图构建：异步处理
- 请求处理：异步非阻塞

### 2. 内存管理
- 流式处理大文件
- 智能缓存策略
- 内存池复用

### 3. 存储优化
- 压缩存储
- 索引优化
- 分片存储

## 部署和分发

### 1. 可执行文件
- 静态链接依赖
- 跨平台编译支持
- 最小化二进制大小

### 2. 配置管理
- 环境变量配置
- 配置文件支持
- 命令行参数

### 3. 日志和监控
- 结构化日志
- 性能指标
- 健康检查接口

## 开发计划

### 阶段1: 基础HTTP服务 (1-2天)
- 搭建axum框架
- 实现基本路由
- 添加CORS支持

### 阶段2: 核心API实现 (3-4天)
- 实现build_graph接口
- 实现query_call_graph接口
- 实现query_code_snippet接口

### 阶段3: 增量更新机制 (2-3天)
- 文件哈希计算
- 增量更新逻辑
- 缓存管理

### 阶段4: 性能优化 (2-3天)
- 并发处理
- 内存优化
- 存储优化

### 阶段5: 测试和部署 (1-2天)
- 单元测试
- 集成测试
- 性能测试
- 部署文档

## 风险评估

### 技术风险
- **内存占用**: 大项目可能导致内存溢出
  - 缓解：流式处理、分片存储
- **并发安全**: 多线程访问图结构
  - 缓解：RwLock、原子操作
- **文件系统**: 大量文件IO操作
  - 缓解：异步IO、批量处理

### 兼容性风险
- **平台差异**: 不同操作系统的路径处理
  - 缓解：使用标准库、跨平台测试
- **依赖版本**: 依赖包版本冲突
  - 缓解：锁定版本、兼容性测试

## 成功指标

### 功能指标
- 支持所有现有CLI功能
- API响应时间 < 100ms (简单查询)
- 支持项目大小 > 100MB源码

### 性能指标
- 内存使用 < 2GB (中等项目)
- 启动时间 < 5秒
- 并发处理 > 10个请求

### 质量指标
- 测试覆盖率 > 80%
- 零内存泄漏
- 错误处理完善

## 总结

本计划将CodeGraph从CLI工具转换为功能完整的HTTP服务，保持轻量级和可移植性的同时，提供强大的代码分析能力。通过增量更新机制和性能优化，确保服务能够高效处理大型代码项目。 