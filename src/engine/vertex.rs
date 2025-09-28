use std::collections::HashMap;

use wgpu::VertexAttribute;

use crate::engine::config::{GraphConfig, VertexBufferLayoutFormat, VertexStepMode};

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImageVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl Vertex for ImageVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: core::mem::size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
struct VertexAttributeLayoutInfo {
    stride: u64,
    mode: wgpu::VertexStepMode,
    attrs: Vec<VertexAttribute>,
}
#[derive(Debug, Clone)]
pub struct VertexBufferInfo {
    infos: HashMap<String, VertexAttributeLayoutInfo>,
}

impl VertexBufferInfo {
    pub fn new() -> Self {
        Self {
            infos: HashMap::new(),
        }
    }

    pub fn setup_config(&mut self, config: &GraphConfig) {
        for vb in &config.resources.vertexbuffers {
            let mut stride = 0;
            let mut attrs = Vec::new();
            for layout in &vb.1.layouts {
                match layout.format {
                    VertexBufferLayoutFormat::Float32 => {
                        attrs.push(wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: stride,
                            shader_location: layout.location as u32,
                        });
                        stride += core::mem::size_of::<[f32; 1]>() as u64;
                    }
                    VertexBufferLayoutFormat::Float32x2 => {
                        attrs.push(wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: stride,
                            shader_location: layout.location as u32,
                        });
                        stride += core::mem::size_of::<[f32; 2]>() as u64;
                    }
                    VertexBufferLayoutFormat::Float32x3 => {
                        attrs.push(wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: stride,
                            shader_location: layout.location as u32,
                        });
                        stride += core::mem::size_of::<[f32; 3]>() as u64;
                    }
                    VertexBufferLayoutFormat::Float32x4 => {
                        attrs.push(wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: stride,
                            shader_location: layout.location as u32,
                        });
                        stride += core::mem::size_of::<[f32; 4]>() as u64;
                    }
                }
            }

            let mode = match vb.1.mode {
                VertexStepMode::VERTEX => wgpu::VertexStepMode::Vertex,
                VertexStepMode::INSTANCE => wgpu::VertexStepMode::Instance,
                _ => {
                    unimplemented!()
                }
            };

            self.infos.insert(
                vb.0.to_string(),
                VertexAttributeLayoutInfo {
                    stride,
                    mode,
                    attrs,
                },
            );
        }
    }

    pub fn get_desc<'a>(&'a mut self, name: &str) -> Option<wgpu::VertexBufferLayout<'a>> {
        if let Some(info) = self.infos.get(name) {
            Some(wgpu::VertexBufferLayout {
                array_stride: info.stride,
                step_mode: info.mode,
                attributes: &info.attrs,
            })
        } else {
            None
        }
    }
}
