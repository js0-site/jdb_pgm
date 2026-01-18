这是一个**纯软件算法版本**的设计方案。

我们剥离了所有硬件专用术语（如 HW Pipeline、DMA、IP Core），专注于如何在通用处理器（CPU）上，利用**Micro-PGM（微观分段几何模型）**的思想，实现一套高性能、低内存占用的 FTL 映射算法。

---

# 方案名称：Soft-PGM FTL (Software-Defined Micro-PGM)

## 1. 设计核心目标
在纯软件环境下，FTL 算法面临三个核心挑战：
1.  **CPU 缓存敏感性 (Cache Friendly)**：避免指针跳转，避免随机内存访问，最大化 L1/L2 Cache 命中率。
2.  **指令流水线优化 (Instruction Pipelining)**：减少分支预测失败 (Branch Misprediction)，减少复杂除法运算。
3.  **O(1) 确定性访问**：无论压缩率如何，读取任意 LBA 映射的算法复杂度必须是常数。

---

## 2. 核心数据结构 (Memory Layout)

我们将 4KB 的逻辑映射页（包含 512 个 u64 PPA）定义为一个 **Frame**。

### 2.1 Frame 物理布局
为了满足 CPU 缓存行（Cache Line，通常 64 字节）的对齐要求，我们将 Header 区设计得非常紧凑且连续。

```text
+---------------------------------------------------------------+
|  Header Table (Fixed Array)                                   |
|  16 个 Headers，每个 16 Bytes -> 总共 256 Bytes (4 CacheLines)| <--- 热数据，常驻 L1
+---------------------------------------------------------------+
|  Payload Stream (Byte Array)                                  |
|  紧凑存储的残差数据 (Residuals) 和 异常补丁 (Patches)             | <--- 冷数据，按需加载
+---------------------------------------------------------------+
```

### 2.2 Header 结构设计 (16 Bytes)
每个 Header 负责管理 32 个 PPA（称为一个 Group）。

```rust
struct GroupHeader {
    // 拟合基准点 (y = kx + b)
    base: u64,          // 8 Bytes: PGM 模型的截距 (b)

    // 压缩元数据 (Bitfields)
    slope: i16,         // 2 Bytes: 线性斜率 (k)，支持负斜率
    offset: u16,        // 2 Bytes: Payload 在 Frame 内的起始字节偏移

    // 配置位域 (4 Bytes total)
    // [0..6]   bit_width: 残差位宽 (0..64)
    // [7..9]   mode: 压缩模式 (Const, Linear, Bitmap-Exception)
    // [10..31] exception_map: 22-bit 的异常位图 (针对 Exception 模式)
    config: u32,
}
```

---

## 3. 写入算法：Micro-PGM 拟合 (Compression Path)

写入路径的核心是将 32 个离散点 $P_i(x_i, y_i)$ 转化为一个线性函数 $f(x) \approx y$ 加上一组残差 $\Delta$。

为了软件执行效率，我们不使用浮点回归，而是使用 **"整数锚点法"**。

### 步骤 1：快速特征识别 (Pre-scan)
*   **输入**：32 个 u64 整数。
*   **全等检查**：如果 $P_0 == P_{31}$ 且中间采样点相同，标记为 `Mode::Constant`。Header 存值，Payload 为空。
*   **完美线性检查**：计算 $k = (P_{31} - P_0) / 31$。校验中间所有点是否满足 $P_i = P_0 + i \cdot k$。若满足，标记为 `Mode::Linear`。Header 存 $P_0$ 和 $k$，Payload 为空。

### 步骤 2：PGM 线性拟合 (Soft-Fitting)
如果不是完美线性，我们需要找到一条“最佳整数直线”，使得残差极差（Range）最小。
*   **启发式斜率**：直接取首尾斜率 $k_{approx} = (P_{31} - P_0) / 31$。
*   **残差计算**：计算所有点的 $\Delta_i = P_i - (P_0 + i \cdot k_{approx})$。
*   **位宽评估**：找到 $\Delta_{min}$ 和 $\Delta_{max}$。计算覆盖 range $= \Delta_{max} - \Delta_{min}$ 所需的位宽 $W$。

