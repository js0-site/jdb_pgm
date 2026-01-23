# Configuration System and Automated Tuning: Finding the Optimal Performance Solution

The design philosophy of JDB-FTL is "scenario adaptation." Different workloads (such as sequential read/write, large-scale random write, or outlier distributions caused by GC) require different hyperparameters to exchange for optimal memory occupancy and access speed.


## 1. Zero-Overhead Static Configuration System

JDB-FTL discards runtime dictionary lookups in favor of a zero-overhead static configuration scheme based on **Rust Traits**:

### 1.1 `Conf` Trait
*   **Physical Implementation**: Through `build.rs`, implementation of the `Conf` trait and its corresponding static strategy code are dynamically generated at compile time based on a configuration file.
*   **Advantages**: Since settings are determined at compile time, the compiler can perform inline optimization on related mask operations and shift constants, completely eliminating the runtime overhead of accessing global variables or parsing JSON.

### 1.2 Key Configuration Items
*   **`GROUP_SIZE`**: The size of a data group. Larger groups can improve segment continuity and thus the compression ratio, but increase intra-group search latency.
*   **`PGM_EPSILON`**: The permitted error for PGM fitting. This is the core trade-off point between compression ratio and fitting precision.
*   **`WRITE_BUFFER_CAPACITY`**: The size of the L0 buffer, which determines the frequency of background flushes.


## 2. Automated Tuning : tune.py

Faced with a vast array of parameter combinations, manual debugging often fails to find the optimal solution. We provide a closed-loop hyperparameter tuning tool:

### 2.1 Hyperparameter Search Based on Genetic Algorithms
*   **Tool Path**: `tune.py` (calls `examples/score.rs`)
*   **Workflow**: Leverages the DEHB (Differential Evolution Hyperband) algorithm to perform evolutionary search within a preset parameter space for a given Trace trajectory file.
*   **Optimization Goal**: `Score = Throughput` (maximizing throughput), combined with several rigid constraints:
    - **Compression Ratio < 30%**: If exceeded, the score is penalized to 1%.
    - **Throughput ≥ 95% of Baseline**: If lower, the score is penalized to 1%.
    - **P99 Latency ≤ 110% of Baseline**: If exceeded, the score is penalized to 1%.

### 2.2 Tuning Workflow
1.  **Data Collection**: Capture real LBA read/write Traces from the environment.
2.  **Evolutionary Training**: Run `tune.py`; the tool automatically runs several FTL instances in parallel and calculates scores.
3.  **Parameter Deployment**: Generate the `Conf` trait implementation based on the optimal parameters output by the tool and recompile the project.


## 3. Performance Evolution Cost Model

To help engineers make scientific decisions, we have summarized the following core parameters and their tuning strategies:

### Key Compile-Time Configuration Parameters

| Parameter               | Default   | Description                                    |
| :---------------------- | :-------- | :--------------------------------------------- |
| `GROUP_SIZE`            | 4096      | Number of LBA entries per data group           |
| `PGM_EPSILON`           | 512       | PGM model permissible error threshold          |
| `WRITE_BUFFER_CAPACITY` | 4,194,304 | L0 buffer size (4MB), controls flush frequency |

> [!NOTE]
> These parameters can be adjusted via environment variables (`FTL_GROUP_SIZE`, `FTL_PGM_EPSILON`, `FTL_BUFFER_CAPACITY`) or using the `tune.py` automated search tool.


## 4. Conclusion

No single model fits all scenarios. However, with **PGM INDEX + Adaptive Residual Compensation = Lossless Compression** at its heart, supported by the dual-wheel drive of "static configuration" + "automated tuning," JDB-FTL enables mapping tables to achieve industrial-grade performance on various storage forms (such as NVMe SSDs, persistent memory, etc.) through simple parameter evolution.
