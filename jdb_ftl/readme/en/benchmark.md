# Benchmarks and Engineering Robustness: The Source of Trust

As a core component of a storage system, JDB-FTL prioritizes correctness and performance stability. This chapter introduces our benchmarking system and how it ensures every code change stands up to real-world workloads.


## 1. Industrial-Grade Trace Playback System

JDB-FTL evaluation does not rely on simple random number generation but is based on actual I/O behavior:

### 1.1 Trace Channel
*   **Physical Implementation**: Supports variable-length binary trace parsing, enabling 1:1 reproduction of real LBA read/write sequences.
*   **Mainstream Support**: Built-in adapters for industrial standard datasets like MSR Cambridge (MSRC), simulating various business models from Web Servers to Exchange Servers.

### 1.2 High-Speed Playback Engine
*   **Million-Level Playback**: Thanks to extreme optimizations in the read path, single-threaded playback capacity reaches **60 million operations per second**. This allows us to complete stress tests on hours of production traffic within seconds.


## 2. Automated Performance Regression (Regress.js)

We focus not only on instantaneous performance but also on **performance sustainability**.

### 2.1 Performance Baseline Tracking
*   **Tool Path**: `./regress.js`
*   **Function Description**: This script automatically compares the current build with the Baseline version across various metrics and generates a color-coded comparison report.
*   **Monitored Metrics**:
    - **Memory Overload**: The increase in mapping table memory footprint.
    - **Throughput (MB/s)**: Percentage of throughput loss compared to pure memory arrays.
    - **Latency Percentiles**: Comprehensive comparison of P50, P90, and P99.

### 2.2 Build Admission Principles
If any modification causes memory usage to increase by more than 1% or read latency to increase by more than 5%, the regression script will throw a warning, forcing developers to re-examine the space tradeoffs.


## 3. Correctness Verification Loop

To ensure no bits are lost during compression and incremental updates, JDB-FTL has established a three-tier defense system:

1.  **Unit Verification (Unit Tests)**:
    Verifies the correctness of the PGM core fitting logic in extreme cases such as slope overflow, zero bit-width, and negative slopes.

2.  **Integrated Playback Test (Verify Trace)**:
    Run `cargo test --test verify`. This test executes a full FTL mapping instance, performing "write, flush, and read" operations on millions of mappings and comparing them one-by-one with a real Hash Map.

3.  **Concurrency Defense Test**:
    Simulates dirty region detection logic. Under high-frequency write interference, it verifies whether the PGM blocks produced by the background flush thread can 100% accurately restore the data.


## 4. Conclusion

In JDB-FTL, documentation, code, and testing are a trinity. The rigorous benchmarking system not only guarantees the industrial-grade quality of the product but also provides the most reliable feedback loop for our continuous optimization. Every improvement in performance metrics is documented and verifiable.
