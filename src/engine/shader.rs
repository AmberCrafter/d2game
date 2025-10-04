use std::{collections::HashMap, io::Read};

use crate::engine::config::GraphConfig;

pub struct ShaderInfo {
    pub vertex: Option<wgpu::ShaderModule>,
    pub fragment: Option<wgpu::ShaderModule>,
    pub map: HashMap<String, wgpu::ShaderModule>,
}

impl ShaderInfo {
    pub fn new() -> Self {
        Self {
            vertex: None,
            fragment: None,
            map: HashMap::new(),
        }
    }

    pub fn load_config(&mut self, device: &wgpu::Device, config: &GraphConfig) {
        for shader in &config.resources.shaders {
            // println!(
            //     "[Debug] {:?}({:?}) {:?}",
            //     file!(),
            //     line!(),
            //     shader.1.filename
            // );
            let module = Self::load_shader(
                device,
                Some(&shader.0),
                &format!("./shader/{:}", shader.1.filename),
            )
            .unwrap();
            self.map.insert(shader.0.clone(), module);
        }
    }

    pub fn setup_vertex_shader(&mut self, device: &wgpu::Device, path: &str) -> anyhow::Result<()> {
        let shader = Self::load_shader(device, Some("Vertex shader"), path)?;
        self.vertex.replace(shader);
        Ok(())
    }

    pub fn setup_fragment_shader(
        &mut self,
        device: &wgpu::Device,
        path: &str,
    ) -> anyhow::Result<()> {
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
