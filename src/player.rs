use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use cgmath::{Matrix4, Rotation3, SquareMatrix};
use wgpu::util::DeviceExt;

use crate::engine::{
    UserDataType, WgpuApp, instance::Instance, model::Model, module::WgpuAppModule, resource,
};

async fn load_resource(app: Arc<tokio::sync::Mutex<WgpuApp>>) -> Arc<Model> {
    let app = app.lock().await;
    // let texture_bind_group_layout = app.texture.bind_group_layout.as_ref().unwrap();
    let texture_bind_group_layout = app
        .graph_resource
        .bind_group_info
        .get("player_gltf_texture")
        .unwrap();

    // let obj_model = resource::load_obj_model(
    //     &app.app_surface.device,
    //     &app.app_surface.queue,
    //     texture_bind_group_layout,
    //     "player.obj",
    // )
    // .await
    // .unwrap();

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
            cgmath::Quaternion::from_axis_angle(cgmath::vec3(0.0, 0.0, 1.0), cgmath::Deg(0.0));
        let scale = 5.0;

        Instance {
            position,
            rotation,
            scale,
        }
    }];

    const SIZE_MAT4: usize = core::mem::size_of::<Matrix4<f32>>();
    let instance_data = &instances
        .iter()
        .flat_map(|val| {
            let model = val.as_model();
            unsafe { core::mem::transmute::<Matrix4<f32>, [u8; SIZE_MAT4]>(model) }
        })
        .collect::<Vec<u8>>();

    let instance_buffer =
        app.app_surface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instances buffer"),
                contents: instance_data.as_slice(),
                usage: wgpu::BufferUsages::VERTEX,
            });

    let mut datas = Vec::new();
    let obj_model = Arc::new(obj_model);
    datas.push(UserDataType::ModelInstance(
        obj_model.clone(),
        0..instances.len() as u32,
        Arc::new(instance_buffer),
    ));
    let mut entry_lock = app.user_data.lock().unwrap();
    let entry = entry_lock.entry("player".to_string()).or_default();
    *entry = datas;

    obj_model
}

pub struct PlayerModule {
    model: Option<Arc<Model>>,
    counter: u32,
}

#[async_trait]
impl WgpuAppModule for PlayerModule {
    fn new() -> Self {
        Self {
            model: None,
            counter: 0,
        }
    }

    async fn probe(&mut self, app: Arc<tokio::sync::Mutex<WgpuApp>>) -> anyhow::Result<()> {
        let model = load_resource(app.clone()).await;
        self.model.replace(model);
        Ok(())
    }

    fn update(&mut self, queue: &wgpu::Queue, dt: std::time::Duration) -> anyhow::Result<()> {
        if let Some(model) = &self.model {
            for mesh in &model.meshes {
                if let Some(animation) = &mesh.animation {
                    let transform = animation.get_transform(0.003 * self.counter as f32)?;
                    mesh.update_transform(transform);
                }
            }
        }

        self.counter += 1;
        Ok(())
    }
}

