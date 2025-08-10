# CodeGraph HTTP Service

CodeGraph现在支持HTTP服务模式，提供RESTful API接口来构建代码图、查询调用关系和获取代码片段。

## 启动HTTP服务

### 方式1: 命令行参数
```bash
# 启动HTTP服务器在8080端口
cargo run -- --server 127.0.0.1:8080

# 启动HTTP服务器在指定IP和端口
cargo run -- --server 0.0.0.0:3000
```

### 方式2: 环境变量
```bash
export CODEGRAPH_SERVER=127.0.0.1:8080
cargo run
```

## API接口

### 1. 健康检查
```
GET /health
```

**响应示例:**
```json
{
  "success": true,
  "data": "CodeGraph HTTP service is running"
}
```

### 2. 构建代码图
```
POST /build_graph
```

**请求格式:**
```json
{
  "project_dir": "/path/to/project",
  "force_rebuild": false,
  "exclude_patterns": ["node_modules", ".venv", "__pycache__", "target"]
}
```

**响应格式:**
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

### 3. 查询调用关系
```
POST /query_call_graph
```

**请求格式:**
```json
{
  "filepath": "/path/to/file.cpp",
  "function_name": "optional_function_name",
  "max_depth": 3
}
```

**响应格式:**
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

### 4. 获取代码片段
```
POST /query_code_snippet
```

**请求格式:**
```json
{
  "filepath": "/path/to/file.cpp",
  "function_name": "process_data",
  "include_context": true,
  "context_lines": 5
}
```

**响应格式:**
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

## 使用示例

### 使用curl测试API

```bash
# 健康检查
curl http://localhost:8080/health

# 构建代码图
curl -X POST http://localhost:8080/build_graph \
  -H "Content-Type: application/json" \
  -d '{
    "project_dir": "/home/user/projects/myproject",
    "force_rebuild": false,
    "exclude_patterns": ["node_modules", ".git"]
  }'

# 查询调用关系
curl -X POST http://localhost:8080/query_call_graph \
  -H "Content-Type: application/json" \
  -d '{
    "filepath": "/home/user/projects/myproject/src/main.cpp",
    "function_name": "main",
    "max_depth": 2
  }'

# 获取代码片段
curl -X POST http://localhost:8080/query_code_snippet \
  -H "Content-Type: application/json" \
  -d '{
    "filepath": "/home/user/projects/myproject/src/main.cpp",
    "function_name": "main",
    "include_context": true,
    "context_lines": 3
  }'
```

### 使用Python测试API

```python
import requests
import json

base_url = "http://localhost:8080"

# 健康检查
response = requests.get(f"{base_url}/health")
print("Health check:", response.json())

# 构建代码图
build_data = {
    "project_dir": "/home/user/projects/myproject",
    "force_rebuild": False,
    "exclude_patterns": ["node_modules", ".git"]
}
response = requests.post(f"{base_url}/build_graph", json=build_data)
print("Build graph:", response.json())

# 查询调用关系
query_data = {
    "filepath": "/home/user/projects/myproject/src/main.cpp",
    "function_name": "main",
    "max_depth": 2
}
response = requests.post(f"{base_url}/query_call_graph", json=query_data)
print("Query call graph:", response.json())
```

### 使用JavaScript测试API

```javascript
const baseUrl = 'http://localhost:8080';

// 健康检查
fetch(`${baseUrl}/health`)
  .then(response => response.json())
  .then(data => console.log('Health check:', data));

// 构建代码图
const buildData = {
  project_dir: '/home/user/projects/myproject',
  force_rebuild: false,
  exclude_patterns: ['node_modules', '.git']
};

fetch(`${baseUrl}/build_graph`, {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify(buildData),
})
.then(response => response.json())
.then(data => console.log('Build graph:', data));
```

## 配置选项

### 环境变量
- `CODEGRAPH_SERVER`: HTTP服务器地址 (默认: 127.0.0.1:8080)
- `CODEGRAPH_CACHE_SIZE`: 缓存大小 (默认: 1000)
- `CODEGRAPH_CACHE_TTL`: 缓存TTL秒数 (默认: 3600)

### 缓存配置
- 内存缓存: 热点数据，LRU淘汰策略
- 磁盘缓存: 持久化存储，项目级别的图结构
- 文件哈希: MD5校验，支持增量更新

## 性能特性

- **异步处理**: 基于tokio的异步运行时
- **并发支持**: 多线程文件解析和图构建
- **增量更新**: 只处理变更的文件
- **智能缓存**: 多层缓存策略
- **CORS支持**: 跨域请求支持

## 错误处理

所有API接口都返回统一的响应格式：

**成功响应:**
```json
{
  "success": true,
  "data": {...}
}
```

**错误响应:**
```json
{
  "success": false,
  "error": "错误描述",
  "code": 400
}
```

**HTTP状态码:**
- `200`: 成功
- `400`: 请求参数错误
- `404`: 资源不存在
- `500`: 服务器内部错误

## 开发模式

在开发模式下，可以启用详细日志：

```bash
RUST_LOG=debug cargo run -- --server 127.0.0.1:8080
```

## 部署建议

### 生产环境
- 使用反向代理 (nginx, Apache)
- 配置SSL/TLS
- 设置适当的CORS策略
- 监控和日志记录

### Docker部署
```dockerfile
FROM rust:1.70 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/codegraph-cli /usr/local/bin/codegraph
EXPOSE 8080
CMD ["codegraph", "--server", "0.0.0.0:8080"]
```

## 故障排除

### 常见问题

1. **端口被占用**
   ```bash
   # 检查端口占用
   netstat -tulpn | grep :8080
   
   # 使用其他端口
   cargo run -- --server 127.0.0.1:8081
   ```

2. **权限问题**
   ```bash
   # 确保有读取项目目录的权限
   sudo chown -R $USER:$USER /path/to/project
   ```

3. **内存不足**
   - 减少并发处理的文件数量
   - 调整缓存大小
   - 使用增量更新模式

### 日志分析
```bash
# 启用详细日志
RUST_LOG=debug cargo run -- --server 127.0.0.1:8080

# 查看错误日志
grep "ERROR" codegraph.log
``` 