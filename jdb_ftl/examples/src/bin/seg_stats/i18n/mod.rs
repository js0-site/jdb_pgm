/// I18n trait for internationalization
/// 国际化 trait
#[allow(dead_code)]
pub trait I18n {
  const TRACE_NOT_FOUND: &'static str;
  const SCANNING_TRACE: &'static str;
  const TRACE_SCANNED: &'static str;
  const INITIALIZING_FTL: &'static str;
  const REPLAYING_TRACE: &'static str;
  const PROCESSED_OPS: &'static str;
  const REPLAY_DURATION: &'static str;
  const SYNCING_TASKS: &'static str;
  const INSPECTING_SEGMENTS: &'static str;
  const TOTAL_SEGMENTS: &'static str;
  const PGM_GROUPS: &'static str;
  const DIRECT_GROUPS: &'static str;
  const EMPTY_GROUPS: &'static str;
  const COMPRESSION_FACTOR: &'static str;
  const LOG_PHY: &'static str;
  const OPT_IMMEDIATE: &'static str;
  const CANDIDATE_SEGMENTS: &'static str;
  const POTENTIAL_SAVINGS: &'static str;
  const PROJECTED_RATIO: &'static str;
  const OPT_POLY: &'static str;
  const TYPE_A: &'static str;
  const TYPE_B: &'static str;
  const POTENTIAL_META_SAVINGS: &'static str;
  const OPT_EXCEPTION: &'static str;
  const RESIDUAL_SPARSITY: &'static str;
  const TOTAL_RESIDUALS: &'static str;
  const ZERO_RESIDUALS: &'static str;
  const EFFECTIVE_VALUES: &'static str;
  const BW_DIST_HEADER: &'static str;
  const BITS_0: &'static str;
  const AVG_BITS: &'static str;
  const P_BITS_VAL: &'static str;
  const NO_SEGMENTS: &'static str;
  const LEN_DIST_HEADER: &'static str;
  const AVG_LEN: &'static str;
  const MIN_LEN: &'static str;
  const P_LEN_VAL: &'static str;
  const MAX_LEN: &'static str;
  const DETAILED_LEN_DIST: &'static str;
  const LEN_QUANTILE_FMT: &'static str;
  const AVG_SEGMENTS_PER_PGM: &'static str;
  const OPT_PFOR: &'static str;
  const PFOR_CANDIDATE_SEGMENTS: &'static str;
  const POTENTIAL_PFOR_SAVINGS: &'static str;
}

mod en;
mod zh;

pub use en::En;
pub use zh::Zh;
