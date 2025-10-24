use std::sync::Arc;

use async_trait::async_trait;
use cgmath::Rotation3;
use tokio::sync::Mutex;
use wgpu::util::DeviceExt;

use crate::engine::{
    WgpuApp,
    instance::Instance,
    model::{Mesh, Model, ModelVertex, ObjMaterial},
    module::WgpuAppModule,
    register_model_instances,
    resource::load_texture,
};

const BACKGOUND_IMGAE_PATH: &str = "grassland.jpg";
const BG_SIZE: f32 = 100.0;
const BG_LAYZER: f32 = 2.0;
const BG_X_SCALE: f32 = 10.0;
const VERTICES: &[ModelVertex] = &[
    ModelVertex {
        position: [BG_SIZE * BG_X_SCALE, BG_LAYZER, BG_SIZE],
        tex_coord: [2.0 * BG_X_SCALE, -2.0],
        normal: [0.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [-BG_SIZE * BG_X_SCALE, BG_LAYZER, BG_SIZE],
        tex_coord: [-2.0 * BG_X_SCALE, -2.0],
        normal: [0.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [BG_SIZE * BG_X_SCALE, BG_LAYZER, -BG_SIZE],
        tex_coord: [2.0 * BG_X_SCALE, 2.0],
        normal: [0.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [-BG_SIZE * BG_X_SCALE, BG_LAYZER, -BG_SIZE],
        tex_coord: [-2.0 * BG_X_SCALE, 2.0],
        normal: [0.0, 0.0, 0.0],
    },
];

// const INDICES: &[u32] = &[0, 1, 2, 2, 1, 3, 3, 1, 2, 2, 1, 0];
const INDICES: &[u32] = &[3, 1, 2, 2, 1, 0];

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
    let background_texture_bind_group_layout = app
        .graph_resource
        .bind_group_info
        .get("bg_texture")
        .unwrap();
    let background_texture_bind_group =
        app.app_surface
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Backgound bind group"),
                layout: background_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(&background_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::Sampler(&background_texture.sampler),
                    },
                ],
            });

    // pack as model
    let model = Model {
        meshes: vec![Mesh {
            name: "Backgound".to_string(),
            vertex: VERTICES.to_vec(),
            vertex_buffer,
            index_buffer,
            num_elements: INDICES.len() as u32,
            material: 0,
            animation: None,
            uniform_transform: None,
        }],
        materials: vec![Arc::new(std::sync::Mutex::new(ObjMaterial {
            name: "Backgound".to_string(),
            diffuse_texture: Some(background_texture),
            bind_group: Some(background_texture_bind_group),
            ..Default::default()
        })) as Arc<_>],
    };

    let instances = {
        let position = cgmath::vec3(0.0, 0.0, 0.0);
        let rotation =
            cgmath::Quaternion::from_axis_angle(cgmath::vec3(0.0, 0.0, 0.0), cgmath::Deg(0.0));
        let scale = 1.0;
        Instance {
            position,
            rotation,
            scale,
        }
    };

    register_model_instances("background", &*app, model, &[instances]);
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
