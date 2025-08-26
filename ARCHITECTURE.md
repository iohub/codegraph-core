# CodeGraph Core 架构说明

## 目录结构重构

经过重构，代码目录结构更加清晰，职责分离更加明确：

### 顶层目录结构

```
src/
├── codegraph/          # 核心代码图分析模块
│   ├── graph.rs        # 图结构定义
│   ├── parser.rs       # 代码解析器
│   ├── types.rs        # 类型定义
│   ├── treesitter/     # Tree-sitter解析器
│   └── repository_manager.rs  # 仓库管理器
├── services/           # 服务层模块
│   ├── snippet_service.rs  # 代码片段服务
│   └── analyzer.rs     # 代码分析服务
├── storage/            # 存储层模块
│   ├── cache.rs        # 缓存管理
│   ├── persistence.rs  # 持久化
│   ├── incremental.rs  # 增量更新
│   └── petgraph_storage.rs  # PetGraph存储格式
├── http/               # HTTP服务
├── cli/                # 命令行工具
└── lib.rs              # 库入口
```

### 模块职责说明

#### 1. codegraph/ - 核心代码图分析
- **graph.rs**: 定义代码图的基础数据结构
- **parser.rs**: 负责解析源代码文件，构建代码图
- **types.rs**: 定义所有核心类型和数据结构
- **treesitter/**: 基于Tree-sitter的语法解析器
- **repository_manager.rs**: 管理整个代码仓库的分析状态

#### 2. services/ - 服务层
- **snippet_service.rs**: 提供代码片段查询、缓存和管理服务
- **analyzer.rs**: 提供高级代码分析功能，如循环依赖检测、复杂度分析等

#### 3. storage/ - 存储层
- **cache.rs**: 内存缓存管理
- **persistence.rs**: 数据持久化
- **incremental.rs**: 增量更新管理
- **petgraph_storage.rs**: PetGraph格式的存储和导出

### 重构优势

1. **职责分离**: 每个目录都有明确的职责边界
2. **模块化**: 服务层和存储层独立，便于测试和维护
3. **可扩展性**: 新增服务或存储后端更加容易
4. **依赖清晰**: 导入路径更加清晰，避免循环依赖

### 导入路径更新

重构后，相关模块的导入路径已更新：

- `SnippetService` 从 `crate::codegraph::snippet_service` 改为 `crate::services::SnippetService`
- `CodeAnalyzer` 从 `crate::codegraph::analyzer` 改为 `crate::services::CodeAnalyzer`
- `PetGraphStorage` 从 `crate::codegraph::petgraph_storage` 改为 `crate::storage::PetGraphStorage`

### 使用示例

```rust
use crate::services::{SnippetService, CodeAnalyzer};
use crate::storage::{PetGraphStorage, PetGraphStorageManager};

// 使用代码片段服务
let snippet_service = SnippetService::new(snippet_index);

// 使用代码分析器
let analyzer = CodeAnalyzer::new();

// 使用存储功能
let storage = PetGraphStorage::from_petgraph(&code_graph);
``` 