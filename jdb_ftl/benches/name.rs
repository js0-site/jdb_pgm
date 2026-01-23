pub trait Name {
  const NAME: &'static str;
}

#[cfg(feature = "bench_ftl")]
impl Name for jdb_ftl::DefaultFtl {
  const NAME: &'static str = "Ftl";
}

#[cfg(feature = "bench_base")]
impl Name for jdb_ftl::bench::base::Base {
  const NAME: &'static str = "[u8]";
}
