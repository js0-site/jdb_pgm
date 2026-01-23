pub trait Conf: Send + Sync + 'static {
  const GROUP_SIZE: usize;
  const WRITE_BUFFER_CAPACITY: usize;
  const PGM_EPSILON: usize;
}

include!(concat!(env!("OUT_DIR"), "/conf_gen.rs"));
