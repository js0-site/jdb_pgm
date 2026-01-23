use jdb_ftl::{DefaultFtl, FtlTrait};

#[test]
fn test_ftl_basic() {
  let mut ftl = DefaultFtl::new(1024);

  // Initial state should be None
  assert_eq!(ftl.get(0), None);
  assert_eq!(ftl.get(511), None);

  // Set and Get
  ftl.set(0, 100);
  assert_eq!(ftl.get(0), Some(100));

  ftl.set(100, 200);
  assert_eq!(ftl.get(100), Some(200));

  // Update
  ftl.set(0, 150);
  assert_eq!(ftl.get(0), Some(150));

  // Remove
  // ftl.rm(0);
  // assert_eq!(ftl.get(0), None);
}

#[test]
fn test_ftl_linear() {
  let n = 32;
  let mut ftl = DefaultFtl::new(n as u64);

  // Write a perfectly linear sequence in one group
  for i in 0..n {
    ftl.set(i as u64, (i * 10) as u64);
  }

  for i in 0..n {
    assert_eq!(ftl.get(i as u64), Some((i * 10) as u64));
  }
}

#[test]
fn test_ftl_packed_random() {
  let mut rng = fastrand::Rng::new();
  let n = 64;
  let mut ftl = DefaultFtl::new(n as u64);
  let mut expected = vec![u64::MAX; n];

  for _ in 0..200 {
    let idx = rng.u64(0..n as u64) as usize;
    let pba = rng.u64(0..10000);
    ftl.set(idx as u64, pba);
    expected[idx] = pba;

    // Verify all
    for (i, &val) in expected.iter().enumerate() {
      let actual = ftl.get(i as u64);
      let exp = if val == u64::MAX { None } else { Some(val) };
      assert_eq!(actual, exp, "Mismatch at index {}", i);
    }
  }
}

#[test]
fn test_ftl_cross_group_update() {
  let n = 512;
  let mut ftl = DefaultFtl::new(n as u64);

  // Fill group 0 with something that takes space
  for i in 0..32 {
    ftl.set(i as u64, i as u64 * 1000);
  }

  // Fill group 1
  for i in 32..64 {
    ftl.set(i as u64, i as u64 * 2000);
  }

  // Verify both
  for i in 0..64 {
    let mult = if i < 32 { 1000 } else { 2000 };
    assert_eq!(ftl.get(i as u64), Some(i as u64 * mult));
  }

  // Update a middle group and check others
  ftl.set(15, 999999);
  assert_eq!(ftl.get(15), Some(999999));
  assert_eq!(ftl.get(14), Some(14 * 1000));
  assert_eq!(ftl.get(32), Some(32 * 2000));
}

#[test]
fn test_ftl_descending_ppa() {
  let n = 32;
  let mut ftl = DefaultFtl::new(n as u64);

  // Descending sequence: pba = 10000 - i * 10
  for i in 0..n {
    ftl.set(i as u64, 10000 - (i * 10) as u64);
  }

  for i in 0..n {
    assert_eq!(ftl.get(i as u64), Some(10000 - (i * 10) as u64));
  }
}

#[test]
fn test_ftl_max_bits() {
  let n = 32;
  let mut ftl = DefaultFtl::new(n as u64);

  // Values that are far apart, requiring near 64 bits
  ftl.set(0, 1);
  ftl.set(1, u64::MAX - 1);

  assert_eq!(ftl.get(0), Some(1));
  assert_eq!(ftl.get(1), Some(u64::MAX - 1));
}

#[test]
fn test_ftl_near_max_val() {
  let n = 32;
  let mut ftl = DefaultFtl::new(n as u64);
  let target = u64::MAX - 100;

  for i in 0..n {
    ftl.set(i as u64, target + (i % 2) as u64);
  }

  for i in 0..n {
    assert_eq!(ftl.get(i as u64), Some(target + (i % 2) as u64));
  }
}

#[test]
fn test_ftl_full_frame_packed() {
  let n = 512; // 16 groups
  let mut ftl = DefaultFtl::new(n as u64);

  // Make every group PACKED with some large random-ish variations
  for i in 0..n {
    let val = (i as u64 * 1234567) ^ 0x5555555555555555;
    // Avoid u64::MAX
    let val = if val == u64::MAX { 0 } else { val };
    ftl.set(i as u64, val);
  }

  for i in 0..n {
    let expected = (i as u64 * 1234567) ^ 0x5555555555555555;
    let expected = if expected == u64::MAX { 0 } else { expected };
    assert_eq!(ftl.get(i as u64), Some(expected), "Mismatch at LBA {}", i);
  }
}
