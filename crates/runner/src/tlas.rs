// use crate::bvh::AABB;

// #[derive(Clone, Copy, Debug, Default)]
// pub struct TLASNode {
//     bounds: AABB,
//     left: usize,
//     right: usize,
//     is_leaf: bool,
//     blas: usize, // Reference to a single blas node if this is a leaf
// }

// impl TLASNode {
//     fn bounds(&self) -> AABB {
//         self.bounds
//     }
// }

// #[derive(Debug)]
// pub struct BVH {
//     pub nodes: Vec<TLASNode>,
//     pub tri_positions: Vec<Vec3>,
//     pub tri_faces: Vec<UVec3>,
//     pub tri_normals: Vec<Vec3>,
// }
