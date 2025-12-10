use wgpu::include_spirv;

use crate::{material::Material, path, queue};

pub struct LambertianPhase(Material);

impl LambertianPhase {
    pub fn new(
        device: &wgpu::Device,
        path_buffer: &path::Paths,
        material_queue: &queue::Queue,
        extension_queue: &queue::Queue,
        label: Option<&str>,
    ) -> Self {
        let compute_shader = device
            .create_shader_module(include_spirv!(concat!(env!("OUT_DIR"), "/lambertian.spv")));

        let mat = Material::new(
            device,
            compute_shader,
            path_buffer,
            material_queue,
            extension_queue,
            label,
        );

        Self(mat)
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        path_buffer: &path::Paths,
        material_queue: &queue::Queue,
        extension_queue: &queue::Queue,
    ) -> wgpu::CommandBuffer {
        self.0
            .render(device, path_buffer, material_queue, extension_queue)
    }
}
