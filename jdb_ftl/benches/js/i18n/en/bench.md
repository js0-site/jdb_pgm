## Benchmark Report

This report aims to verify the core concept of JDB-FTL: **PGM INDEX + Adaptive Bit-Width Residual Compensation = Lossless Compression**. Using real-world I/O traces ([MSRC-trace-003.tar.gz](https://trace.camelab.org/Download.html)), we compare the performance and space efficiency of this scheme against a **Native Rust InMemory Array (Unchecked)** (Baseline).

### Detailed Comparison Data

<%= _.table %>

> [!NOTE]
> If a  **Key-Value Map** were used to store these mappings, the actual memory usage would be approximately **<%= _.stats.hashMapMemMB %> MB**.
> In contrast, JDB-FTL requires only **<%= _.stats.ftlMemMB %> MB**, achieving a compression ratio of **<%= _.stats.hashMapCompressionRatio %>%** against this mapping method.

#### Technical Principle: Lossless Reconstruction Guided by PGM
The high compression rate of JDB-FTL stems from its core formula, which uses a **Drifting Baseline** to eliminate sign bits and packs residuals into an extremely low bit-width:

$$\text{PBA} = \text{Base} + (\text{Index} \times \text{Slope} \gg 24) + \text{Residual}$$

Where:
- $\text{Base}$: Corrected starting physical address (Correction amount $\text{min\_diff}$ ensures $\text{Residual} \ge 0$)
- $\text{Slope}$: 24-bit fixed-point precision slope
- $\text{Residual}$: Non-negative prediction residual after variable-length bit compression

### Core Metrics Analysis

#### 1. Performance Loss Analysis
Test data shows a drop in instantaneous throughput of **<%= _.stats.minThroughputDrop %>% - <%= _.stats.maxThroughputDrop %>%**, but the total end-to-end time increase is approximately **<%= _.stats.overheadRatio %>%**. This discrepancy is primarily due to:

*   **Computational Overhead Diluted in Practice**: Micro-benchmarks mainly evaluate CPU execution efficiency and do not include I/O wait times. In a real-world link involving file systems and media access, the extra computational latency of FTL (less than 100ns) is far lower than the physical latency of Flash (microsecond level).
*   **Minimal Impact on I/O Path**: The 4K random read physical latency of enterprise NVMe SSDs is typically between 80μs - 100μs. JDB-FTL's lookup latency (P99 <%= _.stats.p99GetNs %>ns) is only <%= _.stats.p99GetUs %>μs, accounting for less than 0.1% of total I/O time. This means the computational overhead introduced by the algorithm is almost negligible compared to the response time of Flash media access.

#### 2. Storage Capacity and Memory Usage Analysis
Taking a 15.36TB (16TB) SSD as an example, the memory allocation for mapping management is as follows:

*   **Full Mapping Table Overhead**: If 4KB granular page-level mapping is used without compression, it theoretically requires **32GB of RAM** ($16\text{TB} / 4\text{KB} \times 8\text{Bytes}$).
*   **Current Industry Solutions**: Constrained by cost and power consumption, mainstream SSDs are typically configured with 8GB to 16GB of DRAM (e.g., Samsung PM1733 has 16GB). Since physical memory is insufficient to store the full table, systems usually adopt a hierarchical caching strategy, incurring a loading latency of about 50μs when a cache miss occurs.
*   **Optimization Effect of JDB-FTL**: Calculated at the current compression ratio of **<%= _.stats.compressionRatio %>%**, the full table residency for 16TB capacity only requires about **<%= _.stats.memEstimate16TB %>GB** of memory. This allows the mapping table to reside entirely in memory, avoiding performance jitter caused by cache eviction while reducing the demand for high-performance DRAM capacity.

### Test Environment
<%= _.env %>