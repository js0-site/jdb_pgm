use super::I18n;

/// English language implementation
/// 英语语言实现
pub struct En;

impl I18n for En {
  const TRACE_NOT_FOUND: &'static str = "Trace file {:?} not found! Please run 'bun init.js'.";
  const SCANNING_TRACE: &'static str = "Scanning trace file: {:?}";
  const TRACE_SCANNED: &'static str = "Trace scanned: {} ops. Max LBA: {}";
  const INITIALIZING_FTL: &'static str = "Initializing FTL...";
  const REPLAYING_TRACE: &'static str = "Replaying trace & collecting heatmap...";
  const PROCESSED_OPS: &'static str = "\rProcessed {} ops...";
  const REPLAY_DURATION: &'static str = "\nReplay duration: {:.2?}";
  const SYNCING_TASKS: &'static str = "Syncing background tasks...";
  const INSPECTING_SEGMENTS: &'static str = "Inspecting segments...";
  const TOTAL_SEGMENTS: &'static str = "Total Segments: {}";
  const PGM_GROUPS: &'static str = "PGM Groups: {}";
  const DIRECT_GROUPS: &'static str = "Direct Groups: {}";
  const EMPTY_GROUPS: &'static str = "Empty Groups: {}";
  const COMPRESSION_FACTOR: &'static str = "Compression Factor: {:.2}x (Space Saving: {:.2}%)";
  const LOG_PHY: &'static str = "Log: {} / Phy: {}";
  const OPT_IMMEDIATE: &'static str =
    "\n--- Optimization Simulation: Immediate Mode (Payload <= 42 bits) ---";
  const CANDIDATE_SEGMENTS: &'static str = "Candidate Segments: {} ({:.2}%)";
  const POTENTIAL_SAVINGS: &'static str = "Potential Payload Savings: {}";
  const PROJECTED_RATIO: &'static str = "Projected Compression Ratio: {:.2} (Current: {:.2})";
  const OPT_POLY: &'static str =
    "\n--- Optimization Simulation: Polymorphic Compression (Compact 6B Segments) ---";
  const TYPE_A: &'static str = "Type A (Slope=1, Width=0) - Linear Run: {} ({:.2}%)";
  const TYPE_B: &'static str = "Type B (Slope=0, Width=0) - Constant Run: {} ({:.2}%)";
  const POTENTIAL_META_SAVINGS: &'static str = "Potential Metadata Savings: {}";
  const OPT_EXCEPTION: &'static str =
    "\n--- Optimization Simulation: Exception Table (1 Outlier per Segment) ---";
  const RESIDUAL_SPARSITY: &'static str =
    "\n--- Residual Sparsity (Non-zero bit-width segments) ---";
  const TOTAL_RESIDUALS: &'static str = "Total Residuals: {}";
  const ZERO_RESIDUALS: &'static str = "Zero Residuals:  {} ({:.2}%)";
  const EFFECTIVE_VALUES: &'static str = "Effective Values: {} ({:.2}%)";
  const BW_DIST_HEADER: &'static str =
    "\n--- Segment Bit Width Distribution (Payload bits per element) ---";
  const BITS_0: &'static str = "0 bits: {} ({:.2}%)";
  const AVG_BITS: &'static str = "Avg: {:.2} bits";
  const P_BITS_VAL: &'static str = "P{}:    {} bits";
  const NO_SEGMENTS: &'static str = "No segments found.";
  const LEN_DIST_HEADER: &'static str = "\n--- Segment Length Distribution (Histogram) ---";
  const AVG_LEN: &'static str = "Avg: {:.2}";
  const MIN_LEN: &'static str = "Min: {}";
  const P_LEN_VAL: &'static str = "P{}:         {}";
  const MAX_LEN: &'static str = "Max: {}";
  const DETAILED_LEN_DIST: &'static str = "\nDetailed Distribution:";
  const LEN_QUANTILE_FMT: &'static str = "{:.4}%: {} (count: {})";
  const AVG_SEGMENTS_PER_PGM: &'static str = "Average segments per PGM group: {:.2}";
  const OPT_PFOR: &'static str =
    "\n--- Optimization Simulation: PFOR (Patched Frame Of Reference) ---";
  const PFOR_CANDIDATE_SEGMENTS: &'static str = "PFOR Candidate Segments: {} ({:.2}%)";
  const POTENTIAL_PFOR_SAVINGS: &'static str = "Potential PFOR Savings: {}";
}
