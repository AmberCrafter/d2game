use std::sync::Arc;

use async_trait::async_trait;
use cgmath::{Matrix4, Rotation3};
use tokio::sync::Mutex;
use wgpu::util::DeviceExt;

use crate::engine::{
    UserDataType, WgpuApp,
    instance::Instance,
    model::{Material, Mesh, Model, ModelVertex},
    module::WgpuAppModule,
    resource::load_texture,
};

const BACKGOUND_IMGAE_PATH: &str = "grassland.jpg";
const BG_SIZE: f32 = 25.0;
const BG_LAYZER: f32 = -5.0;
const VERTICES: &[ModelVertex] = &[
    ModelVertex {
        position: [BG_SIZE, BG_SIZE, BG_LAYZER],
        tex_coord: [1.0, 0.0],
        normal: [0.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [-BG_SIZE, BG_SIZE, BG_LAYZER],
        tex_coord: [0.0, 0.0],
        normal: [0.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [BG_SIZE, -BG_SIZE, BG_LAYZER],
        tex_coord: [1.0, 1.0],
        normal: [0.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [-BG_SIZE, -BG_SIZE, BG_LAYZER],
        tex_coord: [0.0, 1.0],
        normal: [0.0, 0.0, 0.0],
    },
];

const INDICES: &[u32] = &[0, 1, 2, 2, 1, 3];

async fn load_background(app: Arc<Mutex<WgpuApp>>) {
    let app = app.lock().await;
    let vertices_data = unsafe { VERTICES.align_to::<u8>().1 };
    let vertex_buffer =
        app.app_surface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Background vertex buffer"),
                contents: vertices_data,
                usage: wgpu::BufferUsages::VERTEX,
            });
    // let vertex_buffer_layout = ImageVertex::desc();

    let indices_data = unsafe { INDICES.align_to::<u8>().1 };
    let index_buffer =
        app.app_surface
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Background indices buffer"),
                contents: indices_data,
                usage: wgpu::BufferUsages::INDEX,
            });

    let background_texture = load_texture(
        &app.app_surface.device,
        &app.app_surface.queue,
        BACKGOUND_IMGAE_PATH,
    )
    .await
    .unwrap();
    let background_texture_bind_group_layout =
        app.graph_resource.bind_group_info.get("texture").unwrap();
    let background_texture_bind_group =
        app.app_surface
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Backgound bind group"),
                layout: background_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&background_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&background_texture.sampler),
                    },
                ],
            });

    // pack as model
    let model = Model {
        meshes: vec![Mesh {
            name: "Backgound".to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: INDICES.len() as u32,
            material: 0,
        }],
        materials: vec![Material {
            name: "Backgound".to_string(),
            diffuse_texture: background_texture,
            bind_group: background_texture_bind_group,
        }],
    };

    let instances = {
        let position = cgmath::vec3(1.0, 2.0, 3.0);
        let rotation =
            cgmath::Quaternion::from_axis_angle(cgmath::vec3(0.0, 0.0, 1.0), cgmath::Deg(90.0));

        Instance { position, rotation }
    };

    const SIZE_MAT4: usize = core::mem::size_of::<Matrix4<f32>>();
    let instance_data = vec![&instances]
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
    datas.push(UserDataType::Model(
        Arc::new(model),
        Arc::new(instance_buffer),
    ));
    let mut entry_lock = app.user_data.lock().unwrap();
    let entry = entry_lock.entry("background".to_string()).or_default();
    *entry = datas;
}

pub struct BackgroundModule {}

#[async_trait]
impl WgpuAppModule for BackgroundModule {
    fn new() -> Self {
        Self {}
    }

    async fn probe(&mut self, app: Arc<Mutex<WgpuApp>>) -> anyhow::Result<()> {
        load_background(app.clone()).await;
        Ok(())
    }
}
