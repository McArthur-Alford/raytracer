#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct EmissiveData {
    pub albedo: [f32; 4],
}
