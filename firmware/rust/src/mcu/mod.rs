#[cfg(feature = "attiny1604")]
pub mod attiny1604;

#[cfg(feature = "feather32u4")]
pub mod feather32u4;

#[cfg(feature = "attiny1604")]
pub use attiny1604::*;

#[cfg(feature = "feather32u4")]
pub use feather32u4::*;