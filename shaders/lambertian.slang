module lambertian;

import camera;
import path;
import queue;

[[vk::binding(0,0)]] RWStructuredBuffer<Path> paths;
[[vk::binding(3,0)]] RWStructuredBuffer<HitData> hits;

[[vk::binding(0,1)]] RWStructuredBuffer<QueueHeader> qh_material;
[[vk::binding(1,1)]] RWStructuredBuffer<uint> qd_material;

[[vk::binding(0,2)]] RWStructuredBuffer<QueueHeader> qh_extension;
[[vk::binding(1,2)]] RWStructuredBuffer<uint> qd_extension;

[shader("compute")]
[numthreads(256,1,1)]
void newRayMain(uint3 threadId : SV_DispatchThreadID) {
  uint index;
  if (queueRead(qh_material, qd_material, index) == -1) {
    return;
  }

  let path = &paths[index];
  let hit = &hits[index];
  // path.rad += -dot(path.ray.dir, path.hit.norm) * float3(2.0) / (9-path.bounces);
  path.throughput *= float3(0.8,0.8,0.95);

  path.ray.pos = hit.hit.pos + hit.hit.norm * 0.01;

  path.ray.dir = path.ray.dir - 2.0 * dot(path.ray.dir, hit.hit.norm) * hit.hit.norm;
  
  queuePush(qh_extension, qd_extension, index);
}
