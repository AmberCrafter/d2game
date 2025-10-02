use std::ops::Range;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Material {
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

#[allow(unused)]
#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

#[allow(unused)]
pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_model(&mut self, model: &'a Model, camera_bind_group: &'a wgpu::BindGroup);
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(&mut self, model: &'b Model, camera_bind_group: &'b wgpu::BindGroup) {
        self.draw_model_instanced(model, 0..1, camera_bind_group);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
    ) {
        // println!("[Debug] {:?}({:?}) {:?}", file!(), line!(), model);
        
        for mesh in model.meshes.iter() {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(&mesh, material, instances.clone(), camera_bind_group);
        }
    }
}
