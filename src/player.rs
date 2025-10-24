use std::sync::Arc;

use async_trait::async_trait;
use cgmath::Rotation3;

use crate::engine::{
    WgpuApp, instance::Instance, model::Model, module::WgpuAppModule, register_model_instances,
    resource,
};

async fn load_resource(
    app: Arc<tokio::sync::Mutex<WgpuApp>>,
) -> (Arc<tokio::sync::Mutex<Model>>, Arc<wgpu::Buffer>) {
    let app = app.lock().await;
    let texture_bind_group_layout = app
        .graph_resource
        .bind_group_info
        .get("player_gltf_texture")
        .unwrap();

    let obj_model = resource::load_gltf_model(
        &app.app_surface.device,
        &app.app_surface.queue,
        texture_bind_group_layout,
        "player_walk.gltf",
    )
    .await
    .unwrap();

    let instances = [{
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

    register_model_instances("player", &*app, obj_model, &instances)
}

pub struct PlayerModule {
    model: Option<Arc<tokio::sync::Mutex<Model>>>,
    instance_buffer: Option<Arc<wgpu::Buffer>>,
    counter: i32,
}

#[async_trait]
impl WgpuAppModule for PlayerModule {
    fn new() -> Self {
        Self {
            model: None,
            instance_buffer: None,
            counter: 0,
        }
    }

    async fn probe(&mut self, app: Arc<tokio::sync::Mutex<WgpuApp>>) -> anyhow::Result<()> {
        let model = load_resource(app.clone()).await;
        self.model.replace(model.0);
        self.instance_buffer.replace(model.1);
        Ok(())
    }

    fn update(&mut self, queue: &wgpu::Queue, _dt: std::time::Duration) -> anyhow::Result<()> {
        if let Some(model) = &mut self.model {
            let mut model = model.blocking_lock();
            for mesh in &mut model.meshes {
                if let Some(animation) = &mesh.animation {
                    let transform = animation.get_transform(0.003 * self.counter as f32)?;
                    mesh.update_transform(transform);
                }
            }
        }

        let instances = [{
            let position = cgmath::vec3(
                0.01 * if self.counter <= 5000 {
                    self.counter as f32
                } else {
                    (10000 - self.counter) as f32
                },
                0.0,
                0.0,
            );
            let rotation =
                // cgmath::Quaternion::from_axis_angle(cgmath::vec3(1.0, 0.0, 0.0), cgmath::Deg(90.0));
                cgmath::Quaternion::from_axis_angle(cgmath::vec3(0.0, 1.0, 0.0), cgmath::Deg(90.0 * if self.counter <= 5000 { 1.0 } else { -1.0 }));
            let scale = 5.0;

            Instance {
                position,
                rotation,
                scale,
            }
        }];

        queue.write_buffer(
            self.instance_buffer.as_ref().unwrap(),
            0,
            &instances[0].as_bytes(),
        );

        self.counter += 1;
        self.counter %= 10000;
        Ok(())
    }
}
