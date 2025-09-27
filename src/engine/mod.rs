pub mod camera;
pub mod instance;
pub mod model;
pub mod render_pipeline;
pub mod resource;
pub mod shader;
pub mod texture;
pub mod vertex;

use std::{
    collections::HashMap,
    ops::Range,
    sync::{Arc, Mutex},
};

use cgmath::{InnerSpace, Matrix4, Rotation3};
use wgpu::util::DeviceExt;
use wgpu_util::{framework::WgpuAppAction, hal::AppSurface};

use winit::{dpi::PhysicalSize, window::Window};

use crate::{background::load_background, engine::{
    camera::{CameraConfig, CameraInfo},
    instance::Instance,
    model::{DrawModel, Model, ModelVertex},
    render_pipeline::RenderPipelineInfo,
    shader::ShaderInfo,
    texture::{Texture, TextureInfo},
    vertex::Vertex,
}};

pub enum UserDataType {
    Model(Arc<Model>, Arc<wgpu::Buffer>),
    ModelInstance(Arc<Model>, Range<u32>, Arc<wgpu::Buffer>),
}

pub struct WgpuApp {
    pub app_surface: AppSurface,
    pub size: PhysicalSize<u32>,
    pub size_changed: bool,
    pub camera: CameraInfo,
    pub texture: TextureInfo,
    pub shader: ShaderInfo,
    pub render_pipeline: RenderPipelineInfo,
    pub update_user_data: bool,
    pub user_data: Arc<Mutex<HashMap<String, Vec<UserDataType>>>>,
}

impl WgpuApp {
    fn resize_surface_if_needed(&mut self) {
        if self.size_changed {
            self.app_surface.config.width = self.size.width;
            self.app_surface.config.height = self.size.height;
            self.app_surface
                .surface
                .configure(&self.app_surface.device, &self.app_surface.config);
            self.size_changed = false;
            self.texture
                .depth_texture
                .replace(Texture::create_depth_texture(
                    &self.app_surface.device,
                    &self.app_surface.config,
                ));
        }
    }
}

fn load_resource(app: &WgpuApp) {
    let texture_bind_group_layout = app.texture.bind_group_layout.as_ref().unwrap();

    let Ok(rt) = tokio::runtime::Runtime::new() else {
        return;
    };

    let obj_model = rt
        .block_on(resource::load_model(
            &app.app_surface.device,
            &app.app_surface.queue,
            texture_bind_group_layout,
            "cube.obj",
        ))
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

                Instance { position, rotation }
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

impl WgpuAppAction for WgpuApp {
    async fn new(window: std::sync::Arc<Window>) -> Self {
        let app_surface = AppSurface::new(window).await.unwrap();

        let size = PhysicalSize {
            width: app_surface.config.width,
            height: app_surface.config.height,
        };

        // x: screen right
        // y: screen top
        // z: out of screen (to user)
        let camera_config = CameraConfig {
            eye: (f32::EPSILON, 0.0, 30.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            aspect: app_surface.config.width as f32 / app_surface.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let mut camera = CameraInfo::new(camera_config);
        camera.update_view_proj();
        camera.setup(&app_surface.device);

        let mut texture = TextureInfo::new();
        texture.setup(&app_surface.device, &app_surface.config);

        let mut shader = ShaderInfo::new();
        shader
            .setup_vertex_shader(&app_surface.device, "./shader/vertex.wgsl")
            .unwrap();
        shader
            .setup_fragment_shader(&app_surface.device, "./shader/fragment.wgsl")
            .unwrap();

        let mut render_pipeline = RenderPipelineInfo::new();
        let mut bind_group_layouts = Vec::new();
        let mut vertex_buffer_layouts = Vec::new();
        if let Some(layout) = texture.bind_group_layout.as_ref() {
            bind_group_layouts.push(layout);
        }
        if let Some(layout) = camera.bind_group_layout.as_ref() {
            bind_group_layouts.push(layout);
        }
        vertex_buffer_layouts.push(ModelVertex::desc());
        vertex_buffer_layouts.push(Instance::desc());
        render_pipeline
            .setup(
                &app_surface.device,
                &app_surface.config,
                &shader,
                vertex_buffer_layouts,
                Some(bind_group_layouts),
            )
            .unwrap();

        let user_data = Arc::new(Mutex::new(HashMap::new()));

        Self {
            app_surface,
            size,
            size_changed: false,
            camera,
            texture,
            shader,
            render_pipeline,
            update_user_data: true,
            user_data,
        }
    }

    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        if new_size == self.size {
            return;
        }
        self.size = new_size;
        self.size_changed = true;
    }

    fn get_size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(
            self.app_surface.config.width,
            self.app_surface.config.height,
        )
    }

    fn keyboard_input(&mut self, event: &winit::event::KeyEvent, _is_synthetic: bool) -> bool {
        self.camera.controller.process_event(event)
    }

    fn update(&mut self, _dt: std::time::Duration) {
        if self.update_user_data {
            load_background(&self);
            load_resource(&self);
            self.update_user_data = false;
        }

        self.camera.update();
        self.app_surface.queue.write_buffer(
            self.camera.buffer.as_ref().unwrap(),
            0,
            self.camera.uniform.as_bytes(),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }
        self.resize_surface_if_needed();
        let cur_texture = self.app_surface.surface.get_current_texture()?;
        let cur_view = cur_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.app_surface
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &cur_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.3,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.texture.depth_texture.as_ref().unwrap().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipeline.render_pipeline.as_ref().unwrap());

            for (data_tag, datas) in self.user_data.lock().unwrap().iter() {
                for data in datas {
                    match data {
                        UserDataType::Model(model, instance_buffer) => {
                            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                            render_pass.draw_model(
                                model,
                                self.camera.bind_group.as_ref().unwrap(),
                            );
                        }
                        UserDataType::ModelInstance(model, instances, instance_buffer) => {
                            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                            render_pass.draw_model_instanced(
                                model,
                                instances.clone(),
                                self.camera.bind_group.as_ref().unwrap(),
                            );
                        }
                    }
                }
            }
        }

        self.app_surface.queue.submit(Some(encoder.finish()));
        cur_texture.present();

        Ok(())
    }
}