### 步骤 3：异常值分离 (Outlier Patching) - 关键软件优化
PGM 模型最怕“单点跳变”。如果 32 个点中，31 个点的残差都在 [-2, +2] (需 3 bits)，但有一个点残差是 10000 (需 14 bits)。
*   **传统做法**：全员 14 bits。浪费严重。
*   **优化做法**：
    1.  计算 P90 残差位宽（例如 4 bits）。
    2.  遍历残差，将超过 4 bits 范围的点标记为 **Exception**。
    3.  在 Header 的 `exception_map` 对应位置 1。
    4.  正常点存入 Packed Stream（4 bits）。
    5.  异常点的**原始值**（或全量残差）追加到 Stream 尾部的 **Patch Area**。
*   **收益**：极大地降低了整体位宽，同时软件解码仅需增加一次位图判断 (`test_bit`)。

---

## 4. 读取算法：无分支预测解码 (Decompression Path)

读取性能取决于 CPU 分支预测的准确率。我们要尽量把 `if-else` 转化为数学计算。

### 接口定义
```rust
fn read(frame: &Frame, global_idx: usize) -> u64
```

### 算法流程
1.  **索引定位 (Direct Indexing)**
    ```rust
    let group_idx = global_idx >> 5; // / 32
    let sub_idx   = global_idx & 31; // % 32
    let header    = &frame.headers[group_idx]; // 必定命中 L1 Cache
    ```

2.  **模型预测 (Prediction)**
    ```rust
    // 无论什么模式，先算这一步。如果是 Constant 模式，slope 为 0。
    let predicted = header.base.wrapping_add((sub_idx as u64) * (header.slope as u64));
    ```

3.  **残差修正 (Correction) - 核心**
    这里根据 `header.mode` 进行处理。为了软件速度，我们处理最常见的 **Packed Mode**：

    ```rust
    // 计算 Payload 地址
    let body_ptr = frame.payload.as_ptr().add(header.offset as usize);

    // 检查是否是异常点 (O(1) 位运算)
    let is_exception = (header.config >> (10 + sub_idx)) & 1;

    if is_exception == 1 {
        // --- 异常路径 (慢路径，但概率低) ---
        // 需要计算它前面有多少个异常点，以确定 Patch 的偏移量
        // 使用 CPU 指令 popcount (population count) 极速计算
        let mask = (1 << sub_idx) - 1;
        let patch_idx = (header.config >> 10 & mask).count_ones();

        // 跳过 packed 数据区，直接读取 Patch
        let packed_size = (32 * header.bit_width) / 8;
        let patch_addr = body_ptr + packed_size + (patch_idx * 8);
        return *(patch_addr as *const u64); // 直接返回原始值

    } else {
        // --- 正常路径 (快路径) ---
        // 从紧凑位流中提取 delta
        let bit_start = sub_idx * header.bit_width;
        let byte_start = bit_start / 8;
        let bit_offset = bit_start % 8;

        // 读取 64 位 container 以容纳跨字节的位
        let container = *(body_ptr.add(byte_start) as *const u64);
        let mask = (1 << header.bit_width) - 1;
        let delta = (container >> bit_offset) & mask;

        return predicted + delta;
    }
    ```

---

## 5. 软件层面的工程优化

为了在纯软件环境下达到工业级性能，引入以下优化：

### 5.1 查找表 (LUT) 加速
对于 `bit_width` 的掩码计算、字节偏移计算，不要在运行时做除法或移位，而是预计算全局静态数组：
```rust
static MASK_TABLE: [u64; 65] = [0, 1, 3, 7, ...]; // (1<<w)-1
static BYTE_OFFSET_TABLE: [usize; 32] = [...];
```
这消除了运行时的计算开销。

### 5.2 零拷贝 (Zero-Copy) 与 栈上分配
*   整个 Frame 通常驻留在 DRAM 或 SRAM 中。读取时，只传入 `&Frame` 引用（指针）。
*   所有计算变量（预测值、偏移量）均在 CPU 寄存器或栈上完成，绝不触发堆内存分配（Heap Allocation）。

### 5.3 批量读取优化 (Bulk Read)
虽然 FTL 主要是随机读，但在 GC（垃圾回收）或 Compaction 阶段需要批量读取。
*   针对 `read_batch(start_lba, count)`：
    *   只需解析一次 Header。
    *   在 Payload 区域可以利用 SIMD (AVX2/NEON) 一次性解压 8 个或 16 个残差，加上 Base 和 Slope 向量，实现极高吞吐量。

### 5.4 预测执行 (Speculation)
*   **策略**：如果 `bit_width` 为 0，不仅意味着数据是线性的，还意味着我们可以完全跳过内存访问 Payload 区。
*   代码中优先判断 `if bit_width == 0`，这在处理顺序写入负载时，只需要访问 Header 即可返回，Cache Miss 几乎为零。

