use std::{collections::HashMap, fmt::Debug, ops::Range, sync::Arc};

use cgmath::SquareMatrix;

use crate::engine::BoxResult;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub vertex: Vec<ModelVertex>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
    pub uniform_transform: Option<crate::engine::buffer::Buffer<cgmath::Matrix4<f32>>>,
}

impl Mesh {
    pub fn update_transform(&mut self, transform: cgmath::Matrix4<f32>) {
        if let Some(buffer) = self.uniform_transform.as_mut() {
            let _ = buffer.set_data(transform);
        }
    }
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct ObjMaterial {
    pub name: String,
    pub ambient: Option<wgpu::Buffer>,
    pub diffuse: Option<wgpu::Buffer>,
    pub specular: Option<wgpu::Buffer>,
    pub shininess: Option<wgpu::Buffer>,
    pub optical_density: Option<wgpu::Buffer>,
    pub ambient_texture: Option<crate::engine::texture::Texture>,
    pub diffuse_texture: Option<crate::engine::texture::Texture>,
    pub specular_texture: Option<crate::engine::texture::Texture>,
    pub normal_texture: Option<crate::engine::texture::Texture>,
    pub shininess_texture: Option<crate::engine::texture::Texture>,
    pub dissolve_texture: Option<crate::engine::texture::Texture>,
    pub illumination_model: Option<wgpu::Buffer>,

    // pub buffers: Option<Vec<wgpu::Buffer>>,
    // pub textures: Option<Vec<wgpu::Texture>>,
    pub bind_group: Option<wgpu::BindGroup>,
}

#[derive(Debug, Default)]
pub struct PBRMaterial {
    pub name: String,
    pub base_color: Option<wgpu::Buffer>,
    pub metallic: Option<wgpu::Buffer>,
    pub roughness: Option<wgpu::Buffer>,
    pub base_color_texture: Option<crate::engine::texture::Texture>,
    pub metallic_roughness_texture: Option<crate::engine::texture::Texture>,
    pub normal_texture: Option<crate::engine::texture::Texture>,
    pub occlusion_texture: Option<crate::engine::texture::Texture>,
    pub emissive_factor: Option<wgpu::Buffer>,
    pub emissive_texture: Option<crate::engine::texture::Texture>,

    pub bind_group: Option<wgpu::BindGroup>,
}

pub trait Material {
    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        None
    }
}

impl Debug for dyn Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Material for ObjMaterial {
    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }
}

impl Material for PBRMaterial {
    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }
}

#[derive(Debug, Default)]
pub struct Animation {
    pub name: String,
    pub mesh_id: usize,
    pub translation: Option<Vec<(f32, [f32; 3])>>,
    pub rotation: Option<Vec<(f32, [f32; 4])>>,
    pub scale: Option<Vec<(f32, [f32; 3])>>,
    pub period: f32,
    pub do_loop: bool,
}

impl Animation {
    pub fn update_period(&mut self, period: f32) -> BoxResult<()> {
        self.period = period;
        Ok(())
    }

    pub fn set_translations(&mut self, translations: Vec<(f32, [f32; 3])>) -> BoxResult<()> {
        self.translation.replace(translations);
        Ok(())
    }

    pub fn set_rotations(&mut self, rotations: Vec<(f32, [f32; 4])>) -> BoxResult<()> {
        self.rotation.replace(rotations);
        Ok(())
    }

    pub fn set_scales(&mut self, scales: Vec<(f32, [f32; 3])>) -> BoxResult<()> {
        self.scale.replace(scales);
        Ok(())
    }

    #[inline]
    fn get_time(&self, time: f32) -> f32 {
        if self.do_loop {
            time % self.period
        } else {
            time.min(self.period)
        }
    }

    fn get_translation_matrix(&self, time: f32) -> cgmath::Matrix4<f32> {
        if self.translation.is_none() || self.period == 0.0 {
            return cgmath::Matrix4::identity();
        }
        let tick = self.get_time(time);
        let mut prev = &self.translation.as_ref().unwrap()[0];
        let mut curr = prev;
        // linear
        for val in self.translation.as_ref().unwrap() {
            if tick > val.0 {
                prev = val;
                continue;
            }
            curr = val;
        }

        let weight = (tick - prev.0) / (curr.0 - prev.0);
        let translation = [
            prev.1[0] + (curr.1[0] - prev.1[0]) * weight,
            prev.1[1] + (curr.1[1] - prev.1[1]) * weight,
            prev.1[2] + (curr.1[2] - prev.1[2]) * weight,
        ]
        .into();

        cgmath::Matrix4::from_translation(translation)
    }

