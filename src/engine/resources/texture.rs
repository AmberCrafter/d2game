use gltf::texture::{MagFilter, MinFilter, WrappingMode};

#[allow(unused)]
#[derive(Debug)]
pub struct Sampler {
    pub mag_filter: Option<MagFilter>,
    pub min_filter: Option<MinFilter>,
    pub wrap_s: WrappingMode,
    pub wrap_t: WrappingMode,
}

impl From<gltf::texture::Sampler<'_>> for Sampler {
    fn from(value: gltf::texture::Sampler<'_>) -> Self {
        Sampler {
            mag_filter: value.mag_filter(),
            min_filter: value.min_filter(),
            wrap_s: value.wrap_s(),
            wrap_t: value.wrap_t(),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Texture {
    pub name: Option<String>,
    pub image_index: usize,
    pub sampler: Sampler,
}

impl From<gltf::texture::Texture<'_>> for Texture {
    fn from(value: gltf::texture::Texture<'_>) -> Self {
        let name = value.name().map(|val| val.to_string());
    
        Self {
            name,
            image_index: value.source().index(),
            sampler: value.sampler().into(),
        }
    }
}
