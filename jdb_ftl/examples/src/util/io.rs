use anyhow::Result;

use crate::util::path::json_out_dir;

/// Save data as JSON to the SVG output directory
/// 将数据作为 JSON 保存到 SVG 输出目录
pub fn save_svg_json<T: serde::Serialize, S: AsRef<str>>(name: S, data: &T) -> Result<()> {
  let dir = json_out_dir();
  if !dir.exists() {
    std::fs::create_dir_all(&dir)?;
  }
  let path = dir.join(format!("{}.json", name.as_ref()));
  let json = sonic_rs::to_string_pretty(data)?;
  std::fs::write(&path, json)?;
  println!("Results saved to: {:?}", path);
  Ok(())
}
