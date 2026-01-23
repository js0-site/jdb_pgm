pub mod bit_writer;
pub mod decoder;
pub mod ef;
pub mod encoder;
pub mod optimal;
pub mod util;

pub use bit_writer::BitWriter;
pub use decoder::{decode, decode_group, decode_segment, read_bits};
pub use encoder::encode;
pub use util::{bit_width, zigzag_decode, zigzag_encode};
