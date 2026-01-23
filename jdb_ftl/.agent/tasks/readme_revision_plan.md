# Implementation Plan - README & Code Review & Revision

This plan outlines the systematic review and revision of the JDB-FTL documentation (Chinese and English) to ensure 100% consistency with the current Rust source code.

## 1. Governance & Quality Standards
- **Accuracy**: Every constant (e.g., `GROUP_SIZE`), data structure size (e.g., `12B Seg`), and complexity claim ($O(1)$, $O(N+M)$) must match the code.
- **Bilingual Consistency**: Chinese and English documents must be semantically synchronized.
- **Modernity**: Ensure descriptions reflect recent optimizations (e.g., `Direct Mode`, `Incremental Reuse`).

## 2. Document Review & Revision Tasks

### Phase 1: Codec & PGM Algorithm (`readme/*/codec.md`)
- **Action**: Fix `GROUP_SIZE` (1024 -> 4096).
- **Action**: Verify `Seg` bit-packing layout (W0, W1, W2) against `src/ftl/seg/mod.rs`.
- **Action**: Align encoding/decoding snippets with `encode.rs` and `decode_group.rs`.
- **Action**: Clarify "128KB limitation" vs. `bit_offset` (20 bits).

### Phase 2: Bitmap & Sparsity (`readme/*/bitmap.md`)
- **Action**: Confirm `SHARD_COUNT = 1024`.
- **Action**: Verify memory calculation logic for `RoaringBitmap` containers.
- **Action**: Detail how `u64::MAX` triggers segment breaks in `optimal.rs`.

### Phase 3: Hardware Optimization & Structures (`readme/*/structures.md`)
- **Action**: Audit memory alignment claims. `Seg` is `align(2)`, check if it's practically `align(4)` in storage.
- **Action**: Verify 128-bit unaligned peek description vs. `read_bits` implementation.

### Phase 4: Flux & Incremental Logic (`readme/*/flush.md`)
- **Action**: Rename `rfind` search to "forward synchronization search" to match `encode.rs`.
- **Action**: Ensure the double-pointer scan $O(N+M)$ description is technically precise.

### Phase 5: Overall Package (`architecture.md`, `tuning.md`, `bench.md`)
- **Action**: General sanity check for architecture diagrams.
- **Action**: Ensure benchmark placeholders are ready for data injection.

## 3. Review Checklist (Per Document)
1. [ ] Constants match `build.rs` or `mod.rs`.
2. [ ] Variable names match the codebase.
3. [ ] Code snippets in MD are up-to-date.
4. [ ] English translation is accurate and professional.
5. [ ] Performance claims are supported by code logic.
