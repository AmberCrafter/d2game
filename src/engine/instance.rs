use cgmath::Matrix;

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

#[allow(unused)]
impl Instance {
    pub fn as_model(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            &(*(self.as_model().as_ptr()
                as *const [u8; core::mem::size_of::<cgmath::Matrix4<f32>>()]))
        }
    }
}
