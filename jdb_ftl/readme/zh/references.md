# 参考文献

以下列出本项目核心技术所借鉴或引用的重要文献与资源。

## 1. PGM 索引
- **标题**: The PGM-index: a fully-dynamic compressed learned index with provable worst-case bounds (PGM 索引：一种具有可证明最坏情况界限的全动态压缩学习索引)
- **网站**: [https://pgm.di.unipi.it/](https://pgm.di.unipi.it/)
- **论文**: [https://arxiv.org/abs/1910.06169](https://arxiv.org/abs/1910.06169)
- **描述**: JDB-FTL 的核心索引结构，利用分段线性回归模型实现高效的压缩索引。


## 2. 测试数据集
- **来源**: SNIA MSRC Traces
- **下载**: [https://trace.camelab.org/Download.html](https://trace.camelab.org/Download.html)
- **描述**: 用于评估 FTL 性能的真实企业级 I/O 轨迹。

## 3. 学习型无损压缩研究
- **LeCo**: Lightweight Compression in Columnar Stores via Learned Piecewise Linear Models (LeCo：通过学习分段线性模型实现列存轻量级压缩)
    - **论文**: [https://arxiv.org/abs/2306.15374](https://arxiv.org/abs/2306.15374)
    - **描述**: 该论文提出了"学习型压缩"框架，核心逻辑是使用分段线性模型拟合数据规律，并对预测偏差（残差）进行位打包压缩，实现高效的随机访问。
- **LIPP**: Updatable Learned Index with Precise Positions (LIPP：具有精确位置的可更新学习型索引)
    - **论文**: [https://arxiv.org/abs/2104.05520](https://arxiv.org/abs/2104.05520)
    - **描述**: LIPP 通过精细化的模型分段，确保所有条目的预测位置在叶子节点都是精确的，从而消除了最后一步的搜索开销。
