#include "common.hlsl"

#ifdef CREATE

cbuffer Params : register(b0) {
    uint4 Rect;
    uint Mode;
};

RWBuffer<uint> HistogramBuf[3] : register(u0);

void CreateRgbHistogram(float3 color)
{
    uint3 rgb = uint3(255.f * color);
    InterlockedAdd(HistogramBuf[0][rgb.r], 1);
    InterlockedAdd(HistogramBuf[1][rgb.g], 1);
    InterlockedAdd(HistogramBuf[2][rgb.b], 1);
}

void CreateHueHistogram(float3 color)
{
    uint3 hsv = 255.f * ToHsv(color);
    InterlockedAdd(HistogramBuf[0][hsv.x], 1);
}

void CreateSaturationHistogram(float3 color)
{
    uint3 hsv = 255.f * ToHsv(color);
    InterlockedAdd(HistogramBuf[0][hsv.y], 1);
}

void CreateBrightnessHistogram(float3 color)
{
    uint3 hsv = 255.f * ToHsv(color);
    InterlockedAdd(HistogramBuf[0][hsv.z], 1);
}

#define THREADS 8
[numthreads(THREADS, THREADS, 1)]
void CreateCs(uint2 id: SV_DispatchThreadID, uint gindex: SV_GroupIndex)
{
    uint2 position = Rect.xy + id;

    if (all(position < Rect.zw)) {
        float3 color = Tex[position].rgb;

        switch (Mode) {
        case 1: CreateRgbHistogram(color); break;
        case 2: CreateHueHistogram(color); break;
        case 3: CreateSaturationHistogram(color); break;
        case 4: CreateBrightnessHistogram(color); break;
        }
    }
}

#endif // CREATE

#ifdef DRAW

cbuffer Params : register(b0) {
    float4 Color;
    float2 Scale;
    float InvPixelCount;
    uint Mode;
};

Buffer<uint> HistogramBuf : register(t1);

struct VertexOut {
    float4 position : SV_Position;
    float4 color : COLOR;
};

VertexOut FillVs(uint vid: SV_VertexID)
{
    uint index = vid / 2;
    uint count = HistogramBuf[index];
    bool bottom = vid % 2 == 0;

    float x = 2.f * (float(index) / 255.f) - 1.f;
    float y = bottom ? -1.f : (InvPixelCount * count - 1.f);

    VertexOut output;
    output.position = float4(x, y, 0.f, 1.f);

    if (Mode != 2) {
        output.color = Color;
    } else {
        output.color = float4(HslToRgb((float)index / 256.f, 0.8f, 0.8f), Color.a);
    }

    return output;
}

VertexOut LineVs(uint vid: SV_VertexID)
{
    uint index = vid;
    uint count = HistogramBuf[index];

    float x = 2.f * (float(index) / 255.f) - 1.f;
    float y = InvPixelCount * count - 1.f;

    VertexOut output;
    output.position = float4(x, y, 0.f, 1.f);

    if (Mode != 2) {
        output.color = Color;
    } else {
        output.color = float4(HslToRgb((float)index / 256.f, 0.8f, 0.6f), Color.a);
    }

    return output;
}

float4 DrawPs(VertexOut input) : SV_Target
{
    return input.color;
}

#endif // DRAW