use std::{env, path::PathBuf};

/// Get the project root directory
/// 获取项目根目录
pub fn root_dir() -> PathBuf {
  let path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
  if path.ends_with("examples")
    && let Some(parent) = path.parent()
  {
    return parent.to_path_buf();
  }
  path
}

/// Get the project data directory
/// 获取项目数据目录
pub fn data_dir() -> PathBuf {
  root_dir().join("data")
}

/// Get the benches/js/svg/json directory
/// 获取 benches/js/svg/json 目录
pub fn json_out_dir() -> PathBuf {
  root_dir().join("benches/js/svg/json")
}
