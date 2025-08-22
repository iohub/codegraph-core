# CodeGraph Core 架构文档

## 重构后的模块结构

经过重构，CodeGraph Core 现在采用了更清晰的模块化架构，遵循单一职责原则和关注点分离。

### 核心模块

#### 1. `codegraph` - 核心代码图模块
- **职责**: 提供代码图的核心数据结构和算法
- **包含**:
  - `types.rs` - 核心类型定义（GraphNode, FunctionInfo, EntityNode等）
  - `graph.rs` - 代码图实现
  - `parser.rs` - 代码解析器
  - `treesitter/` - Tree-sitter语言支持
  - `analyzers/` - 语言特定的代码分析器

#### 2. `storage` - 存储模块
- **职责**: 管理代码图的持久化、缓存和增量更新
- **包含**:
  - `cache.rs` - 缓存管理
  - `persistence.rs` - 持久化管理
  - `incremental.rs` - 增量更新管理
  - `petgraph_storage.rs` - PetGraph格式的存储实现

#### 3. `services` - 服务模块
- **职责**: 提供高级业务逻辑服务
- **包含**:
  - `snippet_service.rs` - 代码片段查询服务

#### 4. `repository` - 仓库管理模块
- **职责**: 管理代码仓库的分析和查询
- **包含**:
  - `repository_manager.rs` - 仓库管理器，整合代码分析、增量更新和查询功能

#### 5. `cli` - 命令行接口
- **职责**: 提供命令行工具
- **包含**:
  - `runner.rs` - 主要运行逻辑
  - `analyze.rs` - 分析命令实现
  - `args.rs` - 命令行参数定义

#### 6. `http` - HTTP服务模块
- **职责**: 提供HTTP API接口

### 架构优势

1. **单一职责**: 每个模块都有明确的职责边界
2. **解耦合**: 模块间通过明确的接口进行交互
3. **可维护性**: 代码结构清晰，易于理解和修改
4. **可扩展性**: 新功能可以添加到相应的模块中
5. **可测试性**: 模块化设计便于单元测试

### 依赖关系

```
cli → repository → services → codegraph
  ↓        ↓         ↓
storage ←--------------
```

- `cli` 依赖 `repository` 和 `storage`
- `repository` 依赖 `services` 和 `storage`
- `services` 依赖 `codegraph`
- `storage` 依赖 `codegraph` 的类型定义

### 重构变更

1. **移动的文件**:
   - `src/codegraph/petgraph_storage.rs` → `src/storage/petgraph_storage.rs`
   - `src/codegraph/snippet_service.rs` → `src/services/snippet_service.rs`
   - `src/codegraph/repository_manager.rs` → `src/repository/repository_manager.rs`

2. **更新的导入路径**:
   - 所有引用已移动模块的文件都已更新导入路径
   - 保持了向后兼容性

3. **模块声明**:
   - 在 `src/lib.rs` 中添加了新模块声明
   - 更新了 `src/codegraph/mod.rs` 的导出

### 使用示例

```rust
// 使用存储功能
use crate::storage::PetGraphStorageManager;

// 使用服务功能
use crate::services::SnippetService;

// 使用仓库管理
use crate::repository::RepositoryManager;

// 使用核心代码图功能
use crate::codegraph::{CodeGraph, types::FunctionInfo};
``` 