use itertools::Itertools;
use wgpu::{BindGroupLayoutEntry, util::DeviceExt};

pub struct Mesh {
    pub positions: Vec<[f32; 4]>,
    pub normals: Vec<[f32; 4]>,
    pub faces: Vec<[u32; 4]>,
}

impl Mesh {
    pub fn from_model(model: &tobj::Mesh) -> Self {
        // let mut positions = Vec::new();
        let positions = model
            .positions
            .chunks_exact(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], 0.0])
            .collect_vec();

        let mut normals = model
            .normals
            .chunks_exact(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], 0.0])
            .collect_vec();

        let faces = model
            .indices
            .chunks_exact(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], 0])
            .collect_vec();

        Self {
            positions,
            normals,
            faces,
        }
    }
}

pub struct Meshes {
    pub unified: Mesh,
    pub triangles_bindgroup: wgpu::BindGroup,
    pub triangles_bindgroup_layout: wgpu::BindGroupLayout,
}

impl Meshes {
    pub fn new(device: &wgpu::Device, meshes: Vec<Mesh>) -> Self {
        // Merge the meshes
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut faces = Vec::new();
        let mut offset = 0;

        for mut mesh in meshes {
            // TODO: Can be optimised using a hashmap/btreemap to re-id
            // all the vertices shared between meshes.
            positions.append(&mut mesh.positions);
            normals.append(&mut mesh.normals);
            faces.append(&mut mesh.faces.iter().map(|f| f.map(|i| i + offset)).collect());

            // Move offset to the end of this meshes vertices
            offset += mesh.positions.len() as u32;
        }

        let unified = Mesh {
            positions,
            normals,
            faces,
        };

        // Make the position buffer:
        let position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Position buffer"),
            contents: bytemuck::cast_slice(&unified.positions),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Make the faces buffer:
        let face_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Face buffer"),
            contents: bytemuck::cast_slice(&unified.faces),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Make the normals buffer:
        let normal_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Normal buffer"),
            contents: bytemuck::cast_slice(&unified.normals),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let triangles_bindgroup_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Triangles bindgroup layout descriptor"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let triangles_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Triangles bindgroup"),
            layout: &triangles_bindgroup_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: position_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: face_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: normal_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            unified,
            triangles_bindgroup,
            triangles_bindgroup_layout,
        }
    }
}

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
// pub struct Instance {
//     /// Each instance transforms that mesh
//     pub transform: Transform,
//     /// Each instance points at a mesh
//     pub mesh: u32,
//     /// Each instance has its own material
//     pub material: u32,
//     pub pad_0: [u32; 2], // to make the overall struct 16 byte aligned
// }

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
// /// This is an extra unnecessary bit of indirection
// /// in case I want per-mesh information in the future.
// pub struct Mesh {
//     /// The blas node in our blas buffer which represents the root
//     pub blas_root: u32,
// }

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
// pub struct AABB {
//     pub lb: [f32; 3],
//     pub _pad0: [u32; 1],
//     pub ub: [f32; 3],
//     pub _pad1: [u32; 1],
// }

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
// pub struct BLASNode {
//     /// Bounding box for the BLAS Node:
//     pub aabb: AABB,
//     /// Children for internal BLAS Node:
//     /// For left and right, 0 indicates no child, not the FIRST
//     /// root node (its never a child so this is fine).
//     pub left: u32,
//     pub right: u32,
//     /// Triangle index start and length in the triangle indices
//     /// buffer. Triangle indices buffer is Buff of UVec3 indices
//     /// into three of the vertex buffer positions.
//     pub start: u32,
//     pub size: u32,
// }

// pub struct BLAS {}

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
// pub struct Transform {
//     pub scale: [f32; 3],
//     pub _pad0: [u32; 1],
//     pub rotation: [f32; 3],
//     pub _pad1: [u32; 1],
//     pub translation: [f32; 3],
//     pub _pad2: [u32; 1],
// }

// GPU Buffers:
//
// - Buff<Instance>
// - Buff<Mesh>
//
// All BLASes concatted together (with updated indices)
// - Buff<BLASNode>
// - Buff<Vec3>     (for triangles)
// - Buff<Vertex>
