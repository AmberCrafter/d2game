use crate::{shader::ShaderInfo, texture::Texture};

pub struct RenderPipelineInfo /*<'a, 'b>*/ {
    // bind_grop_layouts: Vec<&'a wgpu::BindGroupLayout>,
    // vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'b>>,
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    pub layout: Option<wgpu::PipelineLayout>,
}

// impl<'a, 'b> RenderPipelineInfo<'a, 'b> {
impl RenderPipelineInfo {
    pub fn new() -> Self {
        Self {
            // bind_grop_layouts: Vec::new(),
            // vertex_buffer_layouts: Vec::new(),
            render_pipeline: None,
            layout: None,
        }
    }

    // pub fn add_bind_group_layout(
    //     &mut self,
    //     layout: &'a wgpu::BindGroupLayout,
    // ) -> anyhow::Result<()> {
    //     self.bind_grop_layouts.push(layout);
    //     Ok(())
    // }

    // pub fn clear_bind_group_layout(&mut self) {
    //     self.bind_grop_layouts.clear();
    // }

    // pub fn add_vertex_buffer_layout(
    //     &mut self,
    //     layout: wgpu::VertexBufferLayout<'b>,
    // ) -> anyhow::Result<()> {
    //     self.vertex_buffer_layouts.push(layout);
    //     Ok(())
    // }

    // pub fn clear_vertex_buffer_layout(&mut self) {
    //     self.vertex_buffer_layouts.clear();
    // }

    pub fn setup<'a>(
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shader: &ShaderInfo,
        vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'a>>,
        bind_group_layouts: Option<Vec<&wgpu::BindGroupLayout>>,
    ) -> anyhow::Result<()> {
        let pipeline_layout = if let Some(bind_group_layouts) = bind_group_layouts {
            Some(
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render pipeline layout"),
                    // bind_group_layouts: &self.bind_grop_layouts.as_slice(),
                    bind_group_layouts: &bind_group_layouts.as_slice(),
                    push_constant_ranges: &[],
                }),
            )
        } else {
            None
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: pipeline_layout.as_ref(),
            vertex: wgpu::VertexState {
                module: shader.vertex.as_ref().unwrap(),
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &vertex_buffer_layouts.as_slice(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader.fragment.as_ref().unwrap(),
                entry_point: Some("fs_main"),
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

        self.render_pipeline.replace(render_pipeline);
        self.layout = pipeline_layout;

        Ok(())
    }
}
