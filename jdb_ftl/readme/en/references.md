# References

The following references and resources are central to the technologies and algorithms used in this project.

## 1. PGM-index
- **Title**: The PGM-index: a fully-dynamic compressed learned index with provable worst-case bounds
- **Website**: [https://pgm.di.unipi.it/](https://pgm.di.unipi.it/)
- **Paper**: [https://arxiv.org/abs/1910.06169](https://arxiv.org/abs/1910.06169)
- **Description**: The core indexing structure of JDB-FTL, utilizing piecewise linear regression for efficient compressed indexing.


## 2. Test Dataset (MSRC Trace)
- **Source**: SNIA MSRC Traces
- **Download**: [https://trace.camelab.org/Download.html](https://trace.camelab.org/Download.html)
- **Description**: Real-world enterprise I/O traces used for evaluating FTL performance.

## 3. Learned Lossless Compression Research
- **LeCo**: Lightweight Compression via Learning Serial Correlations
    - **Paper**: [https://arxiv.org/abs/2306.15374](https://arxiv.org/abs/2306.15374)
    - **Description**: Proposes a "Learned Compression" framework using piecewise linear models to fit data patterns and bit-packing for residual (delta) correction, enabling fast random access.
- **LIPP**: Updatable Learned Index with Precise Positions
    - **Paper**: [https://arxiv.org/abs/2104.05520](https://arxiv.org/abs/2104.05520)
    - **Description**: Ensures precise position predictions within leaf nodes through fine-grained model partitioning, eliminating the need for "last-mile" searches.
