module tlas;

public struct Transform {
  public float4 scale;
  public float4 rotation;
  public float4 translation;
}

public struct Instance {
  public Transform transform;
  public uint mesh;
  public uint material;
  public uint material_data;
}
