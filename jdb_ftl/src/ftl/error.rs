use std::fmt;

/// Custom Error types for FTL operations.
/// FTL 操作的自定义错误类型。
#[derive(Debug)]
pub enum FtlError {
  /// IO Error during trace or storage operations.
  /// 追踪或存储操作期间的 IO 错误。
  Io(std::io::Error),
  /// Channel closed unexpectedly.
  /// 通道意外关闭。
  ChannelClosed,
  /// Background thread panicked.
  /// 后台线程崩溃。
  ThreadPanic,
}

impl std::error::Error for FtlError {}

impl fmt::Display for FtlError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Io(e) => write!(f, "IO Error: {}", e),
      Self::ChannelClosed => write!(f, "Channel closed unexpectedly"),
      Self::ThreadPanic => write!(f, "Background thread panicked"),
    }
  }
}

impl From<std::io::Error> for FtlError {
  fn from(err: std::io::Error) -> Self {
    Self::Io(err)
  }
}

/// Result alias for FTL operations.
/// FTL 操作的结果别名。
pub type Result<T> = std::result::Result<T, FtlError>;
