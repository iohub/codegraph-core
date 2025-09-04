# 增量构建功能说明

## 概述

CodeGraph Core 现在支持增量构建功能，可以显著提高重复构建的速度。该功能通过以下方式实现：

1. **本地数据库缓存**: 首次构建后，图数据会保存到本地数据库
2. **MD5哈希检查**: 在扫描文件时，会计算每个文件的MD5哈希值
3. **智能跳过**: 如果文件哈希值未变化，则跳过解析过程

## 工作原理

### 1. 首次构建
- 扫描目录下的所有支持的文件
- 解析每个文件，提取函数信息和调用关系
- 构建完整的代码图
- 保存图数据到本地数据库
- 保存每个文件的MD5哈希值

### 2. 后续构建
- 从本地数据库加载现有的图数据
- 扫描目录下的所有文件
- 计算每个文件的MD5哈希值
- 比较哈希值：
  - 如果哈希值一致：跳过文件解析
  - 如果哈希值不同：重新解析文件
- 合并新解析的函数到现有图中
- 更新文件哈希值

## 使用方法

### 基本用法

```rust
use crate::codegraph::parser::CodeParser;

let mut parser = CodeParser::new();

// 第一次构建（全量构建）
let graph1 = parser.build_petgraph_code_graph(&project_dir)?;

// 修改某些文件...

// 第二次构建（增量构建，自动检测变化）
let graph2 = parser.build_petgraph_code_graph(&project_dir)?;
```

### 性能提升

- **首次构建**: 正常速度，需要解析所有文件
- **增量构建**: 显著提升，只解析变化的文件
- **无变化构建**: 最快速度，直接加载缓存数据

## 存储结构

增量构建功能会在项目目录下创建 `.codegraph_cache` 目录，包含：

```
.codegraph_cache/
├── {project_name}/
│   ├── graph.json          # 图数据（JSON格式）
│   ├── graph.bin           # 图数据（二进制格式）
│   └── file_hashes.json    # 文件哈希值映射
```

## 配置选项

### 存储模式

支持三种存储模式：

- `Json`: 只保存JSON格式
- `Binary`: 只保存二进制格式  
- `Both`: 同时保存两种格式

```rust
use crate::storage::PersistenceManager;

let persistence = PersistenceManager::with_storage_mode(StorageMode::Both);
```

## 注意事项

1. **文件删除**: 如果文件被删除，相关数据会从图中移除
2. **哈希冲突**: 使用MD5哈希值，理论上存在碰撞可能性，但实际应用中概率极低
3. **存储空间**: 增量构建会占用额外的磁盘空间存储缓存数据
4. **兼容性**: 增量构建功能向后兼容，不影响现有的全量构建功能

## 测试

运行测试以验证功能：

```bash
cargo test test_incremental_build_with_md5_checking
```

## 故障排除

### 常见问题

1. **缓存损坏**: 删除 `.codegraph_cache` 目录重新构建
2. **权限问题**: 确保有读写项目目录的权限
3. **存储空间不足**: 检查磁盘空间是否充足

### 调试信息

启用详细日志以查看增量构建过程：

```bash
RUST_LOG=debug cargo run -- analyze /path/to/project
```

## 未来改进

- [ ] 支持文件监控（inotify/fsevents）
- [ ] 增量更新调用关系
- [ ] 压缩缓存数据
- [ ] 缓存过期策略
- [ ] 分布式缓存支持 