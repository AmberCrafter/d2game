#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufReader, Cursor};

use std::{collections::HashMap, path::Path, sync::Arc};

use crate::engine::{
    BoxResult,
    buffer::setup_uniform,
    model::{self, Animation, ModelVertex},
    texture,
};
use anyhow::anyhow;
use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

// TODO: fix url base on live server and normal web server
#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&format!(
        "{}/{}/",
        "http://127.0.0.1:5500",
        "day23_xr_wgpu/xr_wgpu" // location.origin().unwrap(),
                                // option_env!("OUT_DIR").unwrap_or("res")
    ))
    .unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> BoxResult<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format_url(file_name);
        let txt = reqwest::get(url).await?.text().await?;
        Ok(txt)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(file_name);
        let txt = std::fs::read_to_string(path)?;
        Ok(txt)
    }
}

pub async fn load_binary(file_name: &str) -> BoxResult<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format_url(file_name);
        let data = reqwest::get(url).await?.bytes().await?.to_vec();
        Ok(data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = std::path::Path::new(env!("OUT_DIR"))
            .join("res")
            .join(file_name);
        let data = std::fs::read(path)?;
        Ok(data)
    }
}

pub async fn load_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    file_name: &str,
) -> BoxResult<texture::Texture> {
    let data = load_binary(file_name).await?;
    Ok(texture::Texture::load_texture_from_bytes(
        device,
        queue,
        Some(file_name),
        &data,
    )?)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_obj_model(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    file_name: &str,
) -> BoxResult<model::Model> {
    use std::path::PathBuf;

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

        materials.push(Arc::new(
            (model::ObjMaterial {
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
            }),
        ) as Arc<_>);
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
            vertex: vertices,
            vertex_buffer,
            index_buffer,
            num_elements: model.mesh.indices.len() as u32,
            material: model.mesh.material_id.unwrap_or(0),
            uniform_transform: None,
        };

        meshes.push(mesh);
    }

    Ok(model::Model {
        name: "obj".to_string(),
        meshes,
        materials,
        animations: HashMap::new(),
    })
}

