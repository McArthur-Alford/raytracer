module render;

struct VertexInput {
  float3 position;
  float3 texCoords;
}

struct VertexOutput {
  float4 clipPos : SV_Position;
  float2 texCoords;
}

[shader("vertex")]
VertexOutput vertexMain(
  VertexInput model
) {
  VertexOutput outVal;
  outVal.texCoords = model.texCoords.xy;
  outVal.clipPos = float4(model.position, 1.0);
  return outVal;
}

[shader("fragment")]
float4 fragmentMain(
  VertexOutput input,
  Texture2D<float4> tDiffuse,
  SamplerState sDiffuse,
) : SV_Target0 {
  return tDiffuse.Sample(sDiffuse, input.texCoords);
}
