# CodeGraph 重构架构说明

## 新的目录结构

```
src/codegraph/
├── mod.rs              # 主模块，重新导出核心类型
├── core.rs             # 核心数据结构 (CodeGraph)
├── types.rs            # 类型定义
├── graph.rs            # 图构建器 (GraphBuilder)
├── languages/          # 语言支持模块
│   ├── mod.rs         # 语言模块主文件
│   ├── detection.rs   # 语言检测
│   ├── rust.rs        # Rust 分析器
│   ├── java.rs        # Java 分析器
│   ├── python.rs      # Python 分析器
│   ├── cpp.rs         # C++ 分析器
│   ├── typescript.rs  # TypeScript 分析器
│   └── javascript.rs  # JavaScript 分析器
├── parsing/            # 解析模块
│   └── mod.rs         # 代码解析器
├── analysis/           # 分析模块
│   └── mod.rs         # 分析器注册表和编排器
├── storage/            # 存储模块
│   └── mod.rs         # 图存储接口和实现
├── query/              # 查询模块
│   └── mod.rs         # 图查询接口和实现
└── utils/              # 工具模块
    └── mod.rs         # 通用工具函数
```
