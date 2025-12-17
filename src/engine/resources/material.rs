pub struct Material {
    pub name: Option<String>,
    pub base_color_factor: [f32; 4],
    pub base_color_texture_index: Option<usize>,
    pub normal_texture_index: Option<usize>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture_index: Option<usize>,
    pub occlusion_texture_index: Option<usize>,
    pub emissive_factor: [f32; 3],
    pub emissive_texture_index: Option<usize>,
}

impl From<gltf::material::Material<'_>> for Material {
    fn from(value: gltf::material::Material<'_>) -> Self {
        let name = value.name().map(|val| val.to_string());
        let pbr = value.pbr_metallic_roughness();

        let base_color_factor = pbr.base_color_factor();
        let base_color_texture_index = pbr.base_color_texture().map(|val| val.texture().index());
        let normal_texture_index = value.normal_texture().map(|val| val.texture().index());
        let metallic_factor = pbr.metallic_factor();
        let roughness_factor = pbr.roughness_factor();
        let metallic_roughness_texture_index = pbr
            .metallic_roughness_texture()
            .map(|val| val.texture().index());
        let occlusion_texture_index = value.occlusion_texture().map(|val| val.texture().index());
        let emissive_factor = value.emissive_factor();
        let emissive_texture_index = value.emissive_texture().map(|val| val.texture().index());

        Self {
            name,
            base_color_factor,
            base_color_texture_index,
            normal_texture_index,
            metallic_factor,
            roughness_factor,
            metallic_roughness_texture_index,
            occlusion_texture_index,
            emissive_factor,
            emissive_texture_index,
        }
    }
}
