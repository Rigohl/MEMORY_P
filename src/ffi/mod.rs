pub mod bridge;
pub mod error;
pub mod jax;
pub mod julia;
pub mod mojo;
pub mod pony;
pub mod zig;

pub fn init() -> crate::error::Result<()> { Ok(()) }
pub fn shutdown() {}
