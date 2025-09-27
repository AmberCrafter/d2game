use wgpu_util::framework;

use crate::{engine::WgpuApp};

mod engine;
mod background;

fn main() -> anyhow::Result<()> {
    // println!("Hello, world!");
    framework::run::<WgpuApp>("MyGame")?;
    Ok(())
}
