use std::{process::exit, sync::Arc};

use cgmath::Rotation3;

use crate::engine::{WgpuApp, instance::Instance, resource};

pub fn load_resource(app: Arc<std::sync::Mutex<WgpuApp>>) -> anyhow::Result<()> {
    let mut app = app.lock().unwrap();
    let texture_bind_group_layout = app
        .graph_resource
        .bind_group_info
        .get("player_gltf_texture")
        .unwrap();

    let obj_model = resource::load_gltf_model(
        &app.app_surface.device,
        &app.app_surface.queue,
        texture_bind_group_layout,
        "res/player_walk.gltf",
    )
    .unwrap();

    let instances = vec![{
        let position = cgmath::vec3(0.0, 0.0, 0.0);
        let rotation =
            // cgmath::Quaternion::from_axis_angle(cgmath::vec3(1.0, 0.0, 0.0), cgmath::Deg(90.0));
            cgmath::Quaternion::from_axis_angle(cgmath::vec3(1.0, 0.0, 0.0), cgmath::Deg(0.0));
        let scale = 5.0;

        Instance {
            position,
            rotation,
            scale,
        }
    }];

    app.register_model_instances("player", obj_model, instances)
}
