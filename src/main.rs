use wgpu_util::framework;

use crate::engine::WgpuApp;

mod background;
mod engine;
mod item;

fn main() -> anyhow::Result<()> {
    // println!("Hello, world!");
    framework::run::<WgpuApp>("MyGame")?;
    Ok(())
}
