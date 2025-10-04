pub mod bindgroup;
pub mod camera;
pub mod config;
pub mod instance;
pub mod model;
pub mod module;
pub mod render_pipeline;
pub mod resource;
pub mod shader;
pub mod texture;
pub mod vertex;

use std::{
    cell::LazyCell,
    collections::HashMap,
    ops::Range,
    pin::Pin,
    sync::{Arc, LazyLock, Mutex, OnceLock},
};

use cgmath::{InnerSpace, Matrix4, Rotation3};
use wgpu::util::DeviceExt;
use wgpu_util::{framework::WgpuAppAction, hal::AppSurface};

use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    background::{self},
    engine::{
        bindgroup::BindGroupInfo,
        camera::{CameraConfig, CameraInfo},
        config::GraphConfig,
        instance::Instance,
        model::{DrawModel, Model},
        module::WgpuAppModule,
        render_pipeline::RenderPipelineInfo,
        shader::ShaderInfo,
        texture::{Texture, TextureInfo},
        vertex::VertexBufferInfo,
    },
    item, player,
};

pub enum UserDataType {
    Model(Arc<Model>, Arc<wgpu::Buffer>),
    ModelInstance(Arc<Model>, Range<u32>, Arc<wgpu::Buffer>),
}

pub struct WgpuAppGraphResource {
    pub graph_config: GraphConfig,
    pub shader: ShaderInfo,
    pub texture: TextureInfo,
    pub vertex_buffer_info: VertexBufferInfo,
    pub bind_group_info: BindGroupInfo,
    pub render_pipeline_info: RenderPipelineInfo,
}

pub struct WgpuApp {
    pub app_surface: AppSurface,
    pub size: PhysicalSize<u32>,
    pub size_changed: bool,
    pub camera: CameraInfo,
    pub graph_resource: WgpuAppGraphResource,
    pub user_data: Arc<Mutex<HashMap<String, Vec<UserDataType>>>>,
}

static APP_MODULES: LazyLock<
    Mutex<HashMap<String, Box<dyn WgpuAppModule + 'static + Sync + Send>>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn registe_module(name: &str, module: Box<dyn WgpuAppModule + 'static + Sync + Send>) {
    APP_MODULES.lock().unwrap().insert(name.to_string(), module);
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
            self.graph_resource
                .texture
                .depth_texture
                .replace(Texture::create_depth_texture(
                    &self.app_surface.device,
                    &self.app_surface.config,
                ));
        }
    }
}

impl WgpuAppAction for WgpuApp {
    async fn new(window: std::sync::Arc<Window>) -> Arc<tokio::sync::Mutex<Self>> {
        let app_surface = AppSurface::new(window).await.unwrap();

        let graph_config = GraphConfig::new("./src/config/graph.toml");
        let mut vertex_buffer_info = VertexBufferInfo::new();
        vertex_buffer_info.setup_config(&graph_config);
        let mut bind_group_info = BindGroupInfo::new();
        bind_group_info.setup(&graph_config, &app_surface.device);

        let size = PhysicalSize {
            width: app_surface.config.width,
            height: app_surface.config.height,
        };

        // x: screen right
        // y: screen top
        // z: out of screen (to user)
        let camera_config = CameraConfig {
            eye: (f32::EPSILON, 0.0, 100.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            aspect: app_surface.config.width as f32 / app_surface.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 200.0,
        };
        let mut camera = CameraInfo::new(camera_config);
        camera.update_view();
        camera.setup(&app_surface.device, bind_group_info.get("camera").unwrap());

        let mut texture = TextureInfo::new();
        texture.setup(&app_surface.device, &app_surface.config);

        let mut shader = ShaderInfo::new();
        shader.load_config(&app_surface.device, &graph_config);

        let mut render_pipeline_info = RenderPipelineInfo::new();
        render_pipeline_info
            .setup(
                &app_surface.device,
                &app_surface.config,
                &graph_config,
                &shader,
                &vertex_buffer_info,
                &bind_group_info,
            )
            .unwrap();

        let graph_resource = WgpuAppGraphResource {
            graph_config,
            texture,
            shader,
            vertex_buffer_info,
            bind_group_info,
            render_pipeline_info,
        };

        let user_data = Arc::new(Mutex::new(HashMap::new()));

        // registe_module("item", Box::new(item::ItemModule::new()));
        registe_module("player", Box::new(player::PlayerModule::new()));
        registe_module("background", Box::new(background::BackgroundModule::new()));

        let app = Self {
            app_surface,
            size,
            size_changed: false,
            camera,
            graph_resource,
            user_data,
        };

        let arc_app = Arc::new(tokio::sync::Mutex::new(app));
        let mut module_lock = APP_MODULES.lock().unwrap();
        for ele in module_lock.iter_mut() {
            let _ = ele.1.probe(arc_app.clone()).await;
        }

        arc_app
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
        // self.camera.controller.process_event(event)

        true
    }

    fn update(&mut self, dt: std::time::Duration) {
        let mut module_lock = APP_MODULES.lock().unwrap();
        for ele in module_lock.iter_mut() {
            let _ = ele.1.update(dt);
        }

        self.camera.update();
        self.app_surface.queue.write_buffer(
            self.camera.buffer.as_ref().unwrap(),
            0,
            &self.camera.uniform.as_bytes(),
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
                    view: &self
                        .graph_resource
                        .texture
                        .depth_texture
                        .as_ref()
                        .unwrap()
                        .view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });


            for (data_tag, datas) in self.user_data.lock().unwrap().iter() {
                // println!("[Debug] {:?}({:?}) {data_tag:?}", file!(), line!());

                let render_pipeline = self
                    .graph_resource
                    .render_pipeline_info
                    .get(data_tag)
                    .unwrap();

                render_pass.set_pipeline(&render_pipeline);

                for data in datas {
                    match data {
                        UserDataType::Model(model, instance_buffer) => {
                            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                            render_pass.draw_model(model, self.camera.bind_group.as_ref().unwrap());
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
