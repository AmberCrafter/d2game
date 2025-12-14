mod logger;

use std::{sync::Arc, time::Duration};

use log::{debug, error, info};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    event_loop::EventLoop,
    window::Window,
};

#[allow(unused)]
pub trait WgpuAppAction {
    fn new(window: Arc<Window>) -> impl core::future::Future<Output = Arc<std::sync::Mutex<Self>>>;
    // fn new(window: Arc<Window>) -> Self;
    fn set_window_resized(&mut self, size: PhysicalSize<u32>);
    fn get_size(&self) -> PhysicalSize<u32>;

    // input operation
    fn keyboard_input(&mut self, event: &KeyEvent, is_synthetic: bool) -> bool {
        false
    }
    fn mouse_click(&mut self, state: ElementState, button: MouseButton) -> bool {
        false
    }
    fn mouse_wheel(&mut self, delta: MouseScrollDelta, phase: TouchPhase) -> bool {
        false
    }
    fn cursor_move(&mut self, position: PhysicalPosition<f64>) -> bool {
        false
    }
    fn device_input(&mut self, event: DeviceEvent) -> bool {
        false
    }

    // state operation
    fn update(&mut self, dt: Duration) {}
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}

#[allow(unused)]
pub struct WgpuAppHandler<A: WgpuAppAction> {
    title: String,
    window: Option<Arc<Window>>,
    app: Option<Arc<std::sync::Mutex<A>>>,
    preload_resources: Vec<Box<dyn Fn(Arc<std::sync::Mutex<A>>) -> anyhow::Result<()>>>,
    last_render_time: std::time::Instant,
}

#[allow(unused)]
impl<A: WgpuAppAction> WgpuAppHandler<A> {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            window: None,
            app: None,
            preload_resources: Vec::new(),
            last_render_time: std::time::Instant::now(),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(self);
        Ok(())
    }

    pub fn add_resource_loader(
        &mut self,
        cb: Box<dyn Fn(Arc<std::sync::Mutex<A>>) -> anyhow::Result<()>>,
    ) -> anyhow::Result<()> {
        self.preload_resources.push(cb);
        Ok(())
    }

    fn config_window(&mut self, window: Arc<Window>) {
        self.window.replace(window);
    }

    fn pre_present_notify(&self) {
        if let Some(window) = self.window.as_ref() {
            window.pre_present_notify();
        }
    }

    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

#[allow(unused)]
// Implement winit eventloop trait
impl<A: WgpuAppAction> ApplicationHandler for WgpuAppHandler<A> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.last_render_time = std::time::Instant::now();
        let window_attrs = Window::default_attributes();
        let Ok(window) = event_loop.create_window(window_attrs) else {
            error!("System Error");
            return;
        };
        let window = Arc::new(window);

        // setup async runtime
        let Ok(rt) = tokio::runtime::Runtime::new() else {
            error!("System Error");
            return;
        };

        let wgpu_app = rt.block_on(A::new(window.clone()));

        for res in self.preload_resources.drain(..) {
            res(wgpu_app.clone());
        }

        self.app.replace(wgpu_app);

        self.config_window(window.clone());
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.app.take();
        self.window.take();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let app_copy = self.app.clone().unwrap();
        let mut app = app_copy.lock().unwrap();

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {
                    info!("Window minimized!");
                } else {
                    info!("Window resize: {:?}", physical_size);
                    app.set_window_resized(physical_size);
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                app.mouse_click(state, button);
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                app.mouse_wheel(delta, phase);
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                app.cursor_move(position);
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                app.keyboard_input(&event, is_synthetic);
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = now - self.last_render_time;
                self.last_render_time = now;

                app.update(dt);
                self.pre_present_notify();

                match app.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        error!("System error: Surface is lost.");
                        eprintln!("System error: Surface is lost.");
                    }
                    Err(e) => {
                        error!("System error: {:?}.", e);
                        eprintln!("System error: {:?}.", e);
                    }
                }

                self.request_redraw();
            }
            e => {
                debug!("Todo: {e:?}!")
            }
        }
    }
}

#[allow(unused)]
pub fn init<A: WgpuAppAction>(title: &str) -> anyhow::Result<WgpuAppHandler<A>> {
    logger::init_logger();
    let mut app = WgpuAppHandler::<A>::new(title);
    Ok(app)
}
