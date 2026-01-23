use super::I18n;

/// Chinese language implementation
/// 中文语言实现
pub struct Zh;

impl I18n for Zh {
  const TRACE_NOT_FOUND: &'static str = "Trace 文件 {:?} 未找到！请先运行 'bun init.js'。";
  const SCANNING_TRACE: &'static str = "正在扫描 Trace 文件: {:?}";
  const TRACE_SCANNED: &'static str = "Trace 已扫描: {} 操作。最大 LBA: {}";
  const INITIALIZING_FTL: &'static str = "初始化 FTL...";
  const REPLAYING_TRACE: &'static str = "重放 trace 并采集热力图...";
  const PROCESSED_OPS: &'static str = "\r已处理 {} 操作...";
  const REPLAY_DURATION: &'static str = "\n重放耗时: {:.2?}";
  const SYNCING_TASKS: &'static str = "同步后台任务...";
  const INSPECTING_SEGMENTS: &'static str = "检查 segments...";
  const TOTAL_SEGMENTS: &'static str = "总 Segments: {}";
  const PGM_GROUPS: &'static str = "PGM Groups: {}";
  const DIRECT_GROUPS: &'static str = "Direct Groups: {}";
  const EMPTY_GROUPS: &'static str = "Empty Groups: {}";
  const COMPRESSION_FACTOR: &'static str = "压缩倍率: {:.2}x (空间节省: {:.2}%)";
  const LOG_PHY: &'static str = "Log: {} / Phy: {}";
  const OPT_IMMEDIATE: &'static str = "\n--- 优化模拟: Immediate Mode (Payload <= 42 bits) ---";
  const CANDIDATE_SEGMENTS: &'static str = "候选 Segments: {} ({:.2}%)";
  const POTENTIAL_SAVINGS: &'static str = "潜在 Payload 节省: {}";
  const PROJECTED_RATIO: &'static str = "预计压缩比: {:.2} (当前: {:.2})";
  const OPT_POLY: &'static str = "\n--- 优化模拟: 多态压缩 (紧凑 6B Segments) ---";
  const TYPE_A: &'static str = "Type A (Slope=1, Width=0) - 线性 Run: {} ({:.2}%)";
  const TYPE_B: &'static str = "Type B (Slope=0, Width=0) - 常量 Run: {} ({:.2}%)";
  const POTENTIAL_META_SAVINGS: &'static str = "潜在元数据节省: {}";
  const OPT_EXCEPTION: &'static str = "\n--- 优化模拟: 异常表 (每 Segment 1 个异常值) ---";
  const RESIDUAL_SPARSITY: &'static str = "\n--- 残差稀疏性 (非零位宽 segments) ---";
  const TOTAL_RESIDUALS: &'static str = "总 Residuals: {}";
  const ZERO_RESIDUALS: &'static str = "零 Residuals:  {} ({:.2}%)";
  const EFFECTIVE_VALUES: &'static str = "有效值: {} ({:.2}%)";
  const BW_DIST_HEADER: &'static str = "\n--- Segment 位宽分布 (每元素 Payload bits) ---";
  const BITS_0: &'static str = "0 bits: {} ({:.2}%)";
  const AVG_BITS: &'static str = "平均值: {:.2} bits";
  const P_BITS_VAL: &'static str = "P{}:    {} bits";
  const NO_SEGMENTS: &'static str = "未找到 segments。";
  const LEN_DIST_HEADER: &'static str = "\n--- Segment 长度分布 (直方图) ---";
  const AVG_LEN: &'static str = "平均值: {:.2}";
  const MIN_LEN: &'static str = "最小值: {}";
  const P_LEN_VAL: &'static str = "P{}:         {}";
  const MAX_LEN: &'static str = "最大值: {}";
  const DETAILED_LEN_DIST: &'static str = "\n详细分布:";
  const LEN_QUANTILE_FMT: &'static str = "{:.4}%: {} (数量: {})";
  const AVG_SEGMENTS_PER_PGM: &'static str = "PGM 组平均线段数: {:.2}";
  const OPT_PFOR: &'static str = "\n--- 优化模拟: PFOR (Patched Frame Of Reference) ---";
  const PFOR_CANDIDATE_SEGMENTS: &'static str = "PFOR 候选 Segments: {} ({:.2}%)";
  const POTENTIAL_PFOR_SAVINGS: &'static str = "潜在 PFOR 节省: {}";
}
