// mod background;
mod engine;
// mod item;
// mod player;

use crate::engine::WgpuApp;
use wgpu_util::framework;

fn main() -> anyhow::Result<()> {
    // registe_app_model("item", Box::new(item::ItemModule::new()));
    // registe_app_model("player", Box::new(player::PlayerModule::new()));
    // registe_app_model("background", Box::new(background::BackgroundModule::new()));

    let mut fw = framework::init::<WgpuApp>("MyGame")?;
    // fw.add_resource_loader(Box::new(player::load_resource))?;
    fw.run().unwrap();
    Ok(())
}
