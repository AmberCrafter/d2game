use wgpu::util::DeviceExt;

use crate::engine::{config::BindGroupConfig, resources, texture::Texture};

#[allow(unused)]
#[derive(Debug)]
pub struct Material {
    pub name: Option<String>,
    // pub factors: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

#[allow(unused)]
enum MaterialEntry<'a> {
    Buffer(wgpu::Buffer),
    TextureView(&'a wgpu::TextureView),
    TextureSmapler(&'a wgpu::Sampler),
}

#[allow(unused)]
impl<'a> MaterialEntry<'a> {
    // const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    fn to_buffer(
        device: &wgpu::Device,
        label: Option<&str>,
        data: &[u8],
        usage: wgpu::BufferUsages,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: label,
            contents: data,
            usage,
        });

        Self::Buffer(buffer)
    }

    fn to_texture_view(texture: &'a Texture) -> Self {
        Self::TextureView(&texture.view)
    }

    fn to_texture_sampler(texture: &'a Texture) -> Self {
        Self::TextureSmapler(&texture.sampler)
    }
}

// GLTF Material
#[allow(unused)]
impl Material {
    // TODO: from config.pipeline
    pub const BIND_GROUP_INDEX: u32 = 0;

    pub fn new(
        material: &resources::Material,
        textures: &[Texture],
        device: &wgpu::Device,
        // queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
        config: &BindGroupConfig,
    ) -> Self {
        let mut buffers = Vec::new();

        for entry in config.entries.iter() {
            let mentry = match entry.binding {
                0 => {
                    // base color factor
                    let label = Some(format!("Material base color factor"));
                    let data = bytemuck::cast_slice(&material.base_color_factor);
                    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
                    MaterialEntry::to_buffer(device, label.as_deref(), data, usage)
                }
                1 => {
                    // metallic factor
                    let label = Some(format!("Material metallic factor"));
                    let data = bytemuck::cast_slice(&material.metallic_factor);
                    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
                    MaterialEntry::to_buffer(device, label.as_deref(), data, usage)
                }
                2 => {
                    // roughness factor
                    let label = Some(format!("Material roughness factor"));
                    let data = bytemuck::cast_slice(&material.roughness_factor);
                    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
                    MaterialEntry::to_buffer(device, label.as_deref(), data, usage)
                }
                3 => {
                    // base color texture
                    // let format = Some(wgpu::TextureFormat::Rgba8UnormSrgb);
                    // let label = Some(format!("Material base color texture"));
                    let texture = material
                        .base_color_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_view(texture)
                }
                4 => {
                    // base color sampler
                    // let label = Some(format!("Material base color sampler"));
                    let texture = material
                        .base_color_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_sampler(texture)
                }
                5 => {
                    // metallic roughness texture
                    // let format = Some(wgpu::TextureFormat::Rgba8UnormSrgb);
                    // let label = Some(format!("Material metallic roughness texture"));
                    let texture = material
                        .metallic_roughness_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_view(texture)
                }
                6 => {
                    // metallic roughness sampler
                    // let label = Some(format!("Material metallic roughness sampler"));
                    let texture = material
                        .base_color_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_sampler(texture)
                }
                7 => {
                    // normal texture
                    // let format = Some(wgpu::TextureFormat::Rgba8UnormSrgb);
                    // let label = Some(format!("Material normal texture"));
                    let texture = material
                        .normal_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_view(texture)
                }
                8 => {
                    // normal sampler
                    // let label = Some(format!("Material normal sampler"));
                    let texture = material
                        .normal_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_sampler(texture)
                }
                9 => {
                    // occlusion texture
                    // let format = Some(wgpu::TextureFormat::Rgba8UnormSrgb);
                    // let label = Some(format!("Material occlusion texture"));
                    let texture = material
                        .occlusion_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_view(texture)
                }
                10 => {
                    // occlusion sampler
                    // let label = Some(format!("Material occlusion sampler"));
                    let texture = material
                        .occlusion_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_sampler(texture)
                }
                11 => {
                    // emissive factor
                    let label = Some(format!("Material emissive factor"));
                    let data = bytemuck::cast_slice(&material.emissive_factor);
                    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
                    MaterialEntry::to_buffer(device, label.as_deref(), data, usage)
                }
                12 => {
                    // emissive texture
                    // let format = Some(wgpu::TextureFormat::Rgba8UnormSrgb);
                    // let label = Some(format!("Material emissive texture"));
                    let texture = material
                        .emissive_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_view(texture)
                }
                13 => {
                    // emissive sampler
                    // let label = Some(format!("Material emissive sampler"));
                    let texture = material
                        .emissive_texture_index
                        .map(|idx| &textures[idx])
                        .unwrap();
                    MaterialEntry::to_texture_sampler(texture)
                }

                _ => {
                    unimplemented!()
                }
            };

            buffers.push((entry.binding as u32, mentry));
        }

        let entries = buffers
            .iter()
            .map(|(idx, buffer)| match buffer {
                MaterialEntry::Buffer(buffer) => wgpu::BindGroupEntry {
                    binding: *idx,
                    resource: buffer.as_entire_binding(),
                },
                MaterialEntry::TextureView(view) => wgpu::BindGroupEntry {
                    binding: *idx,
                    resource: wgpu::BindingResource::TextureView(*view),
                },
                MaterialEntry::TextureSmapler(sampler) => wgpu::BindGroupEntry {
                    binding: *idx,
                    resource: wgpu::BindingResource::Sampler(*sampler),
                },
            })
            .collect::<Vec<_>>();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Material bind group")),
            layout: bind_group_layout,
            entries: &entries,
        });

        Self {
            name: material.name.clone(),
            bind_group,
        }
    }
}
