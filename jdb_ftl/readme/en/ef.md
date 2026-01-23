# Elias-Fano Index Optimization: The Balance of Space and Time

Elias-Fano (EF) encoding is a highly efficient compression algorithm for monotonically increasing sequences, making it ideal for compressed sorted integer arrays. In this project, we have deeply optimized the standard Elias-Fano implementation by introducing a `u16` Skip Table and an improved `predecessor` search algorithm. This achieves near O(1) random access performance while maintaining an extremely low memory footprint (adding only about 1% overhead).

## 1. Usage in FTL

In the PGM (Piecewise Geometric Model) dynamic mapping system, to map massive amounts of LBAs (Logical Block Addresses) to PBAs (Physical Block Addresses), we partition the entire mapping space into thousands of linear **Segments**.

Each Segment has a starting LBA. To quickly locate which Segment a given LBA belongs to during a read operation, we need to store and search this monotonically increasing sequence of starting LBAs (`StartIdxs`).

**Why Elias-Fano?**
1.  **Extreme Compression**: The L2P table and its metadata must reside in memory. EF encoding compresses these sorted indices close to the theoretical lower bound (requiring very few bits per index), significantly reducing the memory footprint of the FTL.
2.  **Fast Predecessor Search**: The core operation in the read path is `predecessor(lba)`, used to quickly locate which Segment a target belongs to.
3.  **Sparse Index Mapping**: In our new "hole-less storage" architecture, EF encoding is used to directly store the logical offsets of all valid data. Via `predecessor(lba)`, we can instantly determine if a logical address exists and map it to a dense physical data stream index, completely replacing the previously expensive bitmap structures.

## 2. Core Architecture and Memory Layout

Our Elias-Fano implementation utilizes a Zero-Copy `EfView` structure, operating directly on memory-mapped byte streams without deserialization.

The memory layout is as follows:

```text
[Header (2 bytes)]
   - Byte 0: l (Lower bits count, 4 bits)
   - Byte 1: upper_len_bytes (Upper bits length)
[Upper Bits]
   - N + (U >> l) bits (Byte aligned)
   - Stores Unary Coding of Gaps for high bits
[Lower Bits]
   - N * l bits (Byte aligned)
   - Stores raw values of lower bits
[Skip Table]
   - ceil(N / 64) * 4 bytes
   - Stores acceleration index for every 64 elements
```

### 1.1 u16 Skip Table Design

To solve the O(N) linear scan issue of standard EF encoding for random access in long sequences, we introduced a compact Skip Table index:

*   **Sampling Interval (SKIP_INTERVAL)**: An index entry is created every 64 elements (Cache Line friendly).
*   **Index Content**:
    *   `bit_pos` (u16): The bit offset of the corresponding element in the Upper Bits stream.
    *   `prev_h` (u16): The High Value of the previous element, used for delta decoding.
*   **Space Overhead**: 4 bytes per entry. For a block of N=4096, 64 entries require 256 bytes, which is a negligible fraction (approx. 1%) of the total data.

## 2. Algorithm Optimization: Predecessor Search

`predecessor(target)` is the most frequently called operation in FTL, used to find the corresponding Segment based on LBA. We implemented a hybrid search strategy combining binary search and adaptive local scanning.

### 2.1 Algorithm Flow

1.  **Block Localization (Binary Search)**:
    Using `prev_h` in the Skip Table, we quickly locate the Block (range of 64 elements) where the `target` might exist via binary search. This reduces the search scope from N to 64.

2.  **Adaptive Local Scan**:
    Within the local range of 64 elements, we use linear scanning. For maximum performance, we distinguish two paths:
    *   **Forward Scan (Hot Path)**: Leveraging CPU branch prediction and cache prefetching, we linearly decode forward from the Block start. Typically, the target is found within 32 scans.
    *   **Backward Correction (Correction Path)**: Due to hash collisions or boundary cases, the target might be logically located before the current Block identified by the Skip Table. A fallback mechanism triggers `get(idx-1)` to verify and correct the result.

3.  **Stateless Pre-check**:
    Before starting the scan, we manually decode the first element of the Block. If this first element is already greater than the target, we fallback immediately. This avoids invalid loop decoding and complex state synchronization overheads.

## 3. Performance Benefits

*   **Cache Friendliness**: The Skip Table is small and contiguous, ensuring high hit rates in CPU L1/L2 caches.
*   **Reduced Branch Misprediction**: Local linear scanning offers a more stable instruction pipeline compared to complex binary searches or random bit stream jumps.
*   **Robustness**: The bidirectional scanning logic perfectly covers all boundary cases (e.g., target smaller than all values, larger than all, or falling exactly on Block boundaries).

This design allows our FTL to maintain microsecond-level query latency under large-scale data while keeping metadata memory usage to a minimum.
