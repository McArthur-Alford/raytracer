use wesl::include_wesl;
use wgpu::{include_spirv, util::DeviceExt};

use crate::{
    blas::{self, BLASData},
    emissive,
    instance::{Instance, Instances},
    material,
    mesh::Meshes,
    path, queue, tlas,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Sphere {
    pub position: [f32; 3],
    pub radius: f32,
}

pub struct ShadowPhase {
    pipeline: wgpu::ComputePipeline,
    reset_pipeline: wgpu::ComputePipeline,
}

impl ShadowPhase {
    pub fn new(
        device: &wgpu::Device,
        paths: &path::Paths,
        shadow_queue: &queue::Queue,
        blas_data: &blas::BLASData,
        tlas_data: &tlas::TLASData,
        light_sample_bind_group_layout: &wgpu::BindGroupLayout,
        material: &material::Material,
        instances: &Instances,
    ) -> Self {
        let compute_shader =
            device.create_shader_module(include_spirv!(concat!(env!("OUT_DIR"), "/shadow.spv")));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ShadowPhase Pipleline Layout"),
            bind_group_layouts: &[
                &paths.path_bind_group_layout,
                &shadow_queue.bind_group_layout,
                &light_sample_bind_group_layout,
                &blas_data.bindgroup_layout,
                &instances.bindgroup_layout,
                &tlas_data.bindgroup_layout,
                &material.data_bindgroup_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ShadowPhase Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("shadowMain"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        let reset_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ShadowPhase reset Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("shadowReset"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        Self {
            pipeline,
            reset_pipeline,
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        path_buffer: &path::Paths,
        shadow_queue: &queue::Queue,
        blas_data: &blas::BLASData,
        tlas_data: &tlas::TLASData,
        light_sample_bind_group: &wgpu::BindGroup,
        material: &material::Material,
        instances: &Instances,
    ) -> wgpu::CommandBuffer {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ExtensionPhase Encoder"),
        });

        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &path_buffer.path_bind_group, &[]);
        compute_pass.set_bind_group(1, &shadow_queue.bind_group, &[]);
        compute_pass.set_bind_group(2, light_sample_bind_group, &[]);
        compute_pass.set_bind_group(3, &blas_data.bindgroup, &[]);
        compute_pass.set_bind_group(4, &instances.bindgroup, &[]);
        compute_pass.set_bind_group(5, &tlas_data.bindgroup, &[]);
        compute_pass.set_bind_group(6, &material.data_bindgroup, &[]);
        compute_pass.dispatch_workgroups(shadow_queue.size.div_ceil(64), 1, 1);

        // Reset extension queue after done:
        compute_pass.set_pipeline(&self.reset_pipeline);
        compute_pass.set_bind_group(0, &path_buffer.path_bind_group, &[]);
        compute_pass.set_bind_group(1, &shadow_queue.bind_group, &[]);
        compute_pass.set_bind_group(2, light_sample_bind_group, &[]);
        compute_pass.set_bind_group(3, &blas_data.bindgroup, &[]);
        compute_pass.set_bind_group(4, &instances.bindgroup, &[]);
        compute_pass.set_bind_group(5, &tlas_data.bindgroup, &[]);
        compute_pass.set_bind_group(6, &material.data_bindgroup, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);

        drop(compute_pass);

        encoder.finish()
    }
}
