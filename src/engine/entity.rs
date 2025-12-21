use crate::engine::resources;

#[allow(unused)]
pub struct Entity {
    pub name: Option<String>,
    pub mesh_index: Option<usize>,
    pub children: Vec<usize>,
    pub transform: cgmath::Matrix4<f32>,
}

impl From<resources::Node> for Entity {
    fn from(node: resources::Node) -> Self {
        Self {
            name: node.name,
            mesh_index: node.mesh,
            children: node.children,
            transform: node.transform,
        }
    }
}
