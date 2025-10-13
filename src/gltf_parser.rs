#[cfg(test)]
mod test {
    use gltf::Gltf;

    use super::*;
    #[test]
    fn case1() -> anyhow::Result<()> {
        // let gltf = Gltf::open("res/player_walk.gltf")?;

        let (document, buffers, images) = gltf::import("res/player_walk.gltf")?;
        // println!("{:?}", document);

        for animation in document.animations() {
            if animation.name() != Some("Sphere.002Action") {
                continue;
            }
            for channel in animation.channels() {
                let sampler = channel.sampler();
                let target_node = channel.target().node();
                let inter = sampler.interpolation();
                println!("inter: {:?}", inter);

                let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

                let ts = if let Some(input) = reader.read_inputs()  {
                    match input {
                        gltf::accessor::Iter::Sparse(val) => {
                            println!("Debug {:?}", val);
                            let times = Vec::new();
                            times
                        }
                        gltf::accessor::Iter::Standard(times) => {
                            let times = times.collect::<Vec<_>>();
                            times
                        }
                    }
                } else {
                    vec![]
                };
                println!("{:?}", ts);


                let kf = if let Some(output) = reader.read_outputs() {
                    match output {
                        gltf::animation::util::ReadOutputs::Translations(val) => {
                            let res = val.collect::<Vec<_>>();
                            println!("trans: {:?}", res);
                        }
                        gltf::animation::util::ReadOutputs::Rotations(val) => {
                            let res = val.into_f32().collect::<Vec<_>>();
                            println!("rota: {:?}", res);
                        }
                        gltf::animation::util::ReadOutputs::Scales(val) => {
                            let res = val.collect::<Vec<_>>();
                            println!("scale: {:?}", res);
                        }
                        gltf::animation::util::ReadOutputs::MorphTargetWeights(val) => {
                            let res = val.into_f32().collect::<Vec<_>>();
                            println!("morph: {:?}", res);
                        }
                    }
                };
            }

        }

        Ok(())
    }
}
