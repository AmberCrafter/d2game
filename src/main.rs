use wgpu_util::framework;

use crate::engine::{module::WgpuAppModule, registe_app_model, WgpuApp};

mod background;
mod engine;
mod item;
mod player;

fn main() -> anyhow::Result<()> {
    // registe_app_model("item", Box::new(item::ItemModule::new()));
    registe_app_model("player", Box::new(player::PlayerModule::new()));
    registe_app_model("background", Box::new(background::BackgroundModule::new()));

    // println!("Hello, world!");
    framework::run::<WgpuApp>("MyGame")?;
    Ok(())
}
