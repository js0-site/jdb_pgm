use jdb_ftl::ftl::codec::ef::{EfView, LayoutU16, byte_len, encode};

fn create_view<'a>(data: &'a [u8], n: usize, u_bound: usize) -> EfView<'a, LayoutU16> {
  EfView::<'a, LayoutU16>::new(data, n, u_bound)
}

#[test]
fn test_ef_roundtrip_empty() {
  let data = vec![];
  let encoded = encode(&data, 4096);
  let view = create_view(&encoded, 0, 4096);
  assert_eq!(view.get(0), 0xFFFF);
  assert_eq!(view.iter().count(), 0);
  assert_eq!(view.predecessor(100), (0, 0));
  assert_eq!(byte_len(0, 4096), 1);
}

#[test]
fn test_ef_basic() {
  let data = vec![10, 20, 30, 4095];
  let encoded = encode(&data, 4096);
  let _est_len = byte_len(data.len(), 4096);
  // byte_len is an upper bound estimate for allocation purposes.
  // Actual encoded size may be smaller when data max value << u_bound.
  assert!(!encoded.is_empty());

  let view = create_view(&encoded, data.len(), 4096);

  // Test Get
  assert_eq!(view.get(0), 10);
  assert_eq!(view.get(1), 20);
  assert_eq!(view.get(2), 30);
  assert_eq!(view.get(3), 4095);
  assert_eq!(view.get(4), 0xFFFF);

  // Test Iter
  let decoded: Vec<u16> = view.iter().collect();
  assert_eq!(decoded, data);

  // Test Predecessor
  // Target < Min (10)
  // Predecessor returns largest element <= target.
  // If target < min, none exists. Current Logic returns (0, 0) or potentially (0, val[0]) if checked?
  // My implementation: if idx==0 { return (0, 0) }
  // And if val > target, fallback to previous.
  // If target=5, val[0]=10. val[0] > target. fallback (0,0). Correct.
  assert_eq!(view.predecessor(5), (0, 0));

  // Target == Min (10)
  assert_eq!(view.predecessor(10), (0, 10));

  // Target in gap (15) -> 10
  assert_eq!(view.predecessor(15), (0, 10));

  // Target in gap (25) -> 20
  assert_eq!(view.predecessor(25), (1, 20));

  // Target (4000) -> 30
  assert_eq!(view.predecessor(4000), (2, 30));

  // Target >= Max (4095)
  assert_eq!(view.predecessor(4095), (3, 4095));
  assert_eq!(view.predecessor(4096), (3, 4095));
}

#[test]
fn test_ef_dense_sequential() {
  // 0, 1, 2 ... 999
  let n = 1000;
  let data: Vec<u16> = (0..n as u16).collect();
  let encoded = encode(&data, 4096);
  let view = create_view(&encoded, n, 4096);

  for i in 0..n {
    assert_eq!(view.get(i), i as u16);
    assert_eq!(view.predecessor(i as u16), (i, i as u16));
  }
}

#[test]
fn test_ef_sparse_random() {
  let mut rng = fastrand::Rng::new();
  let n = 500;
  let mut data = Vec::with_capacity(n);
  let mut set = std::collections::HashSet::new();
  while data.len() < n {
    let val = rng.u16(0..4096);
    if set.insert(val) {
      data.push(val);
    }
  }
  data.sort_unstable();

  let encoded = encode(&data, 4096);
  let view = create_view(&encoded, n, 4096);

  // 1. Verify exact matches
  for (i, &val) in data.iter().enumerate() {
    assert_eq!(view.get(i), val, "Mismatch at index {}", i);
    assert_eq!(
      view.predecessor(val),
      (i, val),
      "Predecessor failed for existing val {}",
      val
    );
  }

  // 2. Verify gaps
  for target in 0..4096 {
    let (idx, val) = view.predecessor(target);
    if let Some(&expected_val) = data.iter().rfind(|&&x| x <= target) {
      assert_eq!(
        val, expected_val,
        "Predecessor value mismatch for target {}",
        target
      );
      // Also check index
      assert_eq!(data[idx], val);
    } else {
      // Should return (0,0) if none found
      assert_eq!((idx, val), (0, 0), "Should find nothing for {}", target);
    }
  }
}

#[test]
fn test_ef_large_gaps() {
  // Test case designed to span multiple u64 words in Upper
  // Data: [0, 4095]. Huge gap.
  let data = vec![0, 4095];
  let encoded = encode(&data, 4096);
  let view = create_view(&encoded, 2, 4096);

  assert_eq!(view.get(0), 0);
  assert_eq!(view.get(1), 4095);

  assert_eq!(view.predecessor(0), (0, 0));
  // Gap check
  assert_eq!(view.predecessor(2000), (0, 0));
  assert_eq!(view.predecessor(4094), (0, 0));
  // Hit second
  assert_eq!(view.predecessor(4095), (1, 4095));
}
