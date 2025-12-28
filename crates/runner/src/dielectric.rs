#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct DielectricData {
    pub albedo: [f32; 4],
    pub ir: f32,
    pub _pad: [u32; 3],
}
