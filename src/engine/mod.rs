pub mod bindgroup;
pub mod camera;
pub mod config;
pub mod model;
pub mod render_pipeline;
pub mod shader;
pub mod texture;
pub mod vertex;

pub mod controller;

pub mod entity;
pub mod material;
pub mod mesh;
pub mod renderer;
pub mod resources;
pub mod scene;

use std::sync::Arc;

use wgpu_util::{framework::WgpuAppAction, hal::AppSurface};

use winit::{dpi::PhysicalSize, window::Window};

use crate::engine::{
    bindgroup::BindGroupInfo,
    config::GraphConfig,
    controller::Controller,
    render_pipeline::RenderPipelineInfo,
    renderer::Renderer,
    shader::ShaderInfo,
    texture::{Texture, TextureInfo},
    vertex::VertexBufferInfo,
};

type BoxResult<T> = anyhow::Result<T>;

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
    pub graph_resource: WgpuAppGraphResource,
    pub renderer: Renderer,
    timer: std::time::Duration,
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

        let renderer = Renderer::new(&app_surface, &graph_resource);

        let app = Self {
            app_surface,
            size,
            size_changed: false,
            controller,
            graph_resource,
            renderer,
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
        // self.camera.controller.process_event(&self.controller);
        // self.camera.update();
        self.timer += dt;

        // self.app_surface.queue.write_buffer(
        //     self.camera.buffer.as_ref().unwrap(),
        //     0,
        //     &self.camera.uniform.as_bytes(),
        // );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }
        self.resize_surface_if_needed();
        self.renderer
            .render(&self.app_surface, &self.graph_resource);
        Ok(())
    }
}
