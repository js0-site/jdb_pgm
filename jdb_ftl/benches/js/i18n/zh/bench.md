## 性能基准测试报告

本报告旨在验证 JDB-FTL 的核心思想：**PGM INDEX + 自适应位宽残差补偿 = 无损压缩**。我们通过真实的 I/O 轨迹 ([MSRC-trace-003.tar.gz](https://trace.camelab.org/Download.html))，对比了该方案与**原生 Rust 内存数组（无边界检查）** (Baseline) 在性能与空间上的表现。


### 详细对比数据

<%= _.table %>

> [!NOTE]
> 若使用**键值映射**存储这些映射，其实际内存占用约为 **<%= _.stats.hashMapMemMB %> MB**。
> 相比之下，JDB-FTL 仅需 **<%= _.stats.ftlMemMB %> MB**，针对该映射方式的压缩率达 **<%= _.stats.hashMapCompressionRatio %>%**。

#### 技术原理：PGM 引导下的无损重建
JDB-FTL 的高压缩率源于其核心公式，通过**漂移基准线**消除符号位，并将残差压缩至极低位宽：

$$\text{物理地址} = \text{基准地址} + (\text{索引} \times \text{斜率} \gg 24) + \text{残差}$$

其中：
- $\text{基准地址}$: 经过修正的起始物理地址 (修正量 $\text{min\_diff}$ 确保 $\text{残差} \ge 0$)
- $\text{斜率}$: 24 位定点精密斜率
- $\text{残差}$: 变长比特压缩后的非负预测残差

### 核心指标分析

#### 1. 性能损耗分析
测试数据显示瞬时吞吐量有 **<%= _.stats.minThroughputDrop %>% - <%= _.stats.maxThroughputDrop %>%** 的下降，但端到端总耗时增幅约为 **<%= _.stats.overheadRatio %>%**。这种差异主要源于：

*   **计算开销在实际应用中被稀释**：微基准测试主要评估 CPU 执行效率，未包含 I/O 等待。在包含文件系统和介质访问的实际链路中，FTL 的额外计算延迟（小于 100ns）远低于 Flash 物理延迟（微秒级）。
*   **对 I/O 链路影响极小**：企业级 NVMe SSD 的 4K 随机读物理延迟通常在 80μs - 100μs 之间。JDB-FTL 的查找延迟（P99 <%= _.stats.p99GetNs %>ns）仅为 <%= _.stats.p99GetUs %>μs，占总 I/O 耗时的比例不足 0.1%。这意味着算法引入的计算开销在 Flash 介质访问响应时间面前几乎可以忽略。

#### 2. 存储容量与内存占用分析
以 15.36TB (16TB) 规格的 SSD 为例，映射管理的内存分配情况如下：

*   **全量映射表开销**：4KB 粒度的页级映射若不采用压缩，理论上需要 **32GB 内存** ($16\text{TB} / 4\text{KB} \times 8\text{字节}$)。
*   **当前工业界方案**：受限于成本和功耗，主流 SSD 通常配置 8GB 到 16GB 的 DRAM（如 Samsung PM1733 配置 16GB）。由于物理内存不足以存放全量表，系统通常采用分级缓存策略，当发生缓存未命中时会产生约 50μs 的加载延迟。
*   **JDB-FTL 的优化效果**：按目前的 **<%= _.stats.compressionRatio %>%** 压缩率计算，16TB 容量的全量表驻留仅需约 **<%= _.stats.memEstimate16TB %>GB** 内存。这使得映射表能够完全常驻内存，避免了缓存置换带来的性能抖动，同时降低了对高性能 DRAM 的容量需求。

### 测试环境
<%= _.env %>