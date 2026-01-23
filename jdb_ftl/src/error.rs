use thiserror::Error;

/// FTL specialized Result type.
/// FTL 专用的 Result 类型。
pub type Result<T> = std::result::Result<T, Error>;

/// FTL Error Enum.
/// FTL 错误枚举。
#[derive(Error, Debug)]
pub enum Error {
  /// Internal logic error or invariant violation.
  /// 内部逻辑错误或违反不变量。
  #[error("Internal error: {0}")]
  Internal(&'static str),
  /// Channel communication error.
  /// 通道通信错误。
  #[error("Communication channel error")]
  ChannelError,
  /// Configuration error.
  /// 配置错误。
  #[error("Configuration error: {0}")]
  ConfigError(&'static str),
}
