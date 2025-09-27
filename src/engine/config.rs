use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ShaderConfig {
    filename: String,
    vertex_entry: String,
    fragment_entry: String,
}

#[derive(Debug, Deserialize)]
enum BufferUsage {
    COPY_SRC,
    COPY_DST,
    VERTEX,
    INDEX,
    UNIFORM,
    STORAGE,
}

#[derive(Debug, Deserialize)]
enum VertexBufferLayoutFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
}

#[derive(Debug, Deserialize)]
struct VertexBufferLayout {
    location: usize,
    format: VertexBufferLayoutFormat,
}

#[derive(Debug, Deserialize)]
struct VertexBufferConfig {
    layouts: Vec<VertexBufferLayout>,
    usgae: Vec<BufferUsage>,
}

#[derive(Debug, Deserialize)]
enum BindGroupVisibilty {
    Vertex,
    Fragment,
    ALL,
}

#[derive(Debug, Deserialize)]
enum BindGroupEntryType {
    Texture,
    Sampler,

    // Buffer subtype
    Uniform,
    Storage,
    StorageRo, // read-only
}

#[derive(Debug, Deserialize)]
struct BindGroupEntry {
    binding: usize,
    ty: BindGroupEntryType,
    Visibility: BindGroupVisibilty,
}

#[derive(Debug, Deserialize)]
struct BindGroupConfig {
    entries: Vec<BindGroupEntry>,
    usgae: Vec<BufferUsage>,
}

#[derive(Debug, Deserialize)]
struct ResourceConfig {
    shaders: HashMap<String, ShaderConfig>,
    vertexbuffers: HashMap<String, VertexBufferConfig>,
    bindgroups: HashMap<String, BindGroupConfig>,
}

#[derive(Debug, Deserialize)]
struct PipelineConfig {
    shader: String,
    depth_texture: bool,
    vertex_buffer_layouts: Vec<String>,
    bind_group_layout: String,
}

#[derive(Debug, Deserialize)]
struct GraphConfig {
    version: String,
    name: String,
    resources: ResourceConfig,
    pipelines: HashMap<String, PipelineConfig>,
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
usgae = [ "VERTEX" ]

[resources.vertexbuffers.instance]
layouts = [
    {location = 5, format = "Float32x4"},
    {location = 6, format = "Float32x4"},
    {location = 7, format = "Float32x4"},
    {location = 8, format = "Float32x4"},
]
usgae = [ "INDEX" ]

[resources.bindgroups.camera]
entries = [
    {binding = 0, ty = "Uniform", Visibility = "Vertex" },
]
usgae = [ "UNIFORM", "COPY_DST" ]

[resources.bindgroups.texture]
entries = [
    {binding = 0, ty = "Texture", Visibility = "Fragment" },
    {binding = 1, ty = "Sampler", Visibility = "Fragment" },
]
usgae = [ "UNIFORM", "COPY_DST" ]

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
