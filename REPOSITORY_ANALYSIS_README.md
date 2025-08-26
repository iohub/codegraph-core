# CodeGraph 仓库分析功能

本项目已根据planning文档实现了完整的仓库分析功能，包括增量更新、实体提取和代码片段查询。

## 主要功能

### 1. 实体提取
- **函数提取**: 使用TreeSitter解析器提取函数声明
- **类/结构体提取**: 支持Rust、Python、JavaScript、TypeScript、Java、C++等语言
- **命名空间支持**: 自动识别和提取命名空间信息

### 2. 调用关系分析
- **函数调用图**: 构建完整的函数调用关系图
- **跨文件解析**: 支持跨文件的函数调用关系解析
- **未解析调用标记**: 对于无法解析的调用，标记为未解析状态

### 3. 增量更新
- **MD5变更检测**: 基于文件MD5哈希值检测文件变更
- **原子更新**: 文件级别的原子更新，确保数据一致性
- **索引重建**: 自动重建受影响的索引和关系

### 4. 代码片段查询
- **按需读取**: 支持按需从磁盘读取代码片段
- **智能缓存**: 自动缓存常用代码片段，提高查询性能
- **行范围支持**: 精确的行范围代码片段提取

## 架构设计

### 核心组件

```
RepositoryManager (仓库管理器)
├── EntityGraph (实体图)
├── PetCodeGraph (调用图)
├── CodeParser (代码解析器)
├── IncrementalManager (增量更新管理器)
└── SnippetService (代码片段服务)
```

### 数据流

1. **文件扫描** → 检测支持的源代码文件
2. **MD5计算** → 检查文件是否变更
3. **TreeSitter解析** → 提取AST符号
4. **实体提取** → 生成函数和类信息
5. **关系分析** → 构建调用关系图
6. **索引更新** → 更新各种索引结构
7. **状态持久化** → 保存分析结果

## 使用方法

### CLI命令

#### 1. 仓库分析
```bash
# 全量分析仓库
cargo run -- repo --path /path/to/repo --stats

# 增量更新模式
cargo run -- repo --path /path/to/repo --incremental --stats

# 搜索特定实体
cargo run -- repo --path /path/to/repo --search "function_name" --stats

# 指定状态目录
cargo run -- repo --path /path/to/repo --state-dir ./my-codegraph --stats
```

#### 2. 传统代码图分析
```bash
# 分析代码并导出为JSON
cargo run -- analyze --input /path/to/code --output graph.json --format json



# 导出为DOT格式
cargo run -- analyze --input /path/to/code --output graph.dot --format dot
```

### 编程接口

#### 1. 创建仓库管理器
```rust
use codegraph::RepositoryManager;

let mut repo_manager = RepositoryManager::new(PathBuf::from("/path/to/repo"));
repo_manager.initialize()?;
```

#### 2. 增量更新文件
```rust
// 更新单个文件
repo_manager.refresh_file(&file_path)?;

// 批量更新文件
repo_manager.refresh_files(&file_paths)?;
```

#### 3. 查询功能
```rust
// 搜索实体
let results = repo_manager.search_entities("function_name");

// 获取函数调用者
let callers = repo_manager.get_function_callers(&function_id);

// 获取代码片段
let snippet = repo_manager.get_snippet(&entity_id, "function")?;
```

#### 4. 状态管理
```rust
// 保存状态
repo_manager.save_state(&state_dir)?;

// 加载状态
repo_manager.load_state(&state_dir)?;
```

## 支持的语言

- **Rust** (.rs)
- **Python** (.py, .py3, .pyx)
- **JavaScript** (.js, .jsx)
- **TypeScript** (.ts, .tsx)
- **Java** (.java)
- **C++** (.cpp, .cc, .cxx, .c++, .c, .h, .hpp, .hxx, .hh)

## 性能特性

### 1. 增量更新
- 只分析变更的文件，避免全量重建
- MD5哈希检测，快速识别文件变更
- 原子更新操作，确保数据一致性

### 2. 智能缓存
- 代码片段按需读取和缓存
- 热点代码片段预加载
- 可配置的缓存大小限制

### 3. 并行处理
- 文件解析支持并行化（基于Rayon）
- 图更新批量处理，减少锁竞争

## 数据持久化

### 1. 图数据
- `entity_graph.json`: 实体图数据
- `call_graph.json`: 调用图数据

### 2. 增量状态
- `incremental_state.json`: 文件MD5和索引信息

### 3. 存储格式
- 使用JSON格式，便于调试和迁移
- 支持版本兼容性检查
- 可选的压缩存储

## 监控和调试

### 1. 统计信息
- 实体数量统计
- 调用关系统计
- 缓存命中率统计
- 解析成功率统计

### 2. 日志记录
- 详细的解析过程日志
- 错误和警告信息
- 性能指标记录

### 3. 调试工具
- 图结构可视化（DOT）
- 代码片段预览
- 关系查询调试

## 扩展性

### 1. 新语言支持
- 实现TreeSitter解析器
- 添加语言特定的实体提取逻辑
- 配置语言特定的忽略规则

### 2. 新实体类型
- 扩展EntityNode枚举
- 实现相应的提取和查询逻辑
- 添加新的关系类型

### 3. 新查询功能
- 扩展SnippetService
- 实现复杂的图查询算法
- 添加统计分析功能

## 最佳实践

### 1. 性能优化
- 合理设置缓存大小
- 使用增量更新模式
- 定期清理未使用的缓存

### 2. 数据一致性
- 定期备份状态数据
- 监控解析错误率
- 验证图结构完整性

### 3. 资源管理
- 控制并发解析数量
- 限制内存使用
- 监控磁盘I/O

## 故障排除

### 1. 常见问题
- **解析失败**: 检查文件编码和语法
- **内存不足**: 减少缓存大小或并发数
- **性能下降**: 检查文件变更频率

### 2. 调试步骤
- 启用详细日志
- 检查文件权限
- 验证依赖完整性

### 3. 恢复策略
- 删除状态文件重新分析
- 检查文件完整性
- 回滚到已知良好状态

## 未来计划

### M1: 最小可用版本 ✅
- [x] 基本实体提取
- [x] 调用关系分析
- [x] 增量更新框架

### M2: 功能扩展
- [ ] 更完整的类关系分析
- [ ] 导入/依赖关系
- [ ] 类型信息提取

### M3: 性能优化
- [ ] 并行解析优化
- [ ] 缓存策略改进
- [ ] 内存使用优化

### M4: 高级功能
- [ ] 循环依赖检测
- [ ] 影响分析
- [ ] 重构建议

## 贡献指南

欢迎贡献代码和想法！请参考以下步骤：

1. Fork项目
2. 创建功能分支
3. 实现功能并添加测试
4. 提交Pull Request

## 许可证

本项目采用MIT许可证，详见LICENSE文件。 