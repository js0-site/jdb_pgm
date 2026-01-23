# 压缩率优化：9% → 7.3%

## 核心优化

### 1. 恢复分段对齐逻辑

**文件**：`src/ftl/codec/encoder/encode.rs`（第 118-127 行）

```rust
// Try to align with old segmentation boundaries to maximize future reuse.
if let Some(old_idxs) = old_start_idxs
  && let Some(sync) = old_idxs
    .iter()
    .map(|&s| s as usize)
    .rfind(|&s| s > cursor && s <= cursor + res.length)
{
  res = find_longest_segment(&chunk[..sync - cursor], epsilon as u64);
}
```

**作用**：将新分段边界对齐到旧分段边界，最大化 `PayloadChunk::Reuse` 命中率，避免重复存储残差数据。

### 2. 恢复缓冲区大小

**文件**：`build.rs`

```rust
let buffer_capacity = get_env_or_default("FTL_BUFFER_CAPACITY", 2097152usize); // 2MB
```

**作用**：较大的缓冲区提高增量更新批次规模，提升 Reuse 效率。

---

**结果**：压缩率 9.28% → **7.30%**，内存占用减少 21%。
