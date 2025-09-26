use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

const WINDOW_HEIGHT: usize = 800;

#[allow(unused)]
pub struct AppSurface {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,    // surface should exist whole time.
    pub device: wgpu::Device,
    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

#[allow(unused)]
impl AppSurface {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let height = (WINDOW_HEIGHT as f64 * window.scale_factor()) as u32;
        let width = (height as f64 * 1.6) as u32;
        let phy_size = PhysicalSize::new(width, height);
        let _ = window.request_inner_size(phy_size);

        // GPU instance
        let gpu = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = gpu.create_surface(window.clone())?;
        let adapter = gpu
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::wgt::DeviceDescriptor {
                label: Some("App Gpu device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;

        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();

        surface.configure(&device, &config);

        Ok(Self {
            window,
            surface,
            device,
            adapter,
            queue,
            config
        })
    }
}
