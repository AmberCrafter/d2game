use cgmath::Matrix4;
use wgpu::util::DeviceExt;

const SIZE_MAT4: usize = core::mem::size_of::<cgmath::Matrix4<f32>>();

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: f32,
}

#[allow(unused)]
impl Instance {
    pub fn as_model(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(self.position)
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_scale(self.scale)
    }

    pub fn as_bytes(&self) -> [u8; SIZE_MAT4] {
        // unsafe {
        //     &(*(self.as_model().as_ptr()
        //         as *const [u8; core::mem::size_of::<cgmath::Matrix4<f32>>()]))
        // }
        unsafe { core::mem::transmute::<cgmath::Matrix4<f32>, [u8; SIZE_MAT4]>(self.as_model()) }
    }
}

pub fn to_instance_buffer(device: &wgpu::Device, instances: &[Instance]) -> wgpu::Buffer {
    let instance_data = instances
        .iter()
        .flat_map(|val| {
            let model = val.as_model();
            unsafe { core::mem::transmute::<Matrix4<f32>, [u8; SIZE_MAT4]>(model) }
        })
        .collect::<Vec<u8>>();

    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instances buffer"),
        contents: instance_data.as_slice(),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    instance_buffer
}
