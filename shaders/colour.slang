module colour;

// HSV stuff ruthlessly stolen from here: https://chilliant.com/rgb2hsv.html
public float3 HUEtoRGB(in float H) {
  float R = abs(H * 6 - 3) - 1;
  float G = 2 - abs(H * 6 - 2);
  float B = 2 - abs(H * 6 - 4);
  return saturate(float3(R,G,B));
}

public float3 HSVtoRGB(in float3 HSV) {
  float3 RGB = HUEtoRGB(HSV.x);
  return ((RGB - 1) * HSV.y + 1) * HSV.z;
}
