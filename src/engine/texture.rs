use anyhow::anyhow;

pub struct TextureInfo {
    pub depth_texture: Option<Texture>,
}

impl TextureInfo {
    pub fn new() -> Self {
        Self {
            depth_texture: None,
        }
    }

    pub fn setup(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        let depth_texture = Texture::create_depth_texture(device, config);
        self.depth_texture.replace(depth_texture);
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Texture {
    pub name: Option<String>,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

#[allow(unused)]
impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    fn single_pixel_bytes(format: wgpu::TextureFormat) -> u32 {
        use wgpu::TextureFormat;
        match format {
            TextureFormat::R8Sint
            | TextureFormat::R8Snorm
            | TextureFormat::R8Uint
            | TextureFormat::R8Unorm => 1,
            TextureFormat::R16Float
            | TextureFormat::R16Sint
            | TextureFormat::R16Snorm
            | TextureFormat::R16Uint
            | TextureFormat::R16Unorm
            | TextureFormat::Rg8Sint
            | TextureFormat::Rg8Snorm
            | TextureFormat::Rg8Uint
            | TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rgba8Sint
            | TextureFormat::Rgba8Uint
            | TextureFormat::Bgra8Unorm
            | TextureFormat::Bgra8UnormSrgb
            | TextureFormat::Rgba8Snorm
            | TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8UnormSrgb => 4,
            TextureFormat::Rgba16Float
            | TextureFormat::Rgba16Sint
            | TextureFormat::Rgba16Snorm
            | TextureFormat::Rgba16Uint
            | TextureFormat::Rgba16Unorm => 8,
            TextureFormat::Rgba32Float | TextureFormat::Rgba32Sint | TextureFormat::Rgba32Uint => {
                16
            }
            _ => 0,
        }
    }

    fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        img: &image::DynamicImage,
    ) -> anyhow::Result<Self> {
        let img = img.to_rgba8();
        let dimension = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimension.0,
            height: dimension.1,
            depth_or_array_layers: 1,
        };

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texel = img.into_raw();
        // let texel = img.as_bytes();

        let texture = device.create_texture(&wgpu::wgt::TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        println!("{label:?}: format: {:?}", texture.format());
        let pixel_byte = Self::single_pixel_bytes(texture.format());

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texel,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pixel_byte * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );

        println!("[Debug] {:?}({:?})", file!(), line!());
        let texture_view = texture.create_view(&wgpu::wgt::TextureViewDescriptor {
            // format: Some(format.remove_srgb_suffix()),
            ..Default::default()
        });
        let texture_sampler = device.create_sampler(&wgpu::wgt::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            name: label.map(|val| val.to_string()),
            texture,
            view: texture_view,
            sampler: texture_sampler,
        })
    }

    pub fn load_texture_from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        bytes: &[u8],
    ) -> anyhow::Result<Self> {
        let img = image::load_from_memory(bytes)?;
        Texture::from_image(device, queue, label, &img)
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let texture_size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::wgt::TextureDescriptor {
            label: Some("depth texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        println!("[Debug] {:?}({:?})", file!(), line!());
        let view = texture.create_view(&wgpu::wgt::TextureViewDescriptor {
            // format: Some(format.remove_srgb_suffix()),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::wgt::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 200.0,
            ..Default::default()
        });

        Self {
            name: Some("depth texture".to_string()),
            texture,
            view,
            sampler,
        }
    }

    pub fn load_texture_from_gltf(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        texture: &gltf::Texture,
        images: &Vec<gltf::image::Data>
    ) -> anyhow::Result<Self> {
        let img_idx = texture.source().index();
        let sampler = texture.sampler();
        let img_data = &images[img_idx];

        if img_data.format != gltf::image::Format::R8G8B8A8 {
            return Err(anyhow!("Not support image format!"));
        }

        let texture_size = wgpu::Extent3d {
            width: img_data.width,
            height: img_data.height,
            depth_or_array_layers: 1,
        };

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texel = &img_data.pixels;

        let texture = device.create_texture(&wgpu::wgt::TextureDescriptor {
            label,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let pixel_byte = Self::single_pixel_bytes(texture.format());

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texel,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pixel_byte * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
            texture_size,
        );

        println!("[Debug] {:?}({:?})", file!(), line!());
        let texture_view = texture.create_view(&wgpu::wgt::TextureViewDescriptor {
            // format: Some(format.remove_srgb_suffix()),
            ..Default::default()
        });

        let get_wrapping_mode = |gltf_mode: gltf::texture::WrappingMode| {
            match gltf_mode {
                gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::MirrorRepeat,
                gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
                gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            }
        };

        let get_magfilter_mode = |gltf_mode: gltf::texture::MagFilter | {
            match gltf_mode {
                gltf::texture::MagFilter::Linear => wgpu::FilterMode::Linear,
                gltf::texture::MagFilter::Nearest => wgpu::FilterMode::Nearest
            }
        };

        let get_minfilter_mode = |gltf_mode: gltf::texture::MinFilter | {
            match gltf_mode {
                gltf::texture::MinFilter::Linear => (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear),
                gltf::texture::MinFilter::Nearest => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
                gltf::texture::MinFilter::LinearMipmapLinear => (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear),
                gltf::texture::MinFilter::LinearMipmapNearest => (wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest),
                gltf::texture::MinFilter::NearestMipmapLinear => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Linear),
                gltf::texture::MinFilter::NearestMipmapNearest => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
            }
        };

        let texture_sampler = device.create_sampler(&wgpu::wgt::SamplerDescriptor {
            address_mode_u: get_wrapping_mode(sampler.wrap_s()),
            address_mode_v: get_wrapping_mode(sampler.wrap_t()),
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: get_magfilter_mode(sampler.mag_filter().unwrap_or(gltf::texture::MagFilter::Linear)),
            min_filter: get_minfilter_mode(sampler.min_filter().unwrap_or(gltf::texture::MinFilter::Nearest)).0,
            mipmap_filter: get_minfilter_mode(sampler.min_filter().unwrap_or(gltf::texture::MinFilter::Nearest)).1,
            ..Default::default()
        });

        Ok(Self {
            name: label.map(|val| val.to_string()),
            texture,
            view: texture_view,
            sampler: texture_sampler,
        })
    }
}
