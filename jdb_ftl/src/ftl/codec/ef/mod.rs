pub mod iter;
pub mod traits;
pub mod util;
pub mod view;
pub mod writer;

// Re-export core types
pub use iter::EfIter;
pub use traits::{EfLayout, EfPrimitive, LayoutU16};
pub use util::SKIP_INTERVAL;
pub use view::EfView;
pub use writer::{byte_len, encode};

/// Type alias for EfView with U16 layout.
pub type EfViewU16<'a> = EfView<'a, LayoutU16>;
