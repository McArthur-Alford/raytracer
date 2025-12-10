module extension;

import path;
import ray;
import queue;
import hit;

struct Sphere {
    float3 position;
    float radius;

  void raySphereIntersect(Ray ray, out float t, out Hit h) {
    t = -1;
    h = Hit();
    h.pos = float3(-1);
    let l = position - ray.pos;
    let s = dot(l, ray.dir);
    let l2 = dot(l, l);
    let r2 = radius * radius;
    if (s < 0 && l2 > r2) {
      return;
    }
    let m2 = l2 - s*s;
    if (m2 > r2) {
      return;
    }
    let q = sqrt(r2 - m2);
    if (l2 > r2) {
      t = s - q;
    } else {
      t = s + q;
    }
    h.pos = ray.pos + t * ray.dir;
    h.norm = (h.pos - position) / radius;
    // Norms from the inside are very broken... must investigate
    // if (l2 < r2) {
    //   h.norm *= -1;
    // }
  }
}

[[vk::binding(0,0)]] RWStructuredBuffer<Path> paths;
[[vk::binding(3,0)]] RWStructuredBuffer<HitData> hits;

[[vk::binding(0,1)]] RWStructuredBuffer<QueueHeader> qh_extension;
[[vk::binding(1,1)]] RWStructuredBuffer<uint> qd_extension;

[[vk::binding(0,2)]] StructuredBuffer<Sphere> primitives;

[shader("compute")]
[numthreads(1,1,1)]
void extensionReset(uint3 threadId : SV_DispatchThreadID) {
  if (threadId.x == threadId.y == threadId.z == 0) {
    queueReset(qh_extension);
    return;
  }
}

[shader("compute")]
[numthreads(256,1,1)]
void extensionMain(uint3 threadId : SV_DispatchThreadID) {
  uint index;
  if (queueRead(qh_extension, qd_extension, index) == -1) {
    return;
  }

  Path path = paths[index];

  // For now, just do a ray sphere intersect
  float t = float.maxValue;
  Hit h;
  for (uint pid = 0; pid < primitives.getCount(); pid++) {
    float t2;
    Hit h2;
    Sphere s = primitives[pid];
    s.raySphereIntersect(path.ray, t2, h2);

    if (t2 >= 0 && t2 <= t) {
      t = t2;
      h = h2;
    }
  }


  if (t < float.maxValue) {
    // paths[index].rad += float3(dot(h.norm,float3(0.0,0.0,-1.0))) / (5-path.bounces);
    hits[index].mat = 1;
    hits[index].hit = h;
    paths[index].bounces -= 1;
    paths[index].terminated = paths[index].terminated | paths[index].bounces == 0;
  } else {
    paths[index].rad += paths[index].throughput * float3(1.0);
    paths[index].terminated = 1;
    // paths[index].rad = float3(0.0);
    // paths[index].terminated = 1;
  }

}
