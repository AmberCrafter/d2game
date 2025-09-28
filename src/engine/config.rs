use std::{collections::HashMap, fs, io::{BufReader, Read}};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ShaderConfig {
    pub filename: String,
    pub vertex_entry: String,
    pub fragment_entry: String,
}

#[derive(Debug, Deserialize)]
pub enum VertexBufferLayoutFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
}

#[derive(Debug, Deserialize)]
pub struct VertexBufferLayout {
    pub location: usize,
    pub format: VertexBufferLayoutFormat,
}

#[derive(Debug, Deserialize)]
pub enum VertexStepMode {
    VERTEX,
    INSTANCE,
}

#[derive(Debug, Deserialize)]
pub struct VertexBufferConfig {
    pub layouts: Vec<VertexBufferLayout>,
    pub mode: VertexStepMode,
}

#[derive(Debug, Deserialize)]
pub enum BindGroupVisibilty {
    Vertex,
    Fragment,
    ALL,
}

#[derive(Debug, Deserialize)]
pub enum BindGroupEntryType {
    Texture,
    Sampler,

    // Buffer subtype
    Uniform,
    Storage,
    StorageRo, // read-only
}

#[derive(Debug, Deserialize)]
pub struct BindGroupEntry {
    pub binding: usize,
    pub ty: BindGroupEntryType,
    pub Visibility: BindGroupVisibilty,
}

#[derive(Debug, Deserialize)]
pub struct BindGroupConfig {
    pub entries: Vec<BindGroupEntry>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceConfig {
    pub shaders: HashMap<String, ShaderConfig>,
    pub vertexbuffers: HashMap<String, VertexBufferConfig>,
    pub bindgroups: HashMap<String, BindGroupConfig>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineConfig {
    pub shader: String,
    pub depth_texture: bool,
    pub vertex_buffer_layouts: Vec<String>,
    pub bind_group_layout: String,
}

#[derive(Debug, Deserialize)]
pub struct GraphConfig {
    pub version: String,
    pub name: String,
    pub resources: ResourceConfig,
    pub pipelines: HashMap<String, PipelineConfig>,
}

impl GraphConfig {
    pub fn new(path: &str) -> Self {
        let file = fs::File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer).unwrap();
        let config: GraphConfig = toml::from_str(&buffer).unwrap();
        config
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TESTCONFIG: &str = r#"
version = "1.0.0"
name = "MyGame grapher"

[resources]
[resources.shaders.backgound]
filename = "backgound.wgsl"
vertex_entry = "vs_main"
fragment_entry = "fs_main"

[resources.shaders.item]
filename = "item.wgsl"
vertex_entry = "vs_main"
fragment_entry = "fs_main"

[resources.vertexbuffers.vertex]
layouts = [
    {location = 0, format = "Float32x3"},
    {location = 1, format = "Float32x2"},
    {location = 2, format = "Float32x3"},
]
mode = "VERTEX"

[resources.vertexbuffers.instance]
layouts = [
    {location = 5, format = "Float32x4"},
    {location = 6, format = "Float32x4"},
    {location = 7, format = "Float32x4"},
    {location = 8, format = "Float32x4"},
]
mode = "INSTANCE"

[resources.bindgroups.camera]
entries = [
    {binding = 0, ty = "Uniform", Visibility = "Vertex" },
]

[resources.bindgroups.texture]
entries = [
    {binding = 0, ty = "Texture", Visibility = "Fragment" },
    {binding = 1, ty = "Sampler", Visibility = "Fragment" },
]

[pipelines]
[pipelines.backgound]
shader = "backgound"
depth_texture = true
vertex_buffer_layouts = [
    "vertex",
]
bind_group_layout = "texture"

[pipelines.item]
shader = "item"
depth_texture = true
vertex_buffer_layouts = [
    "vertex",
    "instance",
]
bind_group_layout = "texture"
    "#;

    #[test]
    fn case1() {
        let config: GraphConfig = toml::from_str(TESTCONFIG).unwrap();
        println!("{:#?}", config);
    }
}
