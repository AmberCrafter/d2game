use wgpu::util::DeviceExt;

#[allow(unused)]
#[derive(Debug)]
pub struct Buffer<T> {
    data: T,
    buffer: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    group: wgpu::BindGroup,
}

#[allow(unused)]
impl<T> Buffer<T> {
    pub fn new(
        data: T,
        buffer: wgpu::Buffer,
        layout: wgpu::BindGroupLayout,
        group: wgpu::BindGroup,
    ) -> Self {
        Buffer {
            data,
            buffer,
            layout,
            group,
        }
    }

    pub fn set_data(&mut self, data: T) -> anyhow::Result<()> {
        self.data = data;
        Ok(())
    }

    pub fn get_data(&self) -> Option<&T> {
        Some(&self.data)
    }

    pub fn get_data_mut(&mut self) -> Option<&mut T> {
        Some(&mut self.data)
    }

    pub fn get_buffer(&self) -> Option<&wgpu::Buffer> {
        Some(&self.buffer)
    }

    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        Some(&self.group)
    }

    pub fn as_bytes(&self) -> &[u8] {
        as_u8_slice(&self.data)
    }
}

pub fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(p as *const T as *const u8, core::mem::size_of::<T>()) }
}

pub fn setup_uniform<T, const C: usize>(
    device: &wgpu::Device,
    name: &str,
    data: T,
    binding: u32,
    visibility: wgpu::ShaderStages,
) -> Buffer<T>
where
    T: Sized,
{
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} uniform buffer", name)),
        contents: as_u8_slice(&data),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(&format!("{} uniform bind group layout", name)),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("{} uniform bind group", name)),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding,
            resource: buffer.as_entire_binding(),
        }],
    });

    Buffer::new(data, buffer, bind_group_layout, bind_group)
}
