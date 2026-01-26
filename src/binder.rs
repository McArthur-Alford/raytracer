use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::{
    app::BevyApp,
    bvh::AABB,
    instance::Instance,
    material::{Material, MaterialId, MaterialServer},
    mesh::{MeshId, MeshServer},
    pathtracer::{Pathtracer, PathtracerOutput},
    render_resources::{RenderDevice, RenderQueue},
    schedule,
    tlas::TLAS,
    transform::Transform,
};

pub fn initialize(app: &mut BevyApp) {
    app.world
        .get_resource_or_init::<Schedules>()
        .add_systems(schedule::Update, binder_system);
}

#[derive(Resource)]
struct BinderLocal {
    tlas_cache: TLAS,
    tlas_regenerate: bool,
}

impl Default for BinderLocal {
    fn default() -> Self {
        Self {
            tlas_cache: Default::default(),
            tlas_regenerate: true,
        }
    }
}

fn binder_system(
    objects: Query<(Ref<Transform>, Ref<MeshId>, &MaterialId)>,
    removed_transforms: RemovedComponents<Transform>,
    removed_meshids: RemovedComponents<MeshId>,
    mesh_server: Res<MeshServer>,
    material_server: Res<MaterialServer>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut binder_local: Local<BinderLocal>,
) {
    // Buffers that i need:
    // Vertex buffer with all the vertices.
    //     BindingArray<VertexBuffer>
    //     position, normal, uv, etc
    // Index buffer of all indices for a mesh.
    //     BindingArray<UVec3> (stores faces)
    // Instance buffer of all instances with idx into vertex/index buffers
    //     Array<Instance>
    // transforms: Array<mat4x4>
    //     corresponds to instance buffer indexing
    // textures: Array<Texture>
    // samplers: Array<Sampler>
    // materials: Array<Material>
    // light_sources: array<instance id>
    //
    // blas: BindingArray<Array<BVHNode>>
    //     blas corresponding to equivalent index into vertex/index arrays
    //
    // tlas: Array<BVHNode>
    //     simple bvh with indexes into tlas instance mapping
    // tlas_instances: Array<uint>
    //     maps to corresponding instance, can then be used to get transform/blas/vert/index buffers

    let mut vertices = Vec::<wgpu::BufferBinding>::new();
    let mut indices = Vec::<wgpu::BufferBinding>::new();
    let mut blases = Vec::<wgpu::BufferBinding>::new();
    let mut aabbs = Vec::<AABB>::new();
    let mut materials = Vec::<Material>::new();
    let mut transforms = Vec::<Transform>::new();
    let mut instances = Vec::<Instance>::new();
    let mut meshes_id_map = HashMap::<MeshId, u32>::new();
    let mut materials_id_map = HashMap::<MaterialId, u32>::new();
    let mut light_sources = Vec::<u32>::new();

    if !removed_transforms.is_empty() && !removed_meshids.is_empty() {
        binder_local.tlas_regenerate = true;
    }

    // let mut textures = vec![];
    // let mut samplers = vec![];

    for (transform, mesh_id, mat_id) in objects {
        if transform.is_changed() || mesh_id.is_changed() {
            binder_local.tlas_regenerate = true;
        }

        let geometry_idx = if let Some(&idx) = meshes_id_map.get(&*mesh_id) {
            idx
        } else {
            let Some(md) = mesh_server.mesh_data(*mesh_id) else {
                continue;
            };

            vertices.push(md.vertex_buffer.as_entire_buffer_binding());
            indices.push(md.index_buffer.as_entire_buffer_binding());
            blases.push(md.blas_buffer.as_entire_buffer_binding());
            aabbs.push(md.aabb);

            let idx = (vertices.len() - 1) as u32;
            meshes_id_map.insert(*mesh_id, idx);
            idx
        };

        let mut emissive = false;
        let material_idx = if let Some(&idx) = materials_id_map.get(mat_id) {
            idx
        } else {
            let Some(material) = material_server.get(*mat_id) else {
                continue;
            };

            if material.emissive.length() > 0.0 || material.emissive_texture > 0 {
                emissive = true;
            }
            materials.push(*material);

            let idx = (materials.len() - 1) as u32;
            materials_id_map.insert(*mat_id, idx);
            idx
        };

        transforms.push(*transform);
        let transform_idx = (transforms.len() - 1) as u32;

        let instance = Instance {
            transform_idx,
            geometry_idx,
            material_idx,
        };
        instances.push(instance);

        if emissive {
            light_sources.push((instances.len() - 1) as u32);
        }
    }

    if instances.is_empty() {
        // Gonna have a hard time binding this :)
        return;
    }

    if light_sources.is_empty() {
        // what to do here? Could insist that all indexes are >0 i guess
        // TODO properly support having no light sources lol
        light_sources.push(u32::MAX);
    }

    if binder_local.tlas_regenerate {
        // Regenerate the TLAS only when transforms or meshes have changed
        binder_local.tlas_regenerate = false;
        binder_local.tlas_cache = TLAS::new(&aabbs, &transforms, &instances);
    }
    let tlas = &binder_local.tlas_cache;

    // let bg_layout = device
    //     .0
    //     .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //         label: Some("Binder Test"),
    //         entries: &[wgpu::BindGroupLayoutEntry {
    //             binding: 0,
    //             visibility: wgpu::ShaderStages::COMPUTE,
    //             ty: wgpu::BindingType::Buffer {
    //                 ty: wgpu::BufferBindingType::Storage { read_only: false },
    //                 has_dynamic_offset: false,
    //                 min_binding_size: None,
    //             },
    //             count: None,
    //         }],
    //     });

    // let bg = device.0.create_bind_group(&wgpu::BindGroupDescriptor {
    //     label: Some("magic"),
    //     layout: &bg_layout,
    //     entries: &[wgpu::BindGroupEntry {
    //         binding: 0,
    //         resource: buffer.1.source_buffer.as_entire_binding(),
    //     }],
    // });

    // let shader = device
    //     .0
    //     .create_shader_module(wgpu::include_spirv!(concat!(env!("OUT_DIR"), "/magic.spv")));

    // let pl_layout = device
    //     .0
    //     .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    //         label: Some("magic_pl_layout"),
    //         bind_group_layouts: &[&bg_layout],
    //         push_constant_ranges: &[],
    //     });

    // let pl = device
    //     .0
    //     .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    //         label: Some("magic_pl"),
    //         layout: Some(&pl_layout),
    //         module: &shader,
    //         entry_point: Some("main"),
    //         compilation_options: wgpu::PipelineCompilationOptions::default(),
    //         cache: None,
    //     });

    // let mut encoder = device
    //     .0
    //     .create_command_encoder(&wgpu::CommandEncoderDescriptor {
    //         label: Some("Magic Encoder"),
    //     });

    // let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
    //     label: Some("CS Descriptor"),
    //     ..Default::default()
    // });

    // compute_pass.set_pipeline(&pl);
    // compute_pass.set_bind_group(0, Some(&bg), &[]);
    // compute_pass.dispatch_workgroups(buffer.0.dims.0.div_ceil(8), buffer.0.dims.1.div_ceil(8), 1);

    // drop(compute_pass);

    // let command = encoder.finish();

    // queue.0.submit([command]);
}
