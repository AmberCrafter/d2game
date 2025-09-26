use wgpu_util::framework;

use crate::{engine::WgpuApp};

mod camera;
mod engine;
mod instance;
mod model;
mod render_pipeline;
mod resource;
mod shader;
mod texture;
mod vertex;

fn main() -> anyhow::Result<()> {
    // println!("Hello, world!");
    framework::run::<WgpuApp>("MyGame")?;
    Ok(())
}