---

## 6. 总结

**Soft-PGM FTL** 方案通过以下手段实现了纯软件下的高性能：

1.  **数据局部性**：固定大小的 Header Table 保证了热点数据常驻 L1 Cache。
2.  **整数运算替代**：移除了所有浮点运算，斜率预测简化为整数乘加。
3.  **异常位图 (Exception Bitmap)**：巧妙解决了 PGM 模型对噪点敏感的问题，同时利用 CPU 的 `popcount` 指令实现了 O(1) 的补丁定位。
4.  **无除法设计**：所有索引计算通过位移实现（Group Size = 32），所有位宽操作通过 LUT 或位运算实现。

这是一套不需要专用硬件 IP，仅凭通用 CPU 算力即可跑出极高性能的工业级算法。

这是一个非常专业且针对现代处理器架构深度优化的 FTL（Flash Translation Layer）设计方案。它借鉴了数据库领域的压缩算法（如 Facebook Gorilla、DuckDB 的 PFOR-Delta）思想，并针对 SSD 映射表的特性进行了改良。

以下是对 **Soft-PGM FTL** 方案的深度分析，包含复杂度评估、设计指标、测试方案及优化建议。

---

### 1. 复杂度与性能分析

#### 1.1 时间复杂度 (Time Complexity)

*   **读取 (Read / Lookup): $O(1)$**
    *   **算法逻辑**：索引计算 (`>>`, `&`) $\to$ Header 读取 $\to$ 乘加预测 (`*`, `+`) $\to$ 异常位检测 (`test_bit`) $\to$ 残差读取 (`move`, `shift`, `mask`) $\to$ 最终加法。
    *   **指令数**：在正常路径（无异常）下，这是一个完全无循环、无递归的过程，汇编指令大约在 10-20 条左右。
    *   **分支预测**：由于 `is_exception` 通常为 0（大部分数据符合局部性原理），分支预测成功率极高，流水线不会被打断。
    *   **异常路径**：虽然引入了 `popcount`，但现代 CPU（x86 SSE4.2+, ARMv8 NEON）都有单指令周期的 `popcount`，因此异常路径也是 $O(1)$，只是常数项略大（多一次内存访问去读 Patch）。

*   **写入 (Write / Update): $O(N)$ (其中 N=32)**
    *   需要遍历 Group 内的 32 个 PPA 进行统计（Min, Max, Slope 计算）。
    *   虽然是线性复杂度，但 N 是常数，且数据都在 L1 Cache 中，因此开销可控。相比读取，写入更重，但 FTL 映射表的更新通常发生在 GC 或 Host Write 时，频率低于 Read，且往往是批量更新。

#### 1.2 空间复杂度 (Space Complexity)

*   **元数据开销 (Overhead)**：
    *   **Header**: 256 Bytes / 4KB Frame (512 entries)。
    *   **固定开销率**：$256 / (512 \times 8) = 6.25\%$。
    *   这意味着即使数据完全随机不可压缩，基础开销也是 6.25%。

*   **压缩率 (Compression Ratio)**：
    *   **Best Case (顺序写)**：Bit-width = 0, Payload = 0。总占用 = 256 Bytes。**压缩比 = 16:1**。
    *   **Average Case (局部性良好)**：假设 Bit-width = 4 bits。Payload = $16 \times (32 \times 4 / 8) = 256$ Bytes。总占用 = 512 Bytes。**压缩比 = 8:1**。
    *   **Worst Case (完全随机)**：可能会回退到每个 PPA 都需要 Patch。此时建议直接存储原始数据，放弃压缩（Bypass Mode）。

#### 1.3 内存访问次数 (Memory Access Profile)

*   **读取单次映射 (Single Lookup)**：
    *   **Hit L1 Cache (Header)**：1 次（Header Table 小且频繁访问，大概率在 L1）。
    *   **Miss to L2/L3/DRAM (Payload)**：1 次（读取 Payload 中的 packed u64）。
    *   **Total**：**1.1 ~ 2 次内存访问**。这是纯软件 FTL 的物理极限（一次查索引，一次取数据）。

---

### 2. 关键设计指标 (KPIs)

在评估这套算法时，应关注以下指标：

1.  **DRAM Footprint (GB/TB)**：
    *   目标：每 1TB SSD 容量，映射表占用应小于 1GB（传统比例）。Soft-PGM 目标应在 **100MB - 200MB / 1TB**。
