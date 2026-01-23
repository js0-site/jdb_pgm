use std::{env, fs, path::Path};

fn main() {
  println!("cargo:rerun-if-env-changed=FTL_GROUP_SIZE");
  println!("cargo:rerun-if-env-changed=FTL_BUFFER_CAPACITY");
  println!("cargo:rerun-if-env-changed=FTL_PGM_EPSILON");

  let out_dir = env::var_os("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("conf_gen.rs");

  // tune.py relies on this exact formatting to update defaults
  let group_size = get_env_or_default("FTL_GROUP_SIZE", 4096usize);
  let buffer_capacity = get_env_or_default("FTL_BUFFER_CAPACITY", 4194304usize);
  let pgm_epsilon = get_env_or_default("FTL_PGM_EPSILON", 512usize);

  let content = format!(
    "
pub struct FtlConf;

impl Conf for FtlConf {{
    const GROUP_SIZE: usize = {};
    const WRITE_BUFFER_CAPACITY: usize = {};
    const PGM_EPSILON: usize = {};
}}
",
    group_size, buffer_capacity, pgm_epsilon
  );

  fs::write(&dest_path, content).unwrap();
  println!("cargo:rerun-if-changed=build.rs");
}

fn get_env_or_default(key: &str, default: usize) -> usize {
  match env::var(key) {
    Ok(val) => val.parse().unwrap_or(default),
    Err(_) => default,
  }
}
