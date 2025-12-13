module extension;

import path;
import ray;
import queue;
import hit;

struct BVHNode {
  float3 lb;
  float3 ub;
  uint left;
  uint right;
  uint is_leaf;
  uint start;
  uint end;
}

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

void rayTriIntersect(uint3 face, Ray ray, out float2 uv, out float t, out Hit h) {
  let p0 = tri_positions[face.x];
  let p1 = tri_positions[face.y];
  let p2 = tri_positions[face.z];

  let n0 = tri_normals[face.x];
  let n1 = tri_normals[face.y];
  let n2 = tri_normals[face.z];

  uv = float2(0.0);
  t = -1;
  h = Hit();
  
  let e1 = p1 - p0;
  let e2 = p2 - p0;
  let q = cross(ray.dir, e2);
  let alpha = dot(e1, q);
  if (alpha > -(10e-5) && alpha < 10e-5) {
    return;
  }
  let f = 1.0 / alpha;
  let s = ray.pos - p0;
  let u = f * dot(s, q);
  if (u < 0.0) {
    return;
  }
  let r = cross(s, e1);
  let v = f * dot(ray.dir, r);
  if (v < 0.0 || u + v > 1.0) {
    return;
  }
  t = f * dot(e2, r);

  // h.norm = normalize(cross(e1, e2));
  h.norm = n0 * (1.0 - u - v) + n1 * u + n2 * v;
  h.pos = ray.pos + t * ray.dir;
  return;
}

void rayBoxIntersect(Ray ray, float3 lb, float3 ub, out float t) {
  t = -1.0;
  var t_min = float.minValue;
  var t_max = float.maxValue;
  let a_c = (lb + ub) / 2.0;
  let p = a_c - ray.pos;
  let uvw = normalize(ub - lb);
  let h = (ub - lb) / 2;
  for (int i = 0; i < 3; i++) {
    let e = dot(uvw[i], p);
    let f = dot(uvw[i], ray.dir);
    if (abs(f) > 10e-20) {
      var t1 = (e+h[i])/f;
      var t2 = (e-h[i])/f;
      if (t1 > t2) {
        let temp = t2;
        t2 = t1;
        t1 = temp;
      }
      if (t1 > t_min) {
        t_min = t1;
      }
      if (t2 < t_max) {
        t_max = t2;
      }
      if (t_min > t_max) {
        return;
      }
      if (t_max < 0) {
        return;
      }
    } else if (- e - h[i] > 0 || -e + h[i] < 0) {
      return;
    }
  }
  if (t_min > 0) {
    t = t_min;
  }
  else {
    t = t_max;
  }
  return;
}

void rayBVHIntersect(Ray ray, uint bvh_node, out float2 uv, out float t, out Hit h) {
  let root = bvh_node;
  var current = bvh_node;
  t = float.maxValue;
  h = Hit();
  uv = float2(0.0);
  do {
    let node = bvh_nodes[current];
    if (node.is_leaf == 0) {
      // Internal node.
      var tp = -1.0;
      rayBoxIntersect(ray, node.lb, node.ub, tp);
      if (tp < 0.0) {
        // Failed to hit, skip to right
        current = node.right;
      } else {
        current = node.left;
      }
    } else {
      // Leaf node. Iterate primitives and check:
      for (int p = node.start; p < node.end; p++) {
        let face = tri_faces[p];
        float t2;
        Hit h2;
        float2 uv2;
        rayTriIntersect(face, ray, uv2, t2, h2);
        if (t2 >= 0 && t2 < t) {
          t = t2;
          h = h2;
          uv = uv2;
        }
      }
      current = node.right;
    }
  } while (root != current);
}

[[vk::binding(0,0)]] RWStructuredBuffer<Path> paths;
[[vk::binding(3,0)]] RWStructuredBuffer<HitData> hits;

[[vk::binding(0,1)]] RWStructuredBuffer<QueueHeader> qh_extension;
[[vk::binding(1,1)]] RWStructuredBuffer<uint> qd_extension;

[[vk::binding(0,2)]] StructuredBuffer<Sphere> primitives;

[[vk::binding(0,3)]] StructuredBuffer<BVHNode> bvh_nodes;
[[vk::binding(1,3)]] StructuredBuffer<float3> tri_positions;
[[vk::binding(2,3)]] StructuredBuffer<uint3> tri_faces;
[[vk::binding(3,3)]] StructuredBuffer<float3> tri_normals;

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

  float t = float.maxValue;
  Hit h;
  for (uint pid = 0; pid < primitives.getCount(); pid++) {
    float t2;
    Hit h2;
    Sphere s = primitives[pid];
    s.raySphereIntersect(path.ray, t2, h2);

    if (t2 >= 0 && t2 <= t) {
      hits[index].mat = 1;
      t = t2;
      h = h2;
    }
  }

  for (uint bid = 0; bid < 1; bid++) {
    float t2;
    Hit h2;
    float2 uv;
    // rayTriIntersect(tri_faces[bid], path.ray, uv, t2, h2);
    rayBVHIntersect(path.ray, bid, uv, t2, h2);

    if (t2 >= 0 && t2 <= t) {
      hits[index].mat = 2;
      t = t2;
      h = h2;
    }
  }


  if (t < float.maxValue) {
    // paths[index].rad += float3(dot(h.norm,float3(0.0,0.0,-1.0))) / (5-path.bounces);
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
