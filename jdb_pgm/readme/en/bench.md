## Pgm-Index Benchmark

Performance comparison of Pgm-Index vs Binary Search with different epsilon values.

| Algorithm     | Epsilon | Data Size | Memory (MB) | Throughput (M/s) |
| ------------- | ------- | --------- | ----------- | ---------------- |
| jdb_pgm       | 32      | 1M        | < 0.01      | 67.21            |
| jdb_pgm       | 32      | 1M        | < 0.01      | 65.16            |
| jdb_pgm       | 64      | 1M        | < 0.01      | 58.90            |
| jdb_pgm       | 64      | 1M        | < 0.01      | 57.08            |
| jdb_pgm       | 128     | 1M        | < 0.01      | 50.65            |
| jdb_pgm       | 128     | 1M        | < 0.01      | 49.77            |
| pgm_index     | 32      | 1M        | 1.43        | 48.79            |
| pgm_index     | 32      | 1M        | 1.43        | 47.12            |
| pgm_index     | 64      | 1M        | 0.72        | 44.01            |
| pgm_index     | 64      | 1M        | 0.72        | 43.37            |
| pgm_index     | 128     | 1M        | 0.36        | 40.26            |
| Binary Search | -       | 1M        | 0           | 40.11            |
| pgm_index     | 128     | 1M        | 0.36        | 39.38            |
| BTreeMap      | -       | 1M        | 16.84       | 15.39            |

### Accuracy Comparison: jdb_pgm vs pgm_index

| Data Size | Epsilon | jdb_pgm Max | jdb_pgm Avg | pgm_index Max | pgm_index Avg |
| --------- | ------- | ----------- | ----------- | ------------- | ------------- |
| 1M        | 32      | 32          | 10.97       | 63            | 31.50         |
| 1M        | 64      | 64          | 21.87       | 128           | 63.50         |
| 1M        | 128     | 128         | 44.19       | 256           | 127.48        |

### Build Time Comparison: jdb_pgm vs pgm_index

| Data Size | Algorithm | Epsilon | Time   |
| --------- | --------- | ------- | ------ |
| 1M        | pgm_index | 32      | 1.40ms |
| 1M        | pgm_index | 64      | 1.28ms |
| 1M        | pgm_index | 128     | 1.24ms |
| 1M        | jdb_pgm   | 32      | 1.17ms |
| 1M        | jdb_pgm   | 64      | 1.19ms |
| 1M        | jdb_pgm   | 128     | 1.19ms |
### Benchmark ConfigQuery Count: 1,000,000Data Sizes: 1,000,000Epsilon Values: 32, 64, 128
---

### Epsilon (ε) Explained

*Epsilon (ε) controls the accuracy-speed trade-off:*

*Mathematical definition: ε defines the maximum absolute error between the predicted position and the actual position in the data array. When calling `load(data, epsilon, ...)`, ε guarantees |pred - actual| ≤ ε, where positions are indices within the data array of length `data.len()`.*

*Example: For 1M elements with ε=32, if the actual key is at position 1000:*
- ε=32 predicts position between 968-1032, then checks up to 64 elements
- ε=128 predicts position between 872-1128, then checks up to 256 elements

### Notes
#### What is Pgm-Index?
Pgm-Index (Piecewise Geometric Model Index) is a learned index structure that approximates the distribution of keys with piecewise linear models. It provides O(log ε) search time with guaranteed error bounds, where ε controls the trade-off between memory and speed.

#### Why Compare with Binary Search?
Binary search is the baseline for sorted array lookup. Pgm-Index aims to: match or exceed binary search performance, reduce memory overhead compared to traditional indexes, and provide better cache locality for large datasets.

#### Environment
- OS: macOS (arm64)
- CPU: Apple M2 Max- Cores: 12- Memory: 64GB
- Rust: 1.85.0
#### References
- [Pgm-Index Paper](https://doi.org/10.1145/3373718.3394764)
- [Official Pgm-Index Site](https://pgm.di.unipi.it/)
- [Learned Indexes](https://arxiv.org/abs/1712.01208)