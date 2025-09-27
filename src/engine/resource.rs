use std::{io::Cursor, path::PathBuf};
// use cfg_if::cfg_if;
use crate::engine::{model, texture};
// use tokio::io::AsyncBufReadExt;
use wgpu::util::DeviceExt;

// #[cfg(target_arch = "wasm32")]
// fn format_url(file_name: &str) -> reqwest::Url {
//     let window = web_sys::window().unwrap();
//     let location = window.location();
//     let base = reqwest::Url::parse(&format!(
//         "{}/{}/",
//         location.origin().unwrap(),
//         option_env!("RES_PATH").unwrap_or("res"),
//     ))
//     .unwrap();
//     base.join(file_name).unwrap()
// }

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    // cfg_if! {
    //     if #[cfg(target_arch = "wasm32")] {
    //         let url = format_url(file_name);
    //         let txt = reqwest::get(url)
    //             .await?
    //             .text()
    //             .await?;
    //     } else {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let txt = std::fs::read_to_string(path)?;
    //     }
    // }

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    // cfg_if! {
    //     if #[cfg(target_arch = "wasm32")] {
    //         let url = format_url(file_name);
    //         let data = reqwest::get(url)
    //             .await?
    //             .bytes()
    //             .await?
    //             .to_vec();
    //     } else {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let data = std::fs::read(path)?;
    //     }
    // }

    Ok(data)
}

pub async fn load_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    file_name: &str,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    Ok(texture::Texture::load_texture_from_bytes(
        device,
        queue,
        Some(file_name),
        &data,
    )?)
}

pub async fn load_model(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    file_name: &str,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = tokio::io::BufReader::new(obj_cursor);

    let material_loader = |p: PathBuf| async move {
        let mat_text = load_string(p.as_os_str().to_str().unwrap()).await.unwrap();
        tobj::tokio::load_mtl_buf(&mut tokio::io::BufReader::new(Cursor::new(mat_text))).await
    };

    let (obj_models, obj_materials) = tobj::tokio::load_obj_buf(
        &mut obj_reader,
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
        material_loader,
    )
    .await?;

    let mut materials = Vec::new();
    for mtl in obj_materials? {
        if let Some(texture_file) = mtl.diffuse_texture {
            let diffuse_texture = load_texture(device, queue, &texture_file).await?;
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("[{file_name}] {texture_file}")),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    },
                ],
            });

            materials.push(model::Material {
                name: mtl.name,
                diffuse_texture,
                bind_group,
            });
        }
    }

    let mut meshes = Vec::new();
    for model in obj_models {
        let vertices = (0..model.mesh.positions.len() / 3)
            .map(|i| model::ModelVertex {
                position: [
                    model.mesh.positions[i * 3 + 0],
                    model.mesh.positions[i * 3 + 1],
                    model.mesh.positions[i * 3 + 2],
                ],
                tex_coord: [
                    model.mesh.texcoords[i * 2 + 0],
                    model.mesh.texcoords[i * 2 + 1],
                ],
                normal: [
                    model.mesh.normals[i * 3 + 0],
                    model.mesh.normals[i * 3 + 1],
                    model.mesh.normals[i * 3 + 2],
                ],
            })
            .collect::<Vec<model::ModelVertex>>();

        let vertices_data = unsafe { vertices.align_to::<u8>().1 };

        let indices_data = unsafe { model.mesh.indices.align_to::<u8>().1 };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{file_name} Vertex buffer")),
            contents: vertices_data,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{file_name} Index buffer")),
            contents: indices_data,
            usage: wgpu::BufferUsages::INDEX,
        });

        let mesh = model::Mesh {
            name: file_name.to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: model.mesh.indices.len() as u32,
            material: model.mesh.material_id.unwrap_or(0),
        };

        meshes.push(mesh);
    }

    Ok(model::Model { meshes, materials })
}
