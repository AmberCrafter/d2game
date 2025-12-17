#[derive(Debug)]
pub struct Scene {
    pub name: Option<String>,
    pub nodes: Vec<usize>,
}

impl From<gltf::scene::Scene<'_>> for Scene {
    fn from(value: gltf::scene::Scene) -> Self {
        let name = value.name().map(|val| val.to_string());
        let nodes = value
            .nodes()
            .map(|val| val.index())
            .collect::<Vec<usize>>();
        Self { name, nodes }
    }
}