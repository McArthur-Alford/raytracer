module path;

import ray;
import hit;

public struct Path {
  public Ray ray;
  public float3 rad;
  public float3 throughput;
  public float2 screen_pos;
  public uint32_t terminated;
  public uint32_t generated;
  public uint32_t bounces;
  // 0 is meaningless
  // 1 is a lambertian
  public uint32_t mat;
  public Hit hit;
};