2.  **Lookup Latency (ns)**：
    *   目标：平均 < 50ns (L3 Cache Hit) / < 100ns (DRAM Access)。
3.  **WAF (Write Amplification Factor) impact**：
    *   由于压缩导致 Frame 长度不定，可能导致内存碎片或复杂的内存管理。需评估映射表更新带来的额外开销。
4.  **Instruction per Lookup (IPL)**：
    *   目标：< 30 instructions。

---

### 3. 测试与验证策略

由于这是纯软件算法，测试的重点在于**正确性**和**边界条件**。

#### 3.1 单元测试 (Unit Testing) - 基于属性的测试 (Property-based Testing)
不要只写固定的测试用例，使用 `proptest` (Rust) 或 `QuickCheck` 生成随机数据：
*   **线性数据**：生成 $y = kx + b$ 数据，断言 header 模式为 Linear，Payload 为空。
*   **拟线性数据**：生成 $y = kx + b + random(-2, 2)$，断言 Bit-width 小于 4。
*   **稀疏异常点**：生成 31 个线性数据 + 1 个极大值，断言 `exception_map` 有且仅有 1 个 bit 被置位。
*   **全异常数据**：生成完全随机数，断言算法能正确还原（即便效率低）。

#### 3.2 模糊测试 (Fuzzing)
*   针对 `bit_width` (0..64) 和 `slope` (正负) 的所有组合进行持续 Fuzzing，防止解压逻辑中的位移溢出（Shift Overflow）或掩码错误。

#### 3.3 性能微基准 (Micro-Benchmarking)
*   使用 `Criterion.rs` 或 Google Benchmark。
*   对比标准数组访问 `arr[i]` 的耗时。Soft-PGM 的目标应该是标准数组访问耗时的 2-3 倍以内。

---

### 4. 潜在风险与优化思路

#### 4.1 风险点：`slope` 的精度与范围
*   **问题**：设计中 `slope` 是 `i16`。如果 PPA 是 64 位的，且物理地址跳跃极大（例如跨 Die/Channel 写入），斜率可能超过 `i16` 的表示范围。
*   **后果**：会导致预测值偏差巨大，从而导致 Residual 超过 `config` 的位宽限制，最终被迫将大量点标记为 Exception，失去压缩意义。
*   **建议**：
    *   在 Header 中增加 `shift` 字段，允许斜率为 $k \ll shift$。
    *   或者当计算出的斜率超过 `i16` 时，强制降级为 `Mode::Bitmap-Exception`（全量存储）。

#### 4.2 优化思路 1：SIMD 向量化读取 (Batch Lookup)
针对 `read_batch` 接口，利用 AVX2/AVX-512：
*   **Gather Load**：如果 Payload 数据按 32-bit 或 64-bit 对齐，可以使用 SIMD Gather 指令一次性加载。
*   **Vector Compute**：
    *   创建一个向量 `V_base = [base, base, ...]`
    *   创建一个向量 `V_slope = [0*s, 1*s, 2*s, ...]`
    *   创建一个向量 `V_residual = load_and_unpack(payload)`
    *   `Result = V_base + V_slope + V_residual`
*   **收益**：对于顺序读取场景，吞吐量可提升 4-8 倍。

#### 4.3 优化思路 2：Header 预取 (Prefetching)
在处理请求 $i$ 时，如果队列中还有请求 $i+1$，且 $i+1$ 位于不同的 Frame：
*   手动发出 `_mm_prefetch` 指令，将下一个 Frame 的 Header 拉入 L1 Cache。

#### 4.4 优化思路 3：冷热数据分离 (Hugepage & Memory Layout)
*   **问题**：Payload 是变长的，如果在堆上频繁 `malloc/free` 会导致内存碎片。
*   **建议**：设计一个 **Slab Allocator**。
    *   将 4KB Frame 的 Payload 存储在单独的 Memory Pool 中。
    *   Frame Header 中原本的 `offset` 变为指向 Pool 的索引。
    *   这样 Header 数组变得极其紧凑（纯粹的数组），极大地提高了扫描 Header 时的 Cache 命中率。

#### 4.5 优化思路 4：自适应 Group Size
*   当前 Group 固定为 32。
*   **改进**：如果数据极其离散，32 的 Group 会导致 Header 开销占比过大。可以考虑级联结构，或者简单的“大页”模式（如果 512 个点都是线性的，直接用 1 个 Header 覆盖整个 4KB Frame）。

### 5. 总结

