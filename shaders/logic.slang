module logic;

import path;
import queue;
import sample;
import camera;

[[vk::binding(0,0)]] RWStructuredBuffer<Path> paths;
[[vk::binding(2,0)]] RWStructuredBuffer<HitData> hits;

[[vk::binding(0,1)]] RWStructuredBuffer<QueueHeader> qh_new_ray;
[[vk::binding(1,1)]] RWStructuredBuffer<uint> qd_new_ray;

[[vk::binding(0,2)]] RWStructuredBuffer<uint> output;

[[vk::binding(0,3)]] ConstantBuffer<uint2> dims;

[[vk::binding(0,4)]] RWStructuredBuffer<uint> sample_index;
[[vk::binding(1,4)]] RWStructuredBuffer<SampleData> sample_data;
[[vk::binding(2,4)]] RWByteAddressBuffer sample_mean;
[[vk::binding(3,4)]] RWStructuredBuffer<float4> sample_std;

[[vk::binding(0,5)]] ConstantBuffer<Camera> camera;

[[vk::binding(0,6)]] RWStructuredBuffer<QueueHeader> qh_lambertian;
[[vk::binding(1,6)]] RWStructuredBuffer<uint> qd_lambertian;
[[vk::binding(2,6)]] RWStructuredBuffer<QueueHeader> qh_metallic;
[[vk::binding(3,6)]] RWStructuredBuffer<uint> qd_metallic;
[[vk::binding(4,6)]] RWStructuredBuffer<QueueHeader> qh_dielectric;
[[vk::binding(5,6)]] RWStructuredBuffer<uint> qd_dielectric;
[[vk::binding(6,6)]] RWStructuredBuffer<QueueHeader> qh_emissive;
[[vk::binding(7,6)]] RWStructuredBuffer<uint> qd_emissive;

[shader("compute")]
[numthreads(1,1,1)]
void logicStart(uint3 threadId : SV_DispatchThreadID) {
  if (threadId.x == threadId.y == threadId.z == 0) {
    queueReset(qh_new_ray);
    queueReset(qh_lambertian);
    queueReset(qh_metallic);
    queueReset(qh_dielectric);
    queueReset(qh_emissive);
    return;
  }
}

[shader("compute")]
[numthreads(64,1,1)]
void maintainSamples(uint3 threadId : SV_DispatchThreadID) {
  if (threadId.x > sample_data.getCount()) {
    return;
  }

  let wgc = WorkgroupCount().x;
  let wgs = WorkgroupSize().x;
  let work = sample_data.getCount() / (wgc * wgs);

  for (int i = threadId.x * work; i < threadId.x * work + work; i++) {
    if (camera.changed != 0) {
      sample_data[i].samples = 1;
      sample_data[i].flags = 0;
      sample_mean.InterlockedExchange(i * sizeof(uint4) + 0 * sizeof(uint), 0);
      sample_mean.InterlockedExchange(i * sizeof(uint4) + 1 * sizeof(uint), 0);
      sample_mean.InterlockedExchange(i * sizeof(uint4) + 2 * sizeof(uint), 0);
      sample_mean.InterlockedExchange(i * sizeof(uint4) + 3 * sizeof(uint), 0);
    }
  }

}

[shader("compute")]
[numthreads(64,1,1)]
void maintainActiveSamples(uint3 threadId : SV_DispatchThreadID) {
  // Pull of the sample completion queue and deactivates those samples
  // swap-removing from the active sample list.
}

groupshared uint m;

[shader("compute")]
[numthreads(64,1,1)]
void logicMain(uint3 threadId : SV_DispatchThreadID) {
  if (threadId.x > paths.getCount()) {
    return;
  }

  let path = &paths[threadId.x];
  let hit = &hits[threadId.x];

  if (path.sampled == 0) {
    // Path has not been sampled
    // Pull a sample from the pool:
    uint sample_id;
    InterlockedAdd(sample_index[0], 1, sample_id);
    sample_id %= sample_data.getCount();
    let sample_data = sample_data[sample_id];
    
    if (sample_data.flags == 1) {
      // Skip this sample, resample it next frame i guess lol
      return;
    }

    // Generate the path starting conditions:
    path.bounces = 256;
    path.throughput = float3(1.0);
    path.rad = float3(0.0);
    path.sampled = 1;
    path.terminated = 0;
    path.sample_id = sample_id;

    hit.hit.prim = uint(-1);
    hit.hit.front_face = 0;
    hit.hit.norm = float3(0.0);
    hit.hit.pos = float3(0.0);
    hit.mat = 0;
    hit.mat_data = 0;

    queuePush(qh_new_ray, qd_new_ray, threadId.x);
  } else if (path.terminated == 1) {
    // Path is terminated, update the sample data:
    var samples = 0;
    InterlockedAdd(sample_data[path.sample_id].samples, 1, samples);

    sample_mean.InterlockedAdd(path.sample_id * sizeof(uint4) + 0 * sizeof(uint),
                                uint(path.rad.x * 1024.0));
    sample_mean.InterlockedAdd(path.sample_id * sizeof(uint4) + 1 * sizeof(uint),
                                uint(path.rad.y * 1024.0));
    sample_mean.InterlockedAdd(path.sample_id * sizeof(uint4) + 2 * sizeof(uint),
                                uint(path.rad.z * 1024.0));

    // Write the sampled radiance to the output buffer:
    let out_pos = sample_data[path.sample_id].out_pos;
    let out_idx = out_pos.x + out_pos.y * dims.x;

    float3 rad = float3(sample_mean.Load3(path.sample_id * sizeof(uint4))) / float(1024 * samples);
    rad *= exp2(-4.0);

    // float3 max_comp = max3(rad.r, rad.g, rad.b);
    // rad = max(rad, 0.01 * max_comp);
    
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    rad = saturate((rad*(a*rad+b))/(rad*(c*rad+d)+e));

    float3 rgb = (rad < 0.0031308) ? rad * 12.92 : pow(rad, 1.0 / 2.4) * 1.055 - 0.55;
    output[out_idx] = pack_rgb(rgb);

    // Re-mark the path to be sampled:
    path.sampled = 0;
  } else {
    // Path is not yet terminated this is where we would give it to the material it hit
    // but for now im just putting it straight back on the extension queue.
    switch (hit.mat) {
      case 0:
        // No material??? Shouldn't get here.
        // output[threadId.x + threadId.y * dims.x] = pack_rgb(float3(0.0,0.0,255.0));
        break;
      case 1:
        queuePush(qh_lambertian, qd_lambertian, threadId.x);
        break;
      case 2:
        queuePush(qh_metallic, qd_metallic, threadId.x);
        break;
      case 3:
        queuePush(qh_dielectric, qd_dielectric, threadId.x);
        break;
      case 4:
        queuePush(qh_emissive, qd_emissive, threadId.x);
        break;
      default:
        break;
    }
  }
}

uint pack_rgb(float3 color) {
    color = saturate(color);
    let r = uint(color.r * 255.0) << 16;
    let g = uint(color.g * 255.0) << 8;
    let b = uint(color.b * 255.0) << 0;
    return r | g | b;
}