pub fn load_gltf_model(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    filename: &str,
) -> BoxResult<model::Model> {
    let path = Path::new(filename);
    let (document, buffers, images) = gltf::import(path)?;

    let mut meshes = Vec::new();
    let mut materials = Vec::new();
    let mut animations = HashMap::new();

    {
        // meshes
        for node in document.nodes() {
            let transform = cgmath::Matrix4::from(node.transform().matrix());
            if let Some(mesh) = node.mesh() {
                let name = format!(
                    "{}",
                    mesh.name()
                        .unwrap_or(&format!("{}/mesh_{}", filename, mesh.index()))
                );

                let mut vertices = Vec::new();
                let mut indices = Vec::new();
                let mut material_idx = 0;

                // FIXME: material for each primitive
                for prim in mesh.primitives() {
                    let r = prim.reader(|buffer| Some(&buffers[buffer.index()]));

                    let p_indices = r.read_indices().unwrap().into_u32();
                    let positions = r.read_positions().unwrap();
                    let texcoords = r.read_tex_coords(0).unwrap().into_f32();
                    let normals = r.read_normals().unwrap();

                    for ((position, tex_coord), normal) in positions.zip(texcoords).zip(normals) {
                        let position =
                            cgmath::Vector4::new(position[0], position[1], position[2], 1.0);
                        let position = transform * position;
                        let position = position.truncate().into();

                        vertices.push(ModelVertex {
                            position,
                            tex_coord,
                            normal,
                        });
                    }

                    indices.extend(p_indices);
                    material_idx = prim.material().index().unwrap_or(0);
                }
                let vertices_data = unsafe { vertices.align_to::<u8>().1 };
                let indices_data = unsafe { indices.align_to::<u8>().1 };

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{filename} Vertex buffer")),
                    contents: vertices_data,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{filename} Index buffer")),
                    contents: indices_data,
                    usage: wgpu::BufferUsages::INDEX,
                });

                const TYPESIZE: usize = core::mem::size_of::<cgmath::Matrix4<f32>>();
                let uniform_transform = setup_uniform::<cgmath::Matrix4<f32>, TYPESIZE>(
                    device,
                    &name,
                    cgmath::Matrix4::identity(),
                    0,
                    wgpu::ShaderStages::VERTEX,
                );

                let mesh = model::Mesh {
                    name: name,
                    vertex: vertices,
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    material: material_idx,
                    uniform_transform: Some(uniform_transform),
                };

                meshes.push(mesh);
            }
        }
    }

    {
        // materials
        for mtl in document.materials() {
            let pbr = mtl.pbr_metallic_roughness();
            let mut entries = Vec::new();

            /*
               unimplement list
               - ior
            */

            // 0
            let base_color = {
                let base_color = pbr.base_color_factor();
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("[{filename}] base_color")),
                    contents: unsafe { base_color.align_to::<u8>().1 },
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
                Some(buffer)
            };
            if let Some(buffer) = base_color.as_ref() {
                entries.push(wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                });
            };

            // 1
            let metallic = {
                let metallic = pbr.metallic_factor();
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("[{filename}] metallic")),
                    contents: &metallic.to_ne_bytes(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
                Some(buffer)
            };
            if let Some(buffer) = metallic.as_ref() {
                entries.push(wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding(),
                });
            };

            // 2
            let roughness = {
                let roughness = pbr.roughness_factor();
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("[{filename}] roughness")),
                    contents: &roughness.to_ne_bytes(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
                Some(buffer)
            };
            if let Some(buffer) = roughness.as_ref() {
                entries.push(wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffer.as_entire_binding(),
                });
            };

            // 3, 4
            let base_color_texture = if let Some(info) = pbr.base_color_texture() {
                let base_color_texture = texture::Texture::load_texture_from_gltf(
                    device,
                    queue,
                    info.texture().name(),
                    &info.texture(),
                    &images,
                )?;
                Some(base_color_texture)
            } else {
                None
            };
            if let Some(texture) = base_color_texture.as_ref() {
                entries.push(wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                });
            }

            // 5, 6
            let metallic_roughness_texture = if let Some(info) = pbr.metallic_roughness_texture() {
                let metallic_roughness_texture = texture::Texture::load_texture_from_gltf(
                    device,
                    queue,
                    info.texture().name(),
                    &info.texture(),
                    &images,
                )?;
                Some(metallic_roughness_texture)
            } else {
                None
            };
            if let Some(texture) = metallic_roughness_texture.as_ref() {
                entries.push(wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                });
            }

            // 7, 8
            let normal_texture = if let Some(info) = mtl.normal_texture() {
                let normal_texture = texture::Texture::load_texture_from_gltf(
                    device,
                    queue,
                    info.texture().name(),
                    &info.texture(),
                    &images,
                )?;
                Some(normal_texture)
            } else {
                None
            };
            if let Some(texture) = normal_texture.as_ref() {
                entries.push(wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                });
            }

            // 9, 10
            let occlusion_texture = if let Some(info) = mtl.occlusion_texture() {
                let occlusion_texture = texture::Texture::load_texture_from_gltf(
                    device,
                    queue,
                    info.texture().name(),
                    &info.texture(),
                    &images,
                )?;
                Some(occlusion_texture)
            } else {
                None
            };
            if let Some(texture) = occlusion_texture.as_ref() {
                entries.push(wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                });
            }

            // 11
            let emissive_factor = {
                let emissive_factor = mtl.emissive_factor();
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("[{filename}] emissive_factor")),
                    contents: unsafe { emissive_factor.align_to::<u8>().1 },
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
                Some(buffer)
            };
            if let Some(buffer) = emissive_factor.as_ref() {
                entries.push(wgpu::BindGroupEntry {
                    binding: 11,
                    resource: buffer.as_entire_binding(),
                });
            };

            // 12, 13
            let emissive_texture = if let Some(info) = mtl.emissive_texture() {
                let emissive_texture = texture::Texture::load_texture_from_gltf(
                    device,
                    queue,
                    info.texture().name(),
                    &info.texture(),
                    &images,
                )?;
                Some(emissive_texture)
            } else {
                None
            };
            if let Some(texture) = emissive_texture.as_ref() {
                entries.push(wgpu::BindGroupEntry {
                    binding: 12,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                });
                entries.push(wgpu::BindGroupEntry {
                    binding: 13,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                });
            }

            // println!("[Debug] {:?}({:?}) {:#?}", file!(), line!(), entries);
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("{filename} bind_group")),
                layout,
                entries: &entries,
            });

            materials.push(Arc::new(model::PBRMaterial {
                name: mtl
                    .name()
                    .unwrap_or(&format!("{}/{}", filename, "material"))
                    .to_string(),
                base_color,
                metallic,
                roughness,
                base_color_texture,
                metallic_roughness_texture,
                normal_texture,
                occlusion_texture,
                emissive_factor,
                emissive_texture,

                bind_group: Some(bind_group),
            }) as Arc<_>);
        }
    }

    {
        // animations
        for doc_animation in document.animations() {
            let name = doc_animation
                .name()
                .unwrap_or(&format!("{}_animation", filename))
                .to_string();

            let mut map: HashMap<usize, Animation> = HashMap::new();

            for channel in doc_animation.channels() {
                let mesh_id = channel.target().node().index();
                let entry = map.entry(mesh_id).or_insert(Animation {
                    name: name.clone(),
                    mesh_id,
                    translation: None,
                    rotation: None,
                    scale: None,
                    period: 0.0_f32,
                    do_loop: true,
                });

                let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
                let mut times = Vec::new();
                for input in reader.read_inputs().unwrap() {
                    times.push(input);
                }

                entry.mesh_id = mesh_id;
                entry.period = entry.period.max(*times.last().unwrap());

                if let Some(output) = reader.read_outputs() {
                    match output {
                        gltf::animation::util::ReadOutputs::MorphTargetWeights(_val) => {
                            unimplemented!()
                        }
                        gltf::animation::util::ReadOutputs::Translations(val) => {
                            let buf = times
                                .iter()
                                .zip(val)
                                .map(|(&t, v)| (t, v))
                                .collect::<Vec<_>>();
                            let res = entry.translation.replace(buf);
                            if let Some(res) = res {
                                return Err(anyhow!("Duplicate translation on {:}", mesh_id).into());
                            }
                        }
                        gltf::animation::util::ReadOutputs::Rotations(val) => {
                            let buf = times
                                .iter()
                                .zip(val.into_f32())
                                .map(|(&t, v)| (t, v))
                                .collect::<Vec<_>>();
                            let res = entry.rotation.replace(buf);
                            if let Some(res) = res {
                                return Err(anyhow!("Duplicate rotation on {:}", mesh_id).into());
                            }
                        }
                        gltf::animation::util::ReadOutputs::Scales(val) => {
                            let buf = times
                                .iter()
                                .zip(val)
                                .map(|(&t, v)| (t, v))
                                .collect::<Vec<_>>();
                            let res = entry.scale.replace(buf);
                            if let Some(res) = res {
                                return Err(anyhow!("Duplicate scale on {:}", mesh_id).into());
                            }
                        }
                    }
                }
            }

            animations.insert(
                name.to_string(),
                map.drain().into_iter().map(|(_k, v)| v).collect::<Vec<_>>(),
            );
        }
    }

    let model = model::Model {
        name: path.file_stem().unwrap().to_str().unwrap().to_string(),
        meshes,
        materials,
        animations,
    };
    Ok(model)
}
