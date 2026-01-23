# Documentation Standard & Workflow (文档规范与工作流)

为了保证项目文档的高质量、准确性和一致性，请遵循以下规范。

## 1. Zero Tolerance for Desynchronization (双语强同步)
*   **实时同步**：任何代码逻辑的变更（Code Change），必须**同时**更新中文 (`zh`) 和英文 (`en`) 文档。禁止“先更中文，以后再说”的行为。
*   **结构一致**：中英文文档的章节结构、段落顺序必须保持严格的 1:1 对应。
*   **Code Review 驱动**：在更新文档前，必须仔细阅读最新的代码实现。文档是代码的映射，**Code Review 是写文档的前置条件**。

## 2. Localization Standards (本地化/翻译标准)
### 中文文档 (`zh`)
*   **纯净标题**：标题中**严禁**出现英文副标题。
    *   ❌ 错误：`## 3.2.1 智能扩张逻辑 (Simplified)`
    *   ✅ 正确：`## 3.2.1 智能扩张逻辑`
*   **Mermaid 中文化**：流程图（Mermaid）中的所有节点标签、连线说明必须翻译为**中文**。
    *   ❌ `A[Snapshot] --> B{Idle?}`
    *   ✅ `A[快照] --> B{空闲?}`

### 英文文档 (`en`)
*   **Mermaid 英文化**：流程图中的所有文本必须为**英文**。
*   **地道表达**：确保术语与代码中的变量命名（如 `dirty_map`, `reuse`）有清晰的对应关系。

## 3. Architecture & Refactoring Awareness (架构感知)
*   **文件路径更新**：如果代码发生了拆分或重构（例如 `pgm.rs` 拆分为 `pgm/mod.rs`），必须检查文档中提及的文件路径是否已失效，并及时修正。
*   **逻辑变更响应**：如果核心算法流程变更（例如从“复杂探测”改为“简单复用”），文档中的 Mermaid 流程图和文字描述必须重写，不能只改细枝末节。

## 4. Validation Workflow (验证工作流)
每次修改 Mermaid 流程图后，**必须**运行验证命令，确保语法正确且能渲染。

```bash
# 验证中文文档
mmdc -i readme/zh/target_file.md -o /tmp/check_zh.svg

# 验证英文文档
mmdc -i readme/en/target_file.md -o /tmp/check_en.svg
```

## Summary Checklist
- [ ] Code Logic Reviewed? (代码逻辑已确认？)
- [ ] ZH & EN Docs Updated? (双语文档已更新？)
- [ ] ZH Mermaid labels are Chinese? (中文图是中文标签？)
- [ ] EN Mermaid labels are English? (英文图是英文标签？)
- [ ] No English subtitles in ZH headers? (中文标题无英文？)
- [ ] `mmdc` Validation Passed? (绘图验证通过？)
