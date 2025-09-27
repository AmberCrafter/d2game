use std::io::Read;

pub struct ShaderInfo {
    pub vertex: Option<wgpu::ShaderModule>,
    pub fragment: Option<wgpu::ShaderModule>,
}

impl ShaderInfo {
    pub fn new() -> Self {
        Self {
            vertex: None,
            fragment: None,
        }
    }

    pub fn setup_vertex_shader(&mut self, device: &wgpu::Device, path: &str) -> anyhow::Result<()> {
        let shader = Self::load_shader(device, Some("Vertex shader"), path)?;
        self.vertex.replace(shader);
        Ok(())
    }

    pub fn setup_fragment_shader(&mut self, device: &wgpu::Device, path: &str) -> anyhow::Result<()> {
        let shader = Self::load_shader(device, Some("Fragment shader"), path)?;
        self.fragment.replace(shader);
        Ok(())
    }

    fn load_shader(
        device: &wgpu::Device,
        label: Option<&str>,
        path: &str,
    ) -> anyhow::Result<wgpu::ShaderModule> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(contents.into()),
        }))
    }
}
