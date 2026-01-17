#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct MetallicData {
    pub albedo: [f32; 4],
    pub fuzz: f32,
    pub _pad: [u32; 3],
}
