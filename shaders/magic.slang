struct VertexData {
  float4 position;
  float4 normal;
  float4 uv;
};

[[vk::binding(0,0)]] RWStructuredBuffer<uint> output;
[[vk::binding(1,0)]] RWStructuredBuffer<VertexData> vertices[];

[shader("compute")]
[numthreads(8,8,1)]
void computeMain(uint3 threadId : SV_DispatchThreadID) {
  output[threadId.x + threadId.y * 512] = uint(vertices[0][0].position.x);
}
