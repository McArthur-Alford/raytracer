module lambertian;

import camera;
import path;
import queue;
import light;
import random;
import bvh;
import tlas;
import material;

public struct LambertianData : IMaterial {
  float4 albedo;

  public float eval_brdf(float3 wi, float3 wo, float3 n) {
    return select(dot(wo, n) >= 0.0, 1.0 / 6.2831, 0.0);
  }

  public bool scatter(out float3 wi, float3 wo, float3 n, inout uint4 random_state) {
    RandomResult r = Random(random_state);
    RandomResult r2 = Random(r.state);
    RandomResult r3 = Random(r2.state);
    random_state = r2.state;

    float3 scatter = n + normalize(1.0 - 2.0 * float3(r.value, r2.value, r3.value));
    wi = select(length(scatter) < 1e-2, n, scatter) - wo;
    return true;
  }

  public bool colour(out float4 colour) {
    colour = albedo;
    return true;
  }

  public bool emit(out float4 colour) {
    colour = float4(0.0);
    return false;
  }
};

[[vk::binding(0,0)]] RWStructuredBuffer<Path> paths;
[[vk::binding(1,0)]] RWStructuredBuffer<uint4> random_states;
[[vk::binding(2,0)]] RWStructuredBuffer<HitData> hits;

[[vk::binding(0,1)]] RWStructuredBuffer<QueueHeader> qh_material;
[[vk::binding(1,1)]] RWStructuredBuffer<uint> qd_material;

[[vk::binding(0,2)]] RWStructuredBuffer<QueueHeader> qh_extension;
[[vk::binding(1,2)]] RWStructuredBuffer<uint> qd_extension;

[[vk::binding(0,3)]] StructuredBuffer<LambertianData> mat_datas;

[[vk::binding(0,4)]] StructuredBuffer<uint> light_samples;

[[vk::binding(0,5)]] StructuredBuffer<BVHNode> blas_nodes;
[[vk::binding(1,5)]] StructuredBuffer<float3> tri_positions;
[[vk::binding(2,5)]] StructuredBuffer<uint3> tri_faces;
[[vk::binding(3,5)]] StructuredBuffer<float3> tri_normals;
[[vk::binding(4,5)]] StructuredBuffer<uint> blas_roots;

[[vk::binding(0,6)]] StructuredBuffer<BVHNode> tlas_nodes;
[[vk::binding(1,6)]] StructuredBuffer<uint> tlas_instance_ids;
[[vk::binding(2,6)]] StructuredBuffer<AABB> tlas_aabbs;

[shader("compute")]
[numthreads(64,1,1)]
void computeMain(uint3 threadId : SV_DispatchThreadID) {
  uint index;
  if (queueRead(qh_material, qd_material, index) == -1) {
    return;
  }

  process_material(
    index,
  
    paths,
    random_states,
    hits,

    qh_material,
    qd_material,

    qh_extension,
    qd_extension,

    mat_datas,

    light_samples,

    blas_nodes,
    tri_positions,
    tri_faces,
    tri_normals,
    blas_roots,

    tlas_nodes,
    tlas_instance_ids,
    tlas_aabbs
  );
}
