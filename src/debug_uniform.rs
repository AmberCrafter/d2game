use cgmath::{Deg, InnerSpace, Matrix, SquareMatrix};
use wgpu::{Device, util::DeviceExt};

type Pos3 = cgmath::Point3<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Mat4 = cgmath::Matrix4<f32>;

#[derive(Debug)]
pub struct DebugStorage {
    data: Mat4,
    pub buffer: Option<wgpu::Buffer>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl DebugStorage {
    pub fn new() -> Self {
        Self {
            data: Mat4::identity(),
            buffer: None,
            bind_group_layout: None,
            bind_group: None,
        }
    }

    pub fn as_bytes<'a>(&'a self) -> &'a [u8] {
        unsafe { &(*(self.data.as_ptr() as *const [u8; core::mem::size_of::<Mat4>()])) }
    }

    pub fn show(&self) {
        println!("[Debug] {:?}", self.data);
    }

    pub fn setup(&mut self, device: &Device) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Debug storage buffer"),
            contents: self.as_bytes(),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Debug storage bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Debug storage bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        self.buffer.replace(buffer);
        self.bind_group_layout.replace(bind_group_layout);
        self.bind_group.replace(bind_group);
    }
}
