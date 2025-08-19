## 重构规划：基于 analyzers 的解析与 CodeGraph 构建

### 目标
- **统一解析流程**：通过 `analyzers` 运行对应语言的解析器，抽取函数、调用关系与片段位置信息。
- **构建代码图（CodeGraph）**：将解析结果标准化并落入 `CodeGraph`（`src/codegraph/graph.rs`），支持 JSON、Mermaid、DOT 导出。
- **生成代码片段索引**：按函数和文件建立片段索引，提供快速跳转与检索能力。

### 范围
- 支持语言：Rust、Java、JavaScript、TypeScript、Python、C++（以现有 `*_analyzer.rs` 为基础）。
- 输入：代码仓库根目录或文件清单。
- 输出：
  - `target/codegraph/graph.json`（序列化的 `CodeGraph`）
  - `target/codegraph/graph.mmd`（Mermaid）
  - `target/codegraph/graph.dot`（DOT）
  - `target/codegraph/snippets.json`（代码片段索引）

### 现状简述
- `src/codegraph/analyzers/` 已有多语言 `*_analyzer.rs` 和 `*_parser.rs`。
- `src/codegraph/treesitter/` 提供 Tree-sitter 语言识别、骨架、AST 结构。
- `src/codegraph/graph.rs` 已定义 `CodeGraph`：包含 `functions`、`call_relations`、`graph_relations` 与统计信息（`CodeGraphStats`）。

### 总体架构与数据流
1. **发现阶段（Discovery）**
   - 扫描源代码目录，基于扩展名或内容进行语言识别（复用 `treesitter::language_id`）。
   - 产出：文件列表（含语言）。
2. **解析阶段（Parsing）**
   - 针对每个文件，选择对应 `LanguageAnalyzer` 执行解析。
   - 产出：标准化的函数清单（`FunctionInfo`）、调用关系（`CallRelation`）、片段（`Snippet`）。
3. **聚合阶段（Aggregation）**
   - 使用 `CodeGraph::add_function`、`CodeGraph::add_call_relation` 聚合为 `CodeGraph`。
   - 维护 `file_functions`、`function_names` 与统计。
4. **索引阶段（Indexing）**
   - 构建 `snippets.json`：按 `function_id`、`file_path`、`range`、`language` 与可选 `preview` 建立索引。
5. **导出阶段（Exporting）**
   - 使用现有 `CodeGraph::to_json`、`to_mermaid`、`to_dot` 导出。

### 统一接口设计
- 新增 `LanguageAnalyzer` trait（若不存在）
  - `fn language(&self) -> LanguageId`
  - `fn parse_file(&self, path: &Path) -> anyhow::Result<ParsedUnit>`
  - `fn extract_functions(&self, unit: &ParsedUnit) -> Vec<FunctionInfo>`
  - `fn extract_calls(&self, unit: &ParsedUnit) -> Vec<CallRelation>`
  - `fn extract_snippets(&self, unit: &ParsedUnit) -> Vec<Snippet>`
- `AnalyzerRegistry`
  - `fn by_language(lang: LanguageId) -> &'static dyn LanguageAnalyzer`
  - 基于现有的 `*_analyzer.rs` 注册。
- `Orchestrator`（新增）
  - `fn run(root: &Path, options: AnalyzeOptions) -> anyhow::Result<AnalyzeResult>`
  - 内部并发遍历与调度，收敛为 `CodeGraph` 与 `snippets`。

### 关键结构（建议）
- `FunctionInfo`（已有字段参考 `CodeGraph` 用法）：
  - `id: Uuid`, `name: String`, `file_path: PathBuf`, `language: String`, `range: (start_line, start_col, end_line, end_col)` 等。
- `CallRelation`（沿用）：
  - `caller_id`, `callee_id`（可为 `Option<Uuid>` 若未解析），`caller_name`, `callee_name`, `is_resolved`。
- `Snippet`（新增）
  - `id: Uuid`（与 `FunctionInfo.id` 对齐或独立）
  - `file_path: PathBuf`, `language: String`
  - `range: (start_line, start_col, end_line, end_col)`
  - `function_id: Option<Uuid>`
  - `preview: Option<String>`（截取前若干字符）
- `AnalyzeOptions`
  - `languages: Option<Vec<LanguageId>>`, `max_workers: usize`, `include_tests: bool`, `follow_symlinks: bool` 等。
