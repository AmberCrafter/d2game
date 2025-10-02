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
        let mut entries = Vec::new();

        // 0
        let ambient = if let Some(ambient) = mtl.ambient {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("[{file_name}] ambient")),
                contents: unsafe { ambient.align_to::<u8>().1 },
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            Some(buffer)
        } else {
            None
        };
        if let Some(buffer) = ambient.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            });
        };

        // 1
        let diffuse = if let Some(diffuse) = mtl.diffuse {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("[{file_name}] diffuse")),
                contents: unsafe { diffuse.align_to::<u8>().1 },
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            Some(buffer)
        } else {
            None
        };
        if let Some(buffer) = diffuse.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: buffer.as_entire_binding(),
            });
        };

        // 2
        let specular = if let Some(specular) = mtl.specular {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("[{file_name}] specular")),
                contents: unsafe { specular.align_to::<u8>().1 },
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            Some(buffer)
        } else {
            None
        };
        if let Some(buffer) = specular.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: buffer.as_entire_binding(),
            });
        };

        // 3
        let shininess = if let Some(shininess) = mtl.shininess {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("[{file_name}] shininess")),
                contents: &shininess.to_ne_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            Some(buffer)
        } else {
            None
        };
        if let Some(buffer) = shininess.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: buffer.as_entire_binding(),
            });
        };

        // 4
        let dissolve = if let Some(dissolve) = mtl.dissolve {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("[{file_name}] dissolve")),
                contents: &dissolve.to_ne_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            Some(buffer)
        } else {
            None
        };
        if let Some(buffer) = dissolve.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 4,
                resource: buffer.as_entire_binding(),
            });
        };

        // 5
        let optical_density = if let Some(optical_density) = mtl.optical_density {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("[{file_name}] optical_density")),
                contents: &optical_density.to_ne_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            Some(buffer)
        } else {
            None
        };
        if let Some(buffer) = optical_density.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 5,
                resource: buffer.as_entire_binding(),
            });
        };

        // 6, 7
        let ambient_texture = if let Some(texture_file) = mtl.ambient_texture {
            let ambient_texture = load_texture(device, queue, &texture_file).await?;
            Some(ambient_texture)
        } else {
            None
        };
        if let Some(texture) = ambient_texture.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
        }

        // 8, 9
        let diffuse_texture = if let Some(texture_file) = mtl.diffuse_texture {
            let diffuse_texture = load_texture(device, queue, &texture_file).await?;
            Some(diffuse_texture)
        } else {
            None
        };
        if let Some(texture) = diffuse_texture.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 8,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 9,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
        }

        // 10, 11
        let specular_texture = if let Some(texture_file) = mtl.specular_texture {
            let specular_texture = load_texture(device, queue, &texture_file).await?;
            Some(specular_texture)
        } else {
            None
        };
        if let Some(texture) = specular_texture.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 10,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 11,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
        }

        // 12, 13
        let normal_texture = if let Some(texture_file) = mtl.normal_texture {
            let normal_texture = load_texture(device, queue, &texture_file).await?;
            Some(normal_texture)
        } else {
            None
        };
        if let Some(texture) = normal_texture.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 12,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 13,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
        }

        // 14, 15
        let shininess_texture = if let Some(texture_file) = mtl.shininess_texture {
            let shininess_texture = load_texture(device, queue, &texture_file).await?;
            Some(shininess_texture)
        } else {
            None
        };
        if let Some(texture) = shininess_texture.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 14,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 15,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
        }

        // 16, 17
        let dissolve_texture = if let Some(texture_file) = mtl.dissolve_texture {
            let dissolve_texture = load_texture(device, queue, &texture_file).await?;
            Some(dissolve_texture)
        } else {
            None
        };
        if let Some(texture) = dissolve_texture.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 16,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 17,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
        }

        // 18
        let illumination_model = if let Some(illumination_model) = mtl.illumination_model {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("[{file_name}] optical_density")),
                contents: &illumination_model.to_ne_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            Some(buffer)
        } else {
            None
        };
        if let Some(buffer) = illumination_model.as_ref() {
            entries.push(wgpu::BindGroupEntry {
                binding: 18,
                resource: buffer.as_entire_binding(),
            });
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{file_name} bind_group")),
            layout,
            entries: &entries,
        });

        materials.push(model::Material {
            name: mtl.name,
            ambient,
            diffuse,
            specular,
            shininess,
            optical_density,
            ambient_texture,
            diffuse_texture,
            specular_texture,
            normal_texture,
            shininess_texture,
            dissolve_texture,
            illumination_model,
            bind_group: Some(bind_group),
        });
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
