use crate::engine::{camera::Camera, model::Model};

pub struct Scene {
    pub name: Option<String>,
    pub models: Vec<Model>,
    pub camera: Camera,
}

#[allow(unused)]
impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            name: None,
            models: Vec::new(),
            camera,
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name.replace(name.to_string());
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn clear_model(&mut self) {
        self.models.clear();
    }

    pub fn render<'a>(&self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(
            Camera::BIND_GROUP_INDEX,
            &self.camera.info.bind_group,
            &[],
        );

        for model in self.models.iter() {
            model.render(render_pass);
        }
    }
}
