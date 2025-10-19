#[allow(unused)]
#[derive(Debug, Default)]
pub struct Animation {
    pub name: String,
    
}

#[cfg(test)]
mod test {
    use gltf::Gltf;

    use super::*;
    #[test]
    fn case1() -> anyhow::Result<()> {
        // let gltf = Gltf::open("res/player_walk.gltf")?;

        let (document, buffers, images) = gltf::import("res/player_walk.gltf")?;
        // println!("{:?}", document);

        
        for node in document.nodes() {
            if node.name() != Some("Sphere.002") {continue;}
            println!("mesh: {:?}", node.mesh().unwrap().index());

            let transform = node.transform().matrix();
            let transform = cgmath::Matrix4::from(transform);

            let mut vertices = Vec::new();
            for prim in node.mesh().unwrap().primitives() {
                let mut reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));
                let vertex = reader.read_positions().unwrap();
                for v in vertex {
                    let tmp = cgmath::Vector4::from([v[0], v[1], v[2], 1.0]);
                    let val = transform * tmp;
                    // println!("{:?} -> {:?}", v, val);

                    vertices.push(val);
                }
            }

            for animation in document.animations() {
                for channel in animation.channels() {
                    if channel.target().node().index() != node.index() {continue;}
                    
                    let mut reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
                    let mut times = Vec::new();

                    for input in reader.read_inputs().unwrap() {
                        // println!("input: {:?}", input);
                        times.push(input);
                    }

                    let mut ops = Vec::new();
                    if let Some(output) = reader.read_outputs() {
                        match output {
                            gltf::animation::util::ReadOutputs::MorphTargetWeights(val) => {}
                            gltf::animation::util::ReadOutputs::Translations(val) => {
                                val.for_each(|v| ops.push(v));
                            }
                            gltf::animation::util::ReadOutputs::Rotations(val) => {}
                            gltf::animation::util::ReadOutputs::Scales(val) => {}
                        }
                    }

                    if (times.len() != ops.len()) {panic!("invalid");}
                    for (t,v) in times.iter().zip(ops.iter()) {
                        println!("{t:?} {v:?}");
                        let transform = cgmath::Matrix4::from_translation(cgmath::Vector3::from(*v));

                        for vertex in &vertices[..4] {
                            println!("{:?} -> {:?}", vertex, transform * vertex);
                        }


                    }
                }
            }
        }




        // for animation in document.animations() {
        //     if animation.name() != Some("Sphere.002Action") {
        //         continue;
        //     }
        //     for channel in animation.channels() {
        //         let sampler = channel.sampler();
        //         let target_node = channel.target().node();
        //         let inter = sampler.interpolation();
        //         println!("inter: {:?}", inter);

        //         let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

        //         let ts = if let Some(input) = reader.read_inputs()  {
        //             match input {
        //                 gltf::accessor::Iter::Sparse(val) => {
        //                     println!("Debug {:?}", val);
        //                     let times = Vec::new();
        //                     times
        //                 }
        //                 gltf::accessor::Iter::Standard(times) => {
        //                     let times = times.collect::<Vec<_>>();
        //                     times
        //                 }
        //             }
        //         } else {
        //             vec![]
        //         };
        //         println!("{:?}", ts);


        //         let kf = if let Some(output) = reader.read_outputs() {
        //             match output {
        //                 gltf::animation::util::ReadOutputs::Translations(val) => {
        //                     let res = val.collect::<Vec<_>>();
        //                     println!("trans: {:?}", res);
        //                 }
        //                 gltf::animation::util::ReadOutputs::Rotations(val) => {
        //                     let res = val.into_f32().collect::<Vec<_>>();
        //                     println!("rota: {:?}", res);
        //                 }
        //                 gltf::animation::util::ReadOutputs::Scales(val) => {
        //                     let res = val.collect::<Vec<_>>();
        //                     println!("scale: {:?}", res);
        //                 }
        //                 gltf::animation::util::ReadOutputs::MorphTargetWeights(val) => {
        //                     let res = val.into_f32().collect::<Vec<_>>();
        //                     println!("morph: {:?}", res);
        //                 }
        //             }
        //         };
        //     }

        // }

        Ok(())
    }
}
