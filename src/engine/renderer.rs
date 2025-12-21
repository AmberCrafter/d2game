use cgmath::SquareMatrix;
use wgpu_util::hal::AppSurface;

use crate::engine::{
    WgpuAppGraphResource,
    camera::{Camera, CameraConfig},
    entity::Entity,
    material::Material,
    mesh::Mesh,
    model::Model,
    resources,
    scene::Scene,
    texture::Texture,
};

pub struct Renderer {
    pub scene: Scene,
}

#[allow(unused)]
impl Renderer {
    pub fn new(app_surface: &AppSurface, graph_resource: &WgpuAppGraphResource) -> Self {
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
        let camera = Camera::new(app_surface, graph_resource, camera_config);

        let scene = Scene::new(camera);
        Self { scene }
    }

    pub fn add_model(&mut self, model: Model) {
        self.scene.add_model(model);
    }

    pub fn clear_model(&mut self) {
        self.scene.clear_model();
    }

    pub fn render(&self, app_surface: &AppSurface, graph_resource: &WgpuAppGraphResource) {
        // println!("{:}({:})::render()", file!(), line!());

        // get previous frame information
        let frame = app_surface
            .surface
            .get_current_texture()
            .expect("Failed to fetch current texture.");
        let view = frame
            .texture
            .create_view(&wgpu::wgt::TextureViewDescriptor {
                // format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                ..Default::default()
            });

        let mut encoder = app_surface
            .device
            .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                // TODO: setup depth texture
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &graph_resource.texture.depth_texture.as_ref().unwrap().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            let tag = self.scene.name.as_deref().unwrap_or("default");
            let pipeline = graph_resource.render_pipeline_info.get(tag).unwrap();

            render_pass.set_pipeline(pipeline);
            self.scene.render(&mut render_pass);
        }
        app_surface.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    // consume resource here
    pub fn load_resources(
        &mut self,
        app_surface: &AppSurface,
        graph_resource: &WgpuAppGraphResource,
        resource: resources::Resource,
        tag: Option<&str>,
    ) {
        let textures = resource
            .textures
            .iter()
            .map(|texture| {
                Texture::load_texture_from_bytes(
                    &app_surface.device,
                    &app_surface.queue,
                    texture.name.as_deref(),
                    &resource.images[texture.image_index],
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        let (bind_group_layout, bind_group_config) = if let Some(tag) = tag {
            let layout = graph_resource.bind_group_info.get(tag).unwrap();
            let config = graph_resource
                .graph_config
                .resources
                .bindgroups
                .get(tag)
                .unwrap();
            (layout, config)
        } else {
            // self.graph_resource.bind_group_info.get("default").unwrap()
            unimplemented!()
        };

        let materials = resource
            .materials
            .iter()
            .map(|material| {
                Material::new(
                    material,
                    &textures,
                    &app_surface.device,
                    bind_group_layout,
                    bind_group_config,
                )
            })
            .collect::<Vec<_>>();

        let meshes = resource
            .meshes
            .iter()
            .map(|mesh| Mesh::new(mesh, &app_surface.device))
            .collect::<Vec<_>>();

        let entities = resource
            .nodes
            .into_iter()
            .map(|node| node.into())
            .collect::<Vec<Entity>>();

        let root_entity_indices = resource.scenes[resource.default_scene_index].nodes.clone();

        let root_entity = Entity {
            name: Some("Root".to_string()),
            mesh_index: None,
            children: root_entity_indices,
            transform: cgmath::Matrix4::from_diagonal(cgmath::Vector4::new(-1.0, 1.0, 1.0, 1.0)),
        };

        let model = Model::new(meshes, materials, entities, root_entity);

        self.add_model(model);
    }
}