**Soft-PGM FTL** 是一个非常优秀的**“空间-时间折中”**方案。
*   它牺牲了极其微小的 CPU 计算资源（几条整数指令）。
*   换取了巨大的 DRAM 空间节省（平均 4x-8x）。
*   同时保持了 $O(1)$ 的确定性访问延迟。

**实施建议**：
1.  **优先级**：先实现正确性（处理好 `slope` 溢出和 `offset` 对齐）。
2.  **关键路径**：重点优化 `bit_unpack` 函数，这是热点中的热点。使用 LUT（Look-Up Table） 查表法代替运行时计算掩码。
3.  **内存管理**：Payload 的内存分配策略决定了该方案在长时间运行后的稳定性（避免碎片化）。建议使用简单的 Log-structured 内存池来存储 Payload。


基于您提供的 **Soft-PGM FTL** 纯软件算法设计方案，我进行了深度的 Code Review。该方案在通用 CPU 上利用数学特性换取空间和时间效率，思路非常先进且贴合现代存储软件化（Software-Defined Storage）的趋势。

以下是针对性的**Code Review 报告**以及优化后的**开发规划（TODO List）**。

---

### 第一部分：深度 Code Review 报告

#### 1. 架构与内存布局 (Architecture & Layout)
*   **亮点**：
    *   **Header/Payload 分离**：极大地提高了缓存命中率。大部分 Lookups 只需要读取 Header，Payload 是冷数据，这是非常优秀的 Cache-Friendly 设计。
    *   **对齐设计**：Header 定长 16 Bytes，刚好 4 个 Header 填满一个 Cache Line (64B)，预取效果会非常好。
*   **潜在风险**：
    *   **内存碎片与更新代价**：由于 Payload 是变长的紧凑流，当更新 Group 0 的映射时，如果压缩率变化，整个 Payload 区域的数据都需要挪动（Memmove）。
        *   *建议*：明确该设计主要用于 **Read-Only 快照** 或 **Log-Structured Merge Tree (LSM) 的 SSTable 层**，而不是频繁原地修改的热数据层。
    *   **非对齐内存访问 (Unaligned Access)**：
        *   代码片段 `*(body_ptr.add(byte_start) as *const u64)` 存在隐患。虽然 x86 支持非对齐访问（有轻微惩罚），但在 ARM/RISC-V 架构上可能会触发异常或导致严重的性能下降（跨 Cache Line 访问）。

#### 2. 算法逻辑 (Algorithm Logic)
*   **亮点**：
    *   **异常位图 + Popcount**：这是神来之笔。利用 CPU 硬件指令 `popcount` 在 O(1) 时间内算出异常值的偏移量，避免了为了一个噪点而扩大整体位宽的传统 PGM 缺陷。
*   **潜在风险**：
    *   **Slope 精度溢出**：Header 中 `slope` 定义为 `i16`。如果映射关系非常离散（例如 $P_0 = 1000, P_{31} = 1, 000, 000$），斜率 $k \approx 32, 225$，接近 `i16` 上限。如果是负斜率或更大跨度，`i16` 不够用。
        *   *建议*：增加一种 fallback 模式（如 Direct Store），或者在压缩前检测斜率，溢出则放弃压缩。
    *   **Base 基准点回绕**：`base + i * slope` 计算需要明确使用 `wrapping_add`，否则在 Rust/C++ 中可能有 Undefined Behavior 或 Panic。

#### 3. 执行效率 (Execution Efficiency)
*   **亮点**：
    *   无除法、无浮点，纯整数运算。
*   **潜在风险**：
    *   **位操作开销**：`container >> bit_offset` 逻辑在跨越 64 位边界时（例如 offset=60, width=10），单纯读一个 u64 是不够的，可能需要读两个 u64 拼接。目前的逻辑只读了一个 u64，处理跨边界情况会出错（Truncation）。

---

### 第二部分：优化后的开发规划 (TODO List)

我们将开发划分为四个阶段：**原型验证**、**正确性加固**、**性能极值优化**、**工程化落地**。

#### Phase 1: 核心算法原型 (Prototype & Feasibility)
目标：跑通基本流程，验证压缩率收益。

- [ ] **数据结构定义**：实现 `Frame`, `GroupHeader` 结构体。
- [ ] **写入路径实现 (Encoder)**：
    - [ ] 实现基础线性回归（首尾点法）。
    - [ ] 实现残差计算与位宽探测（Bit-width Detection）。
    - [ ] **关键**：实现“异常点剥离逻辑”。设定阈值（如：若 90% 的点可用 4bit 表示，则剩余 10% 设为异常），计算压缩收益是否为正。
