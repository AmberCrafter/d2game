use crate::engine::{
    entity::Entity,
    material::Material,
    mesh::{Mesh, Primitive},
};

pub struct Model {
    pub root_entity: Entity,
    pub entities: Vec<Entity>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn new(
        meshes: Vec<Mesh>,
        materials: Vec<Material>,
        entities: Vec<Entity>,
        root_entity: Entity,
    ) -> Self {
        Self {
            root_entity,
            entities,
            meshes,
            materials,
        }
    }

    pub fn render<'a>(&self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.render_impl(&self.root_entity, render_pass);
    }

    pub fn render_impl<'a>(&self, entity: &Entity, render_pass: &mut wgpu::RenderPass<'a>) {
        // Scene -> Model -> entity -> mesh -> primative -> [material, vertex]
        for &eidx in entity.children.iter() {
            let entity = &self.entities[eidx];

            if let Some(mesh_idx) = entity.mesh_index {
                let mesh = &self.meshes[mesh_idx];
                for primative in mesh.primitives.iter() {
                    // material
                    let material = &self.materials[primative.material_index];
                    render_pass.set_bind_group(
                        Material::BindGroup_Index,
                        &material.bind_group,
                        &[],
                    );

                    // Vertex
                    render_pass.set_vertex_buffer(
                        Primitive::Position_Location,
                        primative.positions.slice(..),
                    );
                    render_pass.set_vertex_buffer(
                        Primitive::TexCoords_Location,
                        primative.tex_coords.slice(..),
                    );
                    render_pass
                        .set_vertex_buffer(Primitive::Normal_Location, primative.normals.slice(..));

                    // Indices
                    render_pass
                        .set_index_buffer(primative.indices.slice(..), wgpu::IndexFormat::Uint32);

                    render_pass.draw_indexed(0..primative.indices_num, 0, 0..1);
                }
            }

            self.render_impl(entity, render_pass);
        }
    }
}
