// shade.slang
//
// Performs shading of samples waiting in the shade queue.
// Shading entails computing the changes to sample state by
// using the most recent hit record.
// Shading also determines ray termination or bouncing,
// and will enqueue the sample for another extension accordingly.
module shade;

import common;
import scene;
import pathtracer;
import random;
import queue;
import bvh;

[shader("compute")]
[numthreads(64,1,1)]
void shadeMain(uint3 threadId : SV_DispatchThreadID) {
  let idx = queueRead(shade_qh, shade_qd);
  if (idx < 0) {
    return;
  }

  let s = &samples[idx];
  let h = &extension_hit_records[idx];
  let wi = &extension_rays[idx];
  Instance instance = instances[h.instance_id];

  // Lambertian scatter for now:
  float r1 = random_gen(randoms, 0);
  float r2 = random_gen(randoms, 0);
  float r3 = random_gen(randoms, 0);
  float3 n = h.vert.normal.xyz;
  n *= h.front_face != 0 ? 1.0 : -1.0;
  float3 scatter = n + normalize(1.0 - 2.0 * float3(r1,r2,r3));
  wi.dir = normalize(select(length(scatter) < 1e-6, n, scatter));
  wi.pos = h.vert.position.xyz;

  // Update throughput:
  let mat = materials[instance.material];
  s.throughput *= mat.colour.xyz;
  s.bounces -= 1;

  if (s.bounces == 0) {
    queuePush(terminate_qh, terminate_qd, idx);
  } else {
    queuePush(extension_qh, extension_qd, idx);
  }
}
