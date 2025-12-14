pub mod bindgroup;
pub mod camera;
pub mod config;
pub mod instance;
pub mod model;
pub mod render_pipeline;
pub mod resource;
pub mod shader;
pub mod texture;
pub mod vertex;

pub mod buffer;
pub mod controller;

use std::{
    collections::HashMap,
    ops::Range,
    sync::{Arc, Mutex},
};

use wgpu_util::{framework::WgpuAppAction, hal::AppSurface};

use winit::{dpi::PhysicalSize, window::Window};

use crate::engine::{
    bindgroup::BindGroupInfo,
    camera::{CameraConfig, CameraInfo},
    config::GraphConfig,
    controller::Controller,
    instance::{Instance, to_instance_buffer},
    model::{DrawModel, Model},
    render_pipeline::RenderPipelineInfo,
    shader::ShaderInfo,
    texture::{Texture, TextureInfo},
    vertex::VertexBufferInfo,
};

type BoxResult<T> = anyhow::Result<T>;

#[allow(unused)]
pub enum UserDataType {
    Model(Arc<std::sync::Mutex<Model>>, Arc<wgpu::Buffer>),
    ModelInstance(
        Arc<std::sync::Mutex<Model>>,
        Range<u32>,
        Arc<wgpu::Buffer>,
    ),
    // ModelInstanceBindGroup(Arc<Model>, Range<u32>, Arc<wgpu::Buffer>, Arc<wgpu::BindGroup>),
}

#[allow(unused)]
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
    pub controller: Controller,
    pub camera: CameraInfo,
    pub graph_resource: WgpuAppGraphResource,
    models: Vec<(Arc<std::sync::Mutex<Model>>, Vec<Instance>)>,
    pub user_data: Arc<Mutex<HashMap<String, UserDataType>>>,
    timer: std::time::Duration,
}

// static APP_MODELS: LazyLock<
//     Mutex<HashMap<String, Box<dyn WgpuAppModule + 'static + Sync + Send>>>,
// > = LazyLock::new(|| Mutex::new(HashMap::new()));

// pub fn registe_app_model(name: &str, module: Box<dyn WgpuAppModule + 'static + Sync + Send>) {
//     APP_MODELS.lock().unwrap().insert(name.to_string(), module);
// }

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

    pub fn register_model_instances(
        &mut self,
        name: &str,
        model: Model,
        instances: Vec<Instance>,
    ) -> anyhow::Result<()> {
        let instance_buffer = to_instance_buffer(&self.app_surface.device, instances.as_slice());

        let model = Arc::new(std::sync::Mutex::new(model));
        let instance_buffer = Arc::new(instance_buffer);
        let data = UserDataType::ModelInstance(
            model.clone(),
            0..instances.len() as u32,
            instance_buffer.clone(),
        );

        let mut entry_lock = self.user_data.lock().unwrap();
        entry_lock.insert(name.to_string(), data);

        self.models.push((model, instances));
        Ok(())
    }
}

impl WgpuAppAction for WgpuApp {
    async fn new(window: std::sync::Arc<Window>) -> Arc<std::sync::Mutex<Self>> {
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

        let controller = Controller::new();

        // x: screen right
        // y: screen top
        // z: out of screen (to user)
        let camera_config = CameraConfig {
            eye: (f32::EPSILON, 75.0, 100.0).into(),
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

        let models = Vec::new();
        let user_data = Arc::new(Mutex::new(HashMap::new()));

        let app = Self {
            app_surface,
            size,
            size_changed: false,
            controller,
            camera,
            graph_resource,
            models,
            user_data,
            timer: std::time::Duration::ZERO,
        };

        let arc_app = Arc::new(std::sync::Mutex::new(app));
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
        self.controller.parse_key_event(event)
        // self.camera.controller.process_event(event)

        // true
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.camera.controller.process_event(&self.controller);
        self.camera.update();
        self.timer += dt;

        let user_data = self.user_data.lock().unwrap();
        if let Some(player) = user_data.get("player") {
            match player {
                UserDataType::ModelInstance(model, _range, _instance) => {
                    let mut model = model.lock().unwrap();
                    model.update_animation("SphereAction", self.timer);
                    model.update_animation("Sphere.001Action", self.timer);
                    model.update_animation("Sphere.002Action", self.timer);
                    model.update_animation("Sphere.003Action", self.timer);
                    model.update_animation("Sphere.004Action", self.timer);
                    model.update_animation("Sphere.005Action", self.timer);
                    }
                _ => {}
            }
        }

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

            for (data_tag, data) in self.user_data.lock().unwrap().iter() {
                // println!("[Debug] {:?}({:?}) {data_tag:?}", file!(), line!());

                let render_pipeline = self
                    .graph_resource
                    .render_pipeline_info
                    .get(data_tag)
                    .unwrap();

                render_pass.set_pipeline(&render_pipeline);

                match data {
                    UserDataType::Model(model, instance_buffer) => {
                        let model = &*model.lock().unwrap();
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                        render_pass.draw_model(
                            &self.app_surface.queue,
                            model,
                            self.camera.bind_group.as_ref().unwrap(),
                        );
                    }
                    UserDataType::ModelInstance(model, instances, instance_buffer) => {
                        let model = &*model.lock().unwrap();
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                        render_pass.draw_model_instanced(
                            &self.app_surface.queue,
                            model,
                            instances.clone(),
                            self.camera.bind_group.as_ref().unwrap(),
                        );
                    }
                }
            }
        }

        self.app_surface.queue.submit(Some(encoder.finish()));
        cur_texture.present();

        Ok(())
    }
}
