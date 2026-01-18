use pgm_index::PGMIndex;
use rand::{Rng, SeedableRng, rngs::StdRng};

fn main() {
  let size = 1000;
  let mut rng = StdRng::seed_from_u64(42);
  let mut cur = 0u64;
  let data: Vec<u64> = (0..size)
    .map(|_| {
      cur += rng.random_range(1..100);
      cur
    })
    .collect();

  let eps = 32;
  let index = PGMIndex::new(data.clone(), eps);

  let mut total_err = 0.0;
  for (i, &key) in data.iter().enumerate() {
    let pred = index.predict_pos(key);
    let err = (pred as isize - i as isize).abs() as f64;
    total_err += err;
    if i < 10 {
      println!("Key: {}, Actual: {}, Pred: {}, Err: {}", key, i, pred, err);
    }
  }
  println!("Avg Err: {}", total_err / size as f64);
}