    fn get_rotation_matrix(&self, time: f32) -> cgmath::Matrix4<f32> {
        if self.rotation.is_none() || self.period == 0.0 {
            return cgmath::Matrix4::identity();
        }
        let tick = self.get_time(time);
        let mut prev = &self.rotation.as_ref().unwrap()[0];
        let mut curr = prev;
        // linear
        for val in self.rotation.as_ref().unwrap() {
            if tick > val.0 {
                prev = val;
                continue;
            }
            curr = val;
        }

        let weight = (tick - prev.0) / (curr.0 - prev.0);
        let prev_quat = cgmath::Quaternion::new(prev.1[3], prev.1[0], prev.1[1], prev.1[2]);
        let curr_quat = cgmath::Quaternion::new(curr.1[3], curr.1[0], curr.1[1], curr.1[2]);
        let rotation = prev_quat.slerp(curr_quat, weight);
        cgmath::Matrix4::from(rotation)
    }

    fn get_scale_matrix(&self, time: f32) -> cgmath::Matrix4<f32> {
        if self.scale.is_none() || self.period == 0.0 {
            return cgmath::Matrix4::identity();
        }
        let tick = self.get_time(time);
        let mut prev = &self.scale.as_ref().unwrap()[0];
        let mut curr = prev;
        // linear
        for val in self.scale.as_ref().unwrap() {
            if tick > val.0 {
                prev = val;
                continue;
            }
            curr = val;
        }

        let weight = (tick - prev.0) / (curr.0 - prev.0);

        let scale = [
            prev.1[0] + (curr.1[0] - prev.1[0]) * weight,
            prev.1[1] + (curr.1[1] - prev.1[1]) * weight,
            prev.1[2] + (curr.1[2] - prev.1[2]) * weight,
        ];

        cgmath::Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2])
    }

    pub fn get_transform(&self, time: f32) -> BoxResult<cgmath::Matrix4<f32>> {
        let translation = self.get_translation_matrix(time);
        let rotation = self.get_rotation_matrix(time);
        let scale = self.get_scale_matrix(time);

        Ok(translation * rotation * scale)
    }
}

pub struct Model {
    pub name: String,
    pub meshes: Vec<Mesh>, // index base
    pub materials: Vec<Arc<dyn Material>>,
    pub animations: HashMap<String, Vec<Animation>>,
}

impl Model {
    fn update_animation(&mut self, action: &str, dt: std::time::Duration) {
        if let Some(animations) = self.animations.get(action) {
            for animation in animations {
                if let Ok(transform) = animation.get_transform(dt.as_secs_f32()) {
                    self.meshes[animation.mesh_id].update_transform(transform);
                }
            }
        } else {
            println!("Error: Unsupport action: {:?}", action);
        }
    }
}

#[allow(unused)]
pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        queue: &wgpu::Queue,
        mesh: &'a Mesh,
        material: Arc<dyn Material>,
        camera_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        queue: &wgpu::Queue,
        mesh: &'a Mesh,
        material: Arc<dyn Material>,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_model(
        &mut self,
        queue: &wgpu::Queue,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        queue: &wgpu::Queue,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a> {
    fn draw_mesh(
        &mut self,
        queue: &wgpu::Queue,
        mesh: &'b Mesh,
        material: Arc<dyn Material>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(queue, mesh, material, 0..1, camera_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        queue: &wgpu::Queue,
        mesh: &'b Mesh,
        material: Arc<dyn Material>,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        if let Some(transform) = &mesh.uniform_transform {
            queue.write_buffer(transform.get_buffer().unwrap(), 0, transform.as_bytes());
            self.set_bind_group(2, transform.get_bind_group().unwrap(), &[]);
        }

        self.set_bind_group(0, material.get_bind_group(), &[]);
        // self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(
        &mut self,
        queue: &wgpu::Queue,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_model_instanced(queue, model, 0..1, camera_bind_group);
    }

    fn draw_model_instanced(
        &mut self,
        queue: &wgpu::Queue,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in model.meshes.iter() {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(
                queue,
                &mesh,
                material.clone(),
                instances.clone(),
                camera_bind_group,
            );
        }
    }
}
