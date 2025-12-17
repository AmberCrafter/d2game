mod material;
mod mesh;
mod node;
mod scene;
mod texture;

use image::buffer::ConvertBuffer;

use crate::engine::resources::{
    material::Material, mesh::Mesh, node::Node, scene::Scene, texture::Texture,
};

#[cfg(target_arch = "wasm32")]
pub type Image = web_sys::ImageBitmap;

#[cfg(not(target_arch = "wasm32"))]
pub type Image = image::RgbaImage;

pub type Buffer = Vec<u8>;

struct Model {
    scenes: Vec<Scene>,
    nodes: Vec<Node>,
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
    textures: Vec<Texture>,
    buffers: Vec<Buffer>,
    images: Vec<Image>,

    default_scene_index: usize,
}

// TODO: support wasm
impl Model {
    pub fn load_gltf<P: AsRef<std::path::Path> + std::fmt::Debug>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let (doc, buffers, images) = gltf::import(path)?;
        let buffers = buffers
            .into_iter()
            .map(|buffer| buffer.0)
            .collect::<Vec<Buffer>>();

        let mut imgs = Vec::new();
        for img in images {
            let img = match img.format {
                gltf::image::Format::R8G8B8A8 => {
                    image::RgbaImage::from_raw(img.width, img.height, img.pixels)
                        .ok_or_else(|| format!("Image convertion failed"))
                }
                gltf::image::Format::R8G8B8 => {
                    if let Some(rgb) = image::RgbaImage::from_raw(img.width, img.height, img.pixels) {
                        Ok(rgb.convert())
                    } else {
                        Err(format!("Image convertion failed"))
                    }
                }
                fmt => Err(format!("Unsupport image format: {:?}", fmt)),
            };
            if let Ok(img) = img {
                imgs.push(img);
            }
        }

        let scenes = doc
            .scenes()
            .map(|val| val.into())
            .collect::<Vec<_>>();

        let nodes = doc
            .nodes()
            .map(|val| val.into())
            .collect::<Vec<_>>();

        let mut meshes = Vec::new();
        for mesh in doc.meshes() {
            if let Ok(mesh) = Mesh::parse(&mesh, &buffers) {
                meshes.push(mesh);
            }
        }

        let materials = doc
            .materials()
            .map(|val| val.into())
            .collect::<Vec<_>>();

        let textures = doc
            .textures()
            .map(|val| val.into())
            .collect::<Vec<_>>();
        
        let default_scene_index = doc.default_scene().map(|sence| sence.index()).unwrap_or(0);

        Ok(Self {
            scenes,
            nodes,
            meshes,
            materials,
            textures,
            buffers,
            images: imgs,
            default_scene_index,
        })
    }
}
