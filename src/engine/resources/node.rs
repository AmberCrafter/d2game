#[derive(Debug)]
pub struct Node {
    pub name: Option<String>,
    pub mesh: Option<usize>,
    pub children: Vec<usize>,
    pub transform: cgmath::Matrix4<f32>,
}

impl From<gltf::scene::Node<'_>> for Node {
    fn from(value: gltf::scene::Node) -> Self {
        let name = value.name().map(|val| val.to_string());

        let mesh = value.mesh().map(|mesh| mesh.index());

        let children = value
            .children()
            .map(|child| child.index())
            .collect::<Vec<usize>>();

        let transform = value.transform().matrix();
        let transform = cgmath::Matrix4::from(transform);

        Self {
            name,
            mesh,
            children,
            transform,
        }
    }
}
