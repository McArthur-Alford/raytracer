use glam::UVec3;
use glam::Vec3;
use glam::Vec4Swizzles;
use itertools::Itertools;
use wgpu::util::DeviceExt;

use crate::blas::BLAS;
use crate::blas::BLASData;
use crate::bvh::AABB;
use crate::bvh::AABBGPU;
use crate::bvh::BVH;
use crate::bvh::BVHNode;
use crate::bvh::BVHNodeGPU;
use crate::instance::Instance;
use crate::instance::Transform;
use crate::mesh::Mesh;

#[derive(Debug)]
pub struct TLAS {
    pub nodes: Vec<BVHNode>,
    pub blas: Vec<usize>, // Which blas this points at
    pub aabbs: Vec<AABB>, // Its AABB (transformed)
}

impl BVH for TLAS {
    fn elem_bounds(&self, elem: usize) -> AABB {
        self.aabbs[elem]
    }

    fn elem_centroid(&self, elem: usize) -> Vec3 {
        (self.aabbs[elem].lb + self.aabbs[elem].ub) / 2.0
    }

    fn elem_swap(&mut self, elem: usize, elem2: usize) {
        self.aabbs.swap(elem, elem2);
        self.blas.swap(elem, elem2);
    }

    fn node(&self, idx: usize) -> &BVHNode {
        &self.nodes[idx]
    }

    fn push_node(&mut self, node: BVHNode) -> usize {
        let i = self.nodes.len();
        self.nodes.push(node);
        i
    }

    fn node_mut(&mut self, idx: usize) -> &mut BVHNode {
        &mut self.nodes[idx]
    }

    fn node_bounds(&self, idx: usize) -> AABB {
        self.nodes[idx].bounds
    }
}

impl TLAS {
    pub fn new(blases: &Vec<BLAS>, instances: &Vec<Instance>) -> Self {
        let aabbs = instances
            .iter()
            .map(|i| {
                let aabb = blases[i.mesh as usize].node_bounds(0);
                let translate = glam::Mat4::from_translation(i.transform.translation);
                let rotate = glam::Mat4::from_rotation_x(i.transform.rotation.x).mul_mat4(
                    &glam::Mat4::from_rotation_y(i.transform.rotation.y)
                        .mul_mat4(&glam::Mat4::from_rotation_z(i.transform.rotation.z)),
                );
                let scale = glam::Mat4::from_scale(i.transform.scale);
                let m = translate.mul_mat4(&rotate.mul_mat4(&scale));
                let aabb = AABB {
                    lb: m.mul_vec4(aabb.lb.extend(0.0)).xyz(),
                    ub: m.mul_vec4(aabb.ub.extend(0.0)).xyz(),
                };
                let aabb = AABB {
                    lb: aabb.lb.min(aabb.ub),
                    ub: aabb.lb.max(aabb.ub),
                };
                aabb
            })
            .collect_vec();

        let mut bvh = TLAS {
            nodes: vec![BVHNode {
                is_leaf: true,
                bounds: AABB::default(),
                start: 0,
                end: instances.len(),
                ..Default::default()
            }],
            blas: instances.iter().map(|i| i.mesh as usize).collect(),
            aabbs,
        };

        bvh.initialize();

        bvh
    }
}

pub struct TLASData {
    pub nodes: Vec<BVHNodeGPU>,
    pub bindgroup: wgpu::BindGroup,
    pub bindgroup_layout: wgpu::BindGroupLayout,
}

impl TLASData {
    pub fn new(device: &wgpu::Device, tlas: TLAS) -> Self {
        let nodes = tlas
            .nodes
            .into_iter()
            .map(|n| BVHNodeGPU::from(n))
            .collect_vec();

        let node_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BVHNode buffer"),
            contents: bytemuck::cast_slice(&nodes),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let blas_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BLAS buffer"),
            contents: bytemuck::cast_slice(&tlas.blas),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let aabb_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("AABB buffer"),
            contents: bytemuck::cast_slice(
                &tlas
                    .aabbs
                    .into_iter()
                    .map(|aabb| AABBGPU::from(aabb))
                    .collect_vec(),
            ),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Triangles bindgroup layout descriptor"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
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

        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Triangles bindgroup"),
            layout: &bindgroup_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: node_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: blas_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: aabb_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            nodes,
            bindgroup,
            bindgroup_layout,
        }
    }
}