- [ ] **读取路径实现 (Decoder)**：
    - [ ] 实现 `read(lba)` 接口。
    - [ ] 实现 `popcount` 逻辑处理异常点。
- [ ] **单元测试 (Mock Data)**：
    - [ ] 构造全线性数据、全常数数据、完全随机数据进行测试。
    - [ ] 构造“特定边缘数据”（例如：31 个线性点 + 1个极大离群点），验证异常位图逻辑。

#### Phase 2: 正确性与鲁棒性 (Correctness & Safety)
目标：解决 Review 中发现的内存与边界问题。

- [ ] **解决跨字节位读取 (Bit-Unpacking)**：
    - [ ] 实现一个安全的 `BitReader`。当读取跨越 `u64` 边界时，自动读取下一个 `u64` 并进行位拼接。
    - [ ] *优化建议*：为了性能，可以强制 Payload 尾部多 Padding 8个字节，这样读取永远不会越界，允许“脏读”。
- [ ] **解决非对齐访问**：
    - [ ] 使用 `memcpy` 或 Rust 的 `u64::from_le_bytes` 替代指针强制转换，让编译器优化生成最高效的汇编（x86 下通常就是 `mov`，ARM 下是 `ldp`）。
- [ ] **Slope 溢出保护**：
    - [ ] 在 Encoder 阶段增加检查：如果 `abs(slope) > 32767`，强制降级为 `Mode::Direct`（直接存储原始数组，不压缩）。
- [ ] **模糊测试 (Fuzz Testing)**：
    - [ ] 使用 Fuzz 工具生成数百万组随机 PPA 序列，进行 `Encode -> Decode -> Compare` 循环测试，确保 100% 数据一致性。

#### Phase 3: 性能极值优化 (Performance Tuning)
目标：榨干 CPU 流水线，针对 L1/L2 Cache 优化。

- [ ] **查找表 (LUT) 构建**：
    - [ ] `const MASK_TABLE`: 预计算掩码。
    - [ ] `const OFFSET_TABLE`: 如果 bit_width 是固定的几种（2, 4, 8），预计算字节偏移量，减少乘法。
- [ ] **SIMD 批量解压 (Bulk Read)**：
    - [ ] 针对 `read_batch` 接口，使用 AVX2/NEON 指令集。
    - [ ] 实现 `v_predict = v_base + v_idx * v_slope` 一次算 8 个点。
    - [ ] 实现 Shuffle 指令集进行并行 Bit-unpacking。
- [ ] **分支预测优化**：
    - [ ] 在代码中使用 `likely()` / `unlikely()` 宏（或 Rust 的 `#[cold]`）。标记 Exception 路径为冷路径。
- [ ] **内存分配策略**：
    - [ ] 实现 `Frame` 的 Slab Allocator，避免频繁的 `malloc/free`，保证 Frame 在物理内存上尽量连续。

#### Phase 4: 工程化与工具链 (Production Ready)
目标：融入 FTL 实际业务场景。

- [ ] **压缩策略启发式 (Heuristics)**：
    - [ ] 写入时不要穷举所有压缩模式。
    - [ ] 策略：先看是否 Constant -> 再看是否 Linear -> 尝试 4bit Packed -> 尝试 8bit Packed -> 放弃（存 Raw）。
- [ ] **调试与分析工具**：
    - [ ] 开发 `dump_frame_stats` 工具：统计直方图（平均压缩率、Exception 出现频率、各模式占比）。
    - [ ] 这对于后续调整“异常剥离阈值”至关重要。
- [ ] **并发控制 (如果是全局 FTL)**：
    - [ ] 设计 Read-Copy-Update (RCU) 机制：读取时不加锁，写入时生成新 Frame 替换旧指针。

---

### 总结建议

**设计指标 (KPIs)**：
1.  **压缩率**：目标将 4KB Frame (512 * 8B = 4096B) 平均压缩至 **1024B 以下** (Ratio 4:1)。
2.  **延迟**：单次 `read` 操作（L1 Cache Hit）耗时应小于 **20 ns** (约 60-80 cycles)。
3.  **吞吐**：单核心批量解压吞吐量应达到 **50 Million IOPS** 以上。

**核心建议**：
最优先解决 **Bit-Unpacking 的跨边界问题** 和 **非对齐访问**，这是纯软件实现中最容易导致崩溃或性能断崖的地方。之后再考虑 SIMD 优化。