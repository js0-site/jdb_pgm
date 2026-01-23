# Outlier Identification and Protection Strategy

In a PGM (Piecewise Geometric Model) index, outliers are data points that deviate significantly from the linear trend. Effectively identifying and handling these points is crucial for improving compression ratios and system robustness.

## 1. Design Philosophy

### 1.1 Cost Trade-off
The core of PGM is describing a segment using `(Base, Slope)`. If a segment has to increase its overall `BitWidth` (residual bit-width) just to include a "spike," or if it is forced to break prematurely because of that single point, the storage overhead skyrockets.

Our design philosophy: **If the cost of storing a point independently as an "outlier" is lower than the incremental cost of including it in a PGM segment, it should be excluded.**

### 1.2 Enhancing Compression Efficiency
Removing outliers significantly improves the predictive compression ratio of the PGM index. To illustrate this, consider a typical FTL write scenario:

**Scenario: Local Perturbation in Sequential Writes**
1. **Context**: The host is sequentially writing a large amount of data (e.g., LBA 100 → 1000). This data is typically distributed continuously on the physical storage (PBA), showing a clear linear trend.
2. **Perturbation**: During this process, a small metadata update or background wear-leveling occurs. As a result, LBA 500 is written to a distant physical location (e.g., PBA 99999), while the subsequent LBA 501 returns to the normal sequential sequence.
3. **Without Outlier Removal**: When building the index, PGM must force-break the current segment at LBA 500 to cover this "spike," resulting in multiple short, steeply-sloped segments. This causes the number of segments to surge, increasing metadata overhead.
4. **With Outlier Removal**: The system identifies LBA 500 as an outlier that deviates from the overall trend. It excludes it from the PGM linear modeling and instead records it in a side "patch table." Consequently, the main PGM index can maintain a single, long segment spanning 900+ points, drastically reducing storage consumption.

---

## 2. Implementation: Residual-Patch PGM

We have implemented a hybrid encoding scheme called **Residual-Patch**.

### 2.1 Greedy Extraction with Bridging

To handle local noise in the data, we introduced an **Outlier Bridging** strategy.

#### Core Logic: Lookahead instead of Break
Traditional PGM construction breaks the segment immediately when a point violates the error bound. This causes long segments to fragment when facing single-point noise (e.g., occasional metadata write jitter).

Our new strategy adds a **Lookahead** mechanism:
1.  When a point `P_i` that breaks the linear trend is encountered, the algorithm does not give up immediately but tries to **skip** this point and check subsequent points `P_{i+1}` and `P_{i+2}`.
2.  **Trend Consensus Verification**: If the subsequent two points can perfectly return to the current linear trend, the system determines that `P_i` is an **Internal Outlier (Bridged Outlier)**.
3.  **Bridging**: The algorithm marks `P_i` as an outlier (recorded in the Patch table) while **keeping the current segment unbroken** and continuing to extend it.

#### Pseudo-Code Implementation

Based on `src/ftl/codec/optimal.rs` and `pgm.rs`:

```python
# Core Algorithm Logic
SKIP_THRESHOLD = 4  # Minimum effective segment length

while cursor < n:
    # 1. Find the longest segment, supporting internal outlier skipping
    #    Lookahead: If a bad point is found, but the next 2 points fit, 
    #    skip the bad point and continue extending.
    (segment, skipped_indices) = find_longest_segment(start=cursor, lookahead_check=True)
    
    # Calculate effective length (excluding hollow points skipped in the middle)
    effective_length = segment.length - len(skipped_indices)

    # 2. Heuristic Check: Is the segment too short?
    if effective_length < SKIP_THRESHOLD:
        # [Strategy: Anchor Removal]
        # Even with bridging, this segment is too short. 
        # This implies the Start Point (Anchor) itself might be the outlier.
        # We discard the entire attempt, mark ONLY the 'Start Point' as an outlier, 
        # and restart the search from the next point.
        data.mark_as_outlier(cursor)
        cursor += 1
    else:
        # [Strategy: Commit Segment]
        # The segment is valid. Commit segment info.
        data.commit_segment(segment)
        
        # Register skipped points as "Internal Outliers"
        for idx in skipped_indices:
            data.mark_as_outlier(cursor + idx)
            
        cursor += segment.length
```


### 2.2 Residual Patch Encoding (PFOR-like)
For outliers, we do not store the raw PBA. Instead, we store the **Residual** between the actual PBA and the PGM model's prediction.
- **ZigZag Encoding**: Maps signed deviations to an unsigned space.
- **Dynamic Bit-Width Packing**: The optimal bit-width `OutlierBW` is determined for all outliers in a group and packed efficiently.

### 2.3 Exception Protection Layout
To ensure safety in worst-case scenarios, we designed a **Raw Fallback** mechanism:
- **Calculation**: The encoder calculates the total byte count before finalizing (Header + Index + PGM + Patches).
- **Fallback**: If the total length exceeds the original uncompressed size (32KB), it automatically reverts to `Mode 0` (Raw Storage) and updates the `GroupHeader`. This guarantees that even with completely incompressible random data, the system avoids space expansion.

---

## 3. Performance

In real-world trace testing, this strategy provided significant improvements:
- **Compression Improvement**: Excluding outliers increased the average PGM segment length by 55%, with global space savings jumping from 10% to **35%**.
- **Read Latency**: Using Elias-Fano indices, point queries can quickly determine if a patch table hit occurs, bypassing complex PGM math when necessary.

---
> For more codec details, please refer to the [Codec Documentation](codec.md).
