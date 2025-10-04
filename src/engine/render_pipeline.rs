use std::collections::HashMap;

use crate::engine::{
    bindgroup::BindGroupInfo, config::GraphConfig, shader::ShaderInfo, texture::Texture,
    vertex::VertexBufferInfo,
};

struct RenderPipelineInfoInner {
    layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

pub struct RenderPipelineInfo {
    pub map: HashMap<String, RenderPipelineInfoInner>,
}

// impl<'a, 'b> RenderPipelineInfo<'a, 'b> {
impl RenderPipelineInfo {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn setup(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,

        graph_config: &GraphConfig,
        shader: &ShaderInfo,
        vertex_buffer_info: &VertexBufferInfo,
        bind_group_info: &BindGroupInfo,
    ) -> anyhow::Result<()> {
        for pl in &graph_config.pipelines {
            let nametag = pl.0;

            let mut bind_group_layouts = Vec::new();
            for label in &pl.1.bind_group_layouts {
                bind_group_layouts.push(bind_group_info.get(label).unwrap());
            }

            let mut vertex_buffer_layouts = Vec::new();
            for layout in &pl.1.vertex_buffer_layouts {
                vertex_buffer_layouts.push(vertex_buffer_info.get_desc(&layout).unwrap())
            }

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(nametag),
                bind_group_layouts: &bind_group_layouts.as_slice(),
                push_constant_ranges: &[],
            });

            let shader_config = graph_config.resources.shaders.get(&pl.1.shader).unwrap();

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(nametag),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader.map.get(&pl.1.shader).unwrap(),
                    entry_point: Some(&shader_config.vertex_entry),
                    compilation_options: Default::default(),
                    buffers: &vertex_buffer_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader.map.get(&pl.1.shader).unwrap(),
                    entry_point: Some(&shader_config.fragment_entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format.add_srgb_suffix(),
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),

                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                    // unclipped_depth: false,
                    // conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

            let inner = RenderPipelineInfoInner {
                layout: pipeline_layout,
                pipeline: render_pipeline,
            };

            self.map.insert(nametag.to_string(), inner);
        }
        Ok(())
    }

    pub fn get(&self, label: &str) -> Option<&wgpu::RenderPipeline> {
        if let Some(info) = self.map.get(label) {
            Some(&info.pipeline)
        } else {
            None
        }
    }
}
