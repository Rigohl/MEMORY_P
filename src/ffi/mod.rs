pub mod bridge;
pub mod error;
pub mod jax;
pub mod julia;
pub mod mojo;
pub mod pony;
pub mod zig;

pub async fn init() -> crate::error::Result<()> {
    julia::init()?;
    jax::init()?;
    mojo::init()?;
    pony::init().await?;
    Ok(())
}

pub fn shutdown() {
    julia::shutdown();
    jax::shutdown();
    mojo::shutdown();
    pony::shutdown();
}
