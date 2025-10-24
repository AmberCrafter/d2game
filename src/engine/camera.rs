use cgmath::{Deg, InnerSpace, SquareMatrix};
use wgpu::{Device, util::DeviceExt};
use winit::event::KeyEvent;

type Pos3 = cgmath::Point3<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Vec4 = cgmath::Vector4<f32>;
type Mat4 = cgmath::Matrix4<f32>;

pub struct CameraConfig {
    pub eye: Pos3,
    pub target: Pos3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CameraUniform {
    view_pos: Vec4,
    view_proj: Mat4,
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_pos: Vec4::new(0.0, 0.0, 0.0, 0.0),
            view_proj: Mat4::identity(),
        }
    }

    fn update_view_pos(&mut self, val: Vec4) {
        self.view_pos = val;
    }

    fn update_view_proj(&mut self, val: Mat4) {
        self.view_proj = val;
    }

    pub fn as_bytes(&self) -> [u8; 80] {
        // unsafe { &(*(self.view_proj.as_ptr() as *const [u8; core::mem::size_of::<Mat4>()])) }
        unsafe {
            core::mem::transmute::<CameraUniform, [u8; 80]>(self.clone())
        }
    }
}

#[allow(unused)]
pub struct CameraController {
    speed: f32,
    is_up_press: bool,
    is_down_press: bool,
    is_forwward_press: bool,
    is_backward_press: bool,
    is_left_press: bool,
    is_right_press: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_press: false,
            is_down_press: false,
            is_forwward_press: false,
            is_backward_press: false,
            is_left_press: false,
            is_right_press: false,
        }
    }

    pub fn process_event(&mut self, event: &KeyEvent) -> bool {
        match event {
            KeyEvent {
                state,
                physical_key,
                logical_key,
                ..
            } => match (logical_key, physical_key) {
                (winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space), _) => {
                    self.is_up_press = state.is_pressed();
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Space)) => {
                    self.is_up_press = state.is_pressed();
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ShiftLeft)) => {
                    self.is_down_press = state.is_pressed();
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyW)) => {
                    self.is_forwward_press = state.is_pressed();
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyS)) => {
                    self.is_backward_press = state.is_pressed();
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA)) => {
                    self.is_left_press = state.is_pressed();
                    true
                }
                (_, winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyD)) => {
                    self.is_right_press = state.is_pressed();
                    true
                }
                _ => false,
            },
        }
    }

    fn update_camera_config(&mut self, config: &mut CameraConfig) {
        let forward = config.target - config.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.is_forwward_press && forward_mag > self.speed {
            config.eye += forward_norm * self.speed * 0.5;
        }

        if self.is_backward_press {
            config.eye -= forward_norm * self.speed * 0.3;
        }

        let right = forward_norm.cross(config.up);

        // let forward = config.target - config.eye;
        // let forward_mag = forward.magnitude();

        if self.is_left_press {
            // config.eye = config.target - (forward - right * self.speed).normalize() * forward_mag;
            config.eye = config.eye - right * self.speed;
            config.target -= right * self.speed;
        }

        if self.is_right_press {
            // config.eye = config.target - (forward + right * self.speed).normalize() * forward_mag;
            config.eye = config.eye + right * self.speed;
            config.target += right * self.speed;
        }
    }
}

pub struct CameraInfo {
    config: CameraConfig,
    pub uniform: CameraUniform,
    pub buffer: Option<wgpu::Buffer>,
    // pub bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub bind_group: Option<wgpu::BindGroup>,

    pub controller: CameraController,
}

impl CameraInfo {
    pub fn new(config: CameraConfig) -> Self {
        let uniform = CameraUniform::new();
        let controller = CameraController::new(0.1);

        Self {
            config,
            uniform,
            buffer: None,
            // bind_group_layout: None,
            bind_group: None,
            controller,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        // MVP = proj * view * model
        let view = Mat4::look_at_rh(self.config.eye, self.config.target, self.config.up);
        let proj = cgmath::perspective(
            Deg(self.config.fovy),
            self.config.aspect,
            self.config.znear,
            self.config.zfar,
        );

        return proj * view;
    }

    pub fn update_view(&mut self) {
        let view_pos = Vec4::new(self.config.eye.x, self.config.eye.y, self.config.eye.z, 0.0);
        let view_proj = self.build_view_projection_matrix();
        self.uniform.update_view_pos(view_pos);
        self.uniform.update_view_proj(view_proj);
    }

    pub fn setup(&mut self, device: &Device, bind_group_layout: &wgpu::BindGroupLayout) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: &self.uniform.as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     label: Some("Camera bind group layout"),
        //     entries: &[wgpu::BindGroupLayoutEntry {
        //         binding: 0,
        //         visibility: wgpu::ShaderStages::VERTEX,
        //         ty: wgpu::BindingType::Buffer {
        //             ty: wgpu::BufferBindingType::Uniform,
        //             has_dynamic_offset: false,
        //             min_binding_size: None,
        //         },
        //         count: None,
        //     }],
        // });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        self.buffer.replace(buffer);
        // self.bind_group_layout.replace(bind_group_layout.clone());
        self.bind_group.replace(bind_group);
    }

    pub fn update(&mut self) {
        self.controller.update_camera_config(&mut self.config);
        self.update_view();
    }
}
