use wgpu::util::DeviceExt;

use crate::engine::resources;

#[allow(unused)]
#[derive(Debug)]
pub struct Mesh {
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
}

#[allow(unused)]
impl Mesh {
    pub fn new(mesh: &resources::Mesh, device: &wgpu::Device) -> Self {
        let mut primitives = Vec::new();
        for (index, primative) in mesh.primitives.iter().enumerate() {
            let label = format!(
                "{:?}#{:}",
                mesh.name.as_deref().unwrap_or("UnnamedMesh"),
                index
            );

            let primative = Primitive::new(primative, &label, device);
            primitives.push(primative);
        }

        Self {
            name: mesh.name.clone(),
            primitives,
        }
    }
}

#[derive(Debug)]
pub struct Primitive {
    pub positions: wgpu::Buffer,
    pub tex_coords: wgpu::Buffer,
    pub normals: wgpu::Buffer,
    // pub tangents: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub indices_num: u32,
    pub material_index: usize,
}

impl Primitive {
    pub const POSITION_LOCATION: u32 = 0;
    pub const TEX_COORDS_LOCATION: u32 = 1;
    pub const NORMAL_LOCATION: u32 = 2;

    pub fn new(primative: &resources::Primitive, label: &str, device: &wgpu::Device) -> Self {
        let positions = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Position buffer: {label}")),
            contents: bytemuck::cast_slice(&primative.positions),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let tex_coords = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Tex coordinate buffer: {label}")),
            contents: bytemuck::cast_slice(&primative.tex_coords),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let normals = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Normal buffer: {label}")),
            contents: bytemuck::cast_slice(&primative.normals),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // let tangents = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some(&format!("Tangents buffer: {label}")),
        //     contents: bytemuck::cast_slice(&primative.tangents),
        //     usage: wgpu::BufferUsages::VERTEX
        // });

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Indices buffer: {label}")),
            contents: bytemuck::cast_slice(&primative.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            positions,
            tex_coords,
            normals,
            indices,
            indices_num: primative.indices.len() as u32,
            material_index: primative.material,
        }
    }
}