- `AnalyzeResult`
  - `graph: CodeGraph`, `snippets: Vec<Snippet>`

### 解析与抽取规则（概要）
- 函数（或方法、构造器）
  - 名称、限定名（含类/模块作用域）、文件路径、语言、位置范围。
- 调用
  - 调用点解析：调用者函数上下文内，识别 callee 的标识符/选择子表达式；
  - 解析策略：
    - 先基于同文件/同作用域进行名称解析与匹配；
    - 再退化为基于函数名的模糊匹配，标记 `is_resolved=false`；
  - 记录调用者/被调者的名称与可解析到的 `Uuid`。
- 片段
  - 以函数体范围为主，或基于语句块/类定义生成附加片段（可选）。

### 索引文件格式（`snippets.json`）
- 顶层数组或对象（例如按 `file_path` 分组）。
- 字段：`id`, `file_path`, `language`, `range`, `function_id`, `preview`。
- 生成策略：
  - 优先为每个 `FunctionInfo` 生成一个主片段；
  - 可选：为类、接口、枚举、模块等生成片段（后续迭代）。

### CLI 方案（新增）
- 二进制：`codegraph-core` 或在现有 bin 中新增子命令：
  - `codegraph analyze --root <path> [--lang rust,java,ts,js,python,cpp] [--out target/codegraph] [--format json,mermaid,dot] [--workers N] [--include-tests]`
- 行为：
  1. 扫描与语言识别
  2. 并发解析与抽取
  3. 聚合为 `CodeGraph`
  4. 写出 `graph.json`、`graph.mmd`、`graph.dot`、`snippets.json`

### 并发与性能
- 文件级并发：基于线程池（`max_workers`）分发到不同语言解析器。
- 去重与聚合：
  - 函数名映射/文件函数映射使用线程安全队列或集中单线程收敛；
  - `Uuid` 在提取阶段即确定，避免二次扫描。
- I/O：批量写出，避免频繁小写。

### 错误与降级
- 单文件失败不阻断全局：记录 `errors.json`。
- 解析失败的调用标记 `is_resolved=false`，仍纳入图中（虚线/斜体样式已在 `to_mermaid`/`to_dot` 中体现）。

### 里程碑
- M1：编排器与注册表落地，Rust/JS/TS 跑通最小闭环（函数 + 调用 + 片段 + 导出）。
- M2：Java/Python/CPP 接入；并发优化；`snippets.json` 完整。
- M3：解析精度优化（跨文件/模块解析），故障与指标完善。

### 验收标准
- 在示例仓库/本仓库自举运行：
  - 生成的 `graph.json` 可被 `serde_json` 成功读取；
  - `graph.mmd` 在 Mermaid 渲染正常；`graph.dot` 用 Graphviz 渲染成功；
  - `snippets.json` 可通过 `function_id` 与 `file_path` 快速定位源码片段；
  - 统计信息（文件数、语言数、函数总数、已解析/未解析调用数）合理。

### 风险与对策
- 复杂语言特性导致解析不完整：按语言分阶段增强规则；保留未解析调用。
- 大型仓库性能瓶颈：增加缓存与增量模式；限制最大并发与跳过目录策略。
- 跨模块/依赖解析：第一期忽略外部依赖，记录未解析调用；后续引入符号索引。

### 代码骨架（示意，不在本次直接提交）
```rust
pub struct AnalyzerOrchestrator { /* ... */ }

impl AnalyzerOrchestrator {
    pub fn run(root: &Path, options: AnalyzeOptions) -> anyhow::Result<AnalyzeResult> {
        // 1) 发现文件
        // 2) 并发解析（registry.by_language(file.lang).parse_file）
        // 3) 抽取函数、调用、片段
        // 4) 聚合为 CodeGraph 与 snippets
        // 5) 导出到目标目录
        Ok(AnalyzeResult { /* ... */ })
    }
}
```

### 输出目录结构（建议）
- `target/codegraph/`
  - `graph.json`
  - `graph.mmd`
  - `graph.dot`
  - `snippets.json`
  - `errors.json`（可选）

### 后续工作
- 引入增量构建与缓存键（文件哈希/修改时间）；
- 代码导航 API：给定 `function_id` 返回片段/源码范围；
- 语言解析精度持续改进（泛型、模板、宏等）。
