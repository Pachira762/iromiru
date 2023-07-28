#include "common.hlsl"

cbuffer Params : register(b0) {
    uint4 Rect;
    float4 Mask;
    uint Mode;
};

struct PsInput {
    float4 position : SV_Position;
};

PsInput ViewVs(uint id: SV_VertexID) {
    static const float2 Positions[6] = {
        float2(-1, +1),
        float2(+1, +1),
        float2(-1, -1),
        float2(+1, +1),
        float2(+1, -1),
        float2(-1, -1),
    };

    PsInput output;

    output.position = float4(Positions[id], 0.999999f, 1.f);

    return output;
}

float4 ViewRgb(float3 color)
{
    return float4(Mask.rgb * color, 1.f);
}

float4 ViewHue(float3 color)
{
    float3 hsl = ToHsl(color);
    return float4(HslToRgb(hsl.x, 0.8f, 0.8f), 1.f);
}

float4 ViewSaturation(float3 color)
{
    float saturation = ToHsl(color).y;
    return float4(HslToRgb(lerp(-120.f, 60.f, saturation) / 360.f, 0.8f, saturation), 1.f);
}

float4 ViewBrightness(float3 color)
{
    return float4(ToLuma(color).xxx, 1.f);
}

float4 ViewPs(PsInput input) : SV_Target {
    float3 color = Tex[Rect.xy + uint2(input.position.xy)].rgb;

    switch (Mode) {
    case 1: return ViewRgb(color);
    case 2: return ViewHue(color);
    case 3: return ViewSaturation(color);
    case 4: return ViewBrightness(color);
    default: return float4(color, 1.f);
    }
}
