use winit::window::ResizeDirection;

#[derive(Debug)]
pub struct Mesh {
    pub name: Option<String>,
    pub primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn parse(
        mesh: &gltf::mesh::Mesh,
        buffers: &Vec<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let name = mesh.name().map(|val| val.to_string());
        let mut primitives = Vec::new();
        for primative in mesh.primitives() {
            primitives.push(Primitive::parse(&primative, buffers)?);
        }

        Ok(Self { name, primitives })
    }
}

#[derive(Debug)]
pub struct Primitive {
    pub positions: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub material: usize,
}

impl Primitive {
    pub fn parse(
        primative: &gltf::mesh::Primitive<'_>,
        buffers: &Vec<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let reader = primative.reader(|buffer| Some(&buffers[buffer.index()]));

        let positions = reader
            .read_positions()
            .map(|iter| iter.collect::<Vec<[f32; 3]>>())
            .ok_or_else(|| format!("No position."))?;

        // Support the case without texture
        let tex_coords = reader
            .read_tex_coords(0)
            .map(|iter| iter.into_f32().collect::<Vec<[f32; 2]>>())
            .unwrap_or_else(|| {
                // format!("No tex_coord. using default");
                vec![[0.0, 0.0]; positions.len()]
            });

        let normals = reader
            .read_normals()
            .map(|iter| iter.collect::<Vec<[f32; 3]>>())
            .ok_or_else(|| format!("No normal."))?;

        let indices = reader
            .read_indices()
            .map(|iter| iter.into_u32().collect::<Vec<u32>>())
            .ok_or_else(|| format!("No indices."))?;

        let material = primative.material().index().unwrap_or(0);

        Ok(Self {
            positions,
            tex_coords,
            normals,
            indices,
            material,
        })
    }
}
