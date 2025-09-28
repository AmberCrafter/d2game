use std::collections::HashMap;

use wgpu::Device;

use crate::engine::config::{BindGroupEntryType, BindGroupVisibilty, GraphConfig};

#[derive(Debug, Clone)]
struct BindGroupInfoInner {
    layout: wgpu::BindGroupLayout,
}

#[derive(Debug, Clone)]
pub struct BindGroupInfo {
    infos: HashMap<String, BindGroupInfoInner>,
}

impl BindGroupInfo {
    pub fn new() -> Self {
        Self {
            infos: HashMap::new(),
        }
    }

    #[inline]
    fn get_visibility(visibility: &BindGroupVisibilty) -> wgpu::ShaderStages {
        match visibility {
            BindGroupVisibilty::Vertex => wgpu::ShaderStages::VERTEX,
            BindGroupVisibilty::Fragment => wgpu::ShaderStages::FRAGMENT,
            BindGroupVisibilty::ALL => wgpu::ShaderStages::all(),
        }
    }

    pub fn setup(&mut self, config: &GraphConfig, device: &Device) {
        // let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Camera buffer"),
        //     contents: self.uniform.as_bytes(),
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        // });

        for bg in &config.resources.bindgroups {
            let mut entries = Vec::new();
            for entry in &bg.1.entries {
                match entry.ty {
                    BindGroupEntryType::Texture => {
                        let entry = wgpu::BindGroupLayoutEntry {
                            binding: entry.binding as u32,
                            visibility: Self::get_visibility(&entry.Visibility),
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        };
                        entries.push(entry);
                    }
                    BindGroupEntryType::Sampler => {
                        let entry = wgpu::BindGroupLayoutEntry {
                            binding: entry.binding as u32,
                            visibility: Self::get_visibility(&entry.Visibility),
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        };
                        entries.push(entry);
                    }
                    BindGroupEntryType::Uniform => {
                        let entry = wgpu::BindGroupLayoutEntry {
                            binding: entry.binding as u32,
                            visibility: Self::get_visibility(&entry.Visibility),
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        };
                        entries.push(entry);
                    }
                    BindGroupEntryType::Storage => {
                        unimplemented!()
                    }
                    BindGroupEntryType::StorageRo => {
                        unimplemented!()
                    }
                }
            }

            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(bg.0),
                entries: &entries,
            });

            // let mut usage = wgpu::BufferUsages::empty();

            // for u in &bg.1.usgae {
            //     let flag = match u {
            //         BufferUsage::COPY_SRC => wgpu::BufferUsages::COPY_SRC,
            //         BufferUsage::COPY_DST => wgpu::BufferUsages::COPY_DST,
            //         BufferUsage::VERTEX => wgpu::BufferUsages::VERTEX,
            //         BufferUsage::INDEX => wgpu::BufferUsages::INDEX,
            //         BufferUsage::UNIFORM => wgpu::BufferUsages::UNIFORM,
            //         BufferUsage::STORAGE => wgpu::BufferUsages::STORAGE,
            //     };
            //     usage |= flag;
            // }

            let info = BindGroupInfoInner { layout };
            self.infos.insert(bg.0.to_string(), info);
        }

        // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Camera bind group"),
        //     layout: &bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: buffer.as_entire_binding(),
        //     }],
        // });
    }

    pub fn get(&self, label: &str) -> Option<&BindGroupInfoInner> {
        self.infos.get(label)
    }
}
