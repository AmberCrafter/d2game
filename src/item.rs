use std::sync::Arc;

use async_trait::async_trait;
use cgmath::{InnerSpace, Matrix4, Rotation3};
use wgpu::util::DeviceExt;

use crate::engine::{UserDataType, WgpuApp, instance::Instance, module::WgpuAppModule, resource};

async fn load_resource(app: Arc<tokio::sync::Mutex<WgpuApp>>) {
    let app = app.lock().await;
    // let texture_bind_group_layout = app.texture.bind_group_layout.as_ref().unwrap();
    let texture_bind_group_layout = app.graph_resource.bind_group_info.get("texture").unwrap();

    let obj_model = resource::load_model(
        &app.app_surface.device,
        &app.app_surface.queue,
        texture_bind_group_layout,
        "cube.obj",
    )
    .await
    .unwrap();

    const NUM_INSTANCE_PER_ROW: u32 = 11;
    const SPACE_BETWEEN: f32 = 3.0;
    let instances = (0..NUM_INSTANCE_PER_ROW)
        .flat_map(|y| {
            (0..NUM_INSTANCE_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCE_PER_ROW as f32 / 2.0);
                let y = SPACE_BETWEEN * (y as f32 - NUM_INSTANCE_PER_ROW as f32 / 2.0);
                let position = cgmath::vec3(x as f32, y as f32, 0 as f32);
                let rotation = if position.magnitude() < f32::EPSILON {
                    cgmath::Quaternion::from_axis_angle(
                        cgmath::vec3(0.0, 0.0, 1.0),
                        cgmath::Deg(0.0),
                    )
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                let scale = 1.0;

                Instance { position, rotation, scale}
            })
        })
        .collect::<Vec<Instance>>();

    const SIZE_MAT4: usize = core::mem::size_of::<Matrix4<f32>>();
    let instance_data = instances
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
    datas.push(UserDataType::ModelInstance(
        Arc::new(obj_model),
        0..instances.len() as u32,
        Arc::new(instance_buffer),
    ));
    let mut entry_lock = app.user_data.lock().unwrap();
    let entry = entry_lock.entry("entity".to_string()).or_default();
    *entry = datas;
}

pub struct ItemModule {}

#[async_trait]
impl WgpuAppModule for ItemModule {
    fn new() -> Self {
        Self {}
    }

    async fn probe(&mut self, app: Arc<tokio::sync::Mutex<WgpuApp>>) -> anyhow::Result<()> {
        // load_resource(app.clone()).await;
        Ok(())
    }

    fn update(&mut self, dt: std::time::Duration) -> anyhow::Result<()> {
        Ok(())
    }
}
