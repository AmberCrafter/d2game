use cgmath::Matrix;

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

#[allow(unused)]
impl Instance {
    pub fn as_model(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            &(*(self.as_model().as_ptr()
                as *const [u8; core::mem::size_of::<cgmath::Matrix4<f32>>()]))
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: core::mem::size_of::<cgmath::Matrix4<f32>>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    offset: core::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    offset: core::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 7,
                },
                wgpu::VertexAttribute {
                    offset: core::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 8,
                },
            ],
        }
    }
}
