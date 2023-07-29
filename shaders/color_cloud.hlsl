#include "common.hlsl"

uint FlattenColorId(uint3 id) {
    return 256 * 256 * id.z + 256 * id.y + id.x;
}

float3 ColorIdToRgb(uint color_index)
{
    return float3(color_index & 0xff, (color_index & 0xff00) >> 8, (color_index & 0xff0000) >> 16) / float(0xff);
}

float3 RgbToPosition(float3 rgb)
{
    return 2.f * rgb - 1.f;
}

float3 HsvToPosition(float3 rgb)
{
    float3 hsv = ToHsv(rgb);
    float h = hsv.x;
    float s = hsv.y;
    float v = 2.f * hsv.z - 1.f;

    float x, z;
    sincos(2.f * Pi * h, z, x);
    x *= s;
    z *= s;
    float y = v;

    return float3(x, y, -z);
}

float3 HslToPosition(float3 rgb)
{
    float3 hsl = ToHsl(rgb);
    float h = hsl.x;             // 0 ~ 1
    float s = hsl.y;             // 0 ~ 1
    float l = 2.f * hsl.z - 1.f; // -1 ~ +1

    float r = sqrt(1.f - l * l);
    float s_max = 1.00001f - abs(l);

    float x, z;
    sincos(2.f * Pi * h, z, x);
    x *= r * (s / s_max);
    z *= r * (s / s_max);
    float y = l;

    return float3(x, y, -z);
}

float3 YuvToPosition(float3 rgb)
{
    static const float s = 2.f * -0.5f;
    static const float c = 2.f * 0.114572f;

    float3 yuv = 2.f * ToYuv(rgb) - 1.f;
    float u = -yuv.y;
    float v = yuv.z;
    float2 uv = float2(u, v);

    return float3(dot(uv, float2(c, -s)), yuv.x, dot(uv, float2(s, c)));
}

#ifdef COUNT

cbuffer Params : register(b0) {
    uint4 Rect;
};

RWBuffer<uint> CountBuf : register(u0);

#define THREAD 8

[numthreads(THREAD, THREAD, 1)]
void CountCs(uint2 group_id: SV_GroupID, uint2 id: SV_DispatchThreadID, uint group_index: SV_GroupIndex)
{
    uint2 position = Rect.xy + id;
    if (all(position < Rect.zw)) {
        float3 color = saturate(Tex[position].rgb / 1.f);
        uint3 color_id = 0xff * color;
        uint color_index = FlattenColorId(color_id);

#ifdef NAIVE
        InterlockedAdd(CountBuf[color_index], 1);
#elif defined CS_6_0
        while (true) {
            bool is_first_lane = WaveIsFirstLane();
            bool is_same_color_as_first_lane = WaveReadLaneFirst(color_index) == color_index;
            uint count = WaveActiveCountBits(is_same_color_as_first_lane);

            if (is_same_color_as_first_lane) {
                if (is_first_lane) {
                    InterlockedAdd(CountBuf[color_index], count);
                }
                break;
            }
        }
#elif defined CS_6_5
        uint4 same_color_lanes_mask = WaveMatch(color_index);
        if (WaveMultiPrefixCountBits(true, same_color_lanes_mask) == 0) { // first lane thas is this color index
            uint4 counts = countbits(same_color_lanes_mask);
            InterlockedAdd(CountBuf[color_index], counts.x + counts.y + counts.z + counts.w);
        }
#endif
    } 
}

#endif // COUNT

#ifdef COMPACT

struct IndirectCommand
{
    uint vertex_count;
    uint instance_count;
    uint vertex_offset;
    uint instance_offset;
};

Buffer<uint> CountBuf : register(t1);
RWBuffer<uint> PackedBuf : register(u0);
AppendStructuredBuffer<IndirectCommand> CommandBuf : register(u1);

#define GRID 8
#define THREAD 8
#define BLOCK 4

groupshared uint num_indices_in_group;

[numthreads(THREAD, THREAD, THREAD)]
void CompactCs(uint3 group_id: SV_GroupID, uint group_index: SV_GroupIndex, uint3 thread_id: SV_GroupThreadID, uint3 id: SV_DispatchThreadID)
{
    if (group_index == 0) {
        num_indices_in_group = 0;
    }
    GroupMemoryBarrierWithGroupSync();

    uint indices[BLOCK * BLOCK * BLOCK];
    uint num_indices = 0;

    uint color_index0 = group_id.z << 21 | group_id.y << 18 | group_id.x << 15;
    for (uint i = 0; i < BLOCK * BLOCK * BLOCK; ++i) {
        uint color_index = color_index0 | i << 9 | group_index;
        uint count = CountBuf[color_index];
        if (count != 0) {
            indices[num_indices] = color_index;
            ++num_indices;
        }
    }

    uint num_indices_in_wave = WaveActiveSum(num_indices);

    uint wave_offset;
    if (WaveIsFirstLane()) {
        InterlockedAdd(num_indices_in_group, num_indices_in_wave, wave_offset);
    }
    wave_offset = WaveReadLaneFirst(wave_offset);

    uint lane_offset = WavePrefixSum(num_indices);

    uint offset = color_index0 +  wave_offset + lane_offset;
    for (uint i = 0; i < num_indices; ++i) {
        PackedBuf[offset + i] = indices[i];
    }

    GroupMemoryBarrierWithGroupSync();

    if (group_index == 0 && num_indices_in_group != 0) {
        IndirectCommand command;
        command.vertex_count = 3;
        command.instance_count = num_indices_in_group;
        command.vertex_offset = 0;
        command.instance_offset = color_index0;
        CommandBuf.Append(command);
    }
}

#endif // COMPACT

#ifdef DRAW

cbuffer Params : register(b0) {
    float4x4 Projection;
    float2 Scale;
    uint NumPixels;
    uint ColorSpace;
};

Buffer<uint> CountBuf : register(t1);

struct VertexOut {
    float4 position : SV_Position;
    float4 color : COLOR;
    float2 uv : TEXCOORD;
};

float CalcSize(uint count)
{
    float rate = min(float(count) / float(NumPixels), 0.04f);
    return max(1.f * pow(rate, 1.f / 2.f), 0.25f / 256.f);
}

VertexOut GetVertexAttribute(uint index, float3 color, float3 center, float size)
{
    static const float2 Uvs[3] = {
        float2(-1.f, +3.f),
        float2(+3.f, -1.f),
        float2(-1.f, -1.f),
    };

    float3 position = center;
    position.xy += size * Uvs[index];
    position.xy *= 0.95f * Scale;
    position.z = 0.25f * position.z + 0.5f - 0.0001f * color.g;

    VertexOut vert;
    vert.position = float4(position, 1.0);
    vert.color = float4(color, 1.0);
    vert.uv = Uvs[index];

    return vert;
}

#ifdef MESH

#define GRID 8
#define ELEMS 32
#define STEPS 2

struct Payload {
    uint indices[GRID * GRID * GRID];
    uint num;
};

groupshared Payload payload;
groupshared uint num_indices_in_group;

[numthreads(4, 4, 4)]
void DrawAs(uint3 gid: SV_GroupID, uint gindex: SV_GroupIndex)
{
    if (gindex == 0) {
        num_indices_in_group = 0;
    }
    GroupMemoryBarrierWithGroupSync();

    uint indices[STEPS * STEPS * STEPS];
    uint num_indices = 0;

    uint color_index0 = gid.z << 19 | gid.y << 14 | gid.x << 9;
    for (uint i = 0; i < STEPS * STEPS * STEPS; ++i) {
        uint color_index = color_index0 | i << 6 | gindex;
        uint count = CountBuf[color_index];

        if (count > 0) {
            indices[num_indices] = color_index;
            ++num_indices;
        }
    }
    uint num_indices_in_wave = WaveActiveSum(num_indices);

    uint wave_offset;
    if (WaveIsFirstLane()) {
        InterlockedAdd(num_indices_in_group, num_indices_in_wave, wave_offset);
    }
    wave_offset = WaveReadLaneFirst(wave_offset);

    uint lane_offset = WavePrefixSum(num_indices);

    uint offset = wave_offset + lane_offset;
    for (uint i = 0; i < num_indices; ++i) {
        payload.indices[offset + i] = indices[i];
    }

    GroupMemoryBarrierWithGroupSync();

    payload.num = num_indices_in_group;
    uint num_dispatch = (num_indices_in_group + ELEMS - 1) / ELEMS;
    DispatchMesh(num_dispatch, 1, 1, payload);
}

#define PRIMITIVES (1 * ELEMS)
#define VERTICES (3 * ELEMS)
[outputtopology("triangle")]
[numthreads(ELEMS, 1, 1)]
void DrawMs(uint id: SV_DispatchThreadID, uint gid: SV_GroupID, uint tid: SV_GroupThreadID, in payload Payload payload, out vertices VertexOut vertes[VERTICES], out indices uint3 tris[PRIMITIVES])
{
    uint num_elems = min(payload.num - (ELEMS * gid), ELEMS);
    SetMeshOutputCounts(3 * num_elems, 1 * num_elems);

    if (id < payload.num ) {
        uint color_index = payload.indices[id];
        uint count = CountBuf[color_index];

        float3 color = ColorIdToRgb(color_index);
        float size = CalcSize(count);

        float3 center;
        switch (ColorSpace) {
        case 0: center = RgbToPosition(color); break;
        case 1: center = HsvToPosition(color); break;
        case 2: center = HslToPosition(color); break;
        case 3: center = YuvToPosition(color); break;
        }
        center = mul(Projection, float4(center, 1.f)).xyz;

        uint vindex = 3 * tid;
        vertes[vindex + 0] = GetVertexAttribute(0, color, center, size);
        vertes[vindex + 1] = GetVertexAttribute(1, color, center, size);
        vertes[vindex + 2] = GetVertexAttribute(2, color, center, size);

        uint pindex = 1 * tid;
        tris[pindex + 0] = uint3(vindex + 0, vindex + 1, vindex + 2);
    }
}

#endif // MESH

#ifdef INDIRECT

VertexOut DrawVs(uint vertex_id: SV_VertexID, uint instance_id: SV_InstanceID, uint color_index: COLOR_INDEX)
{
    float3 color = ColorIdToRgb(color_index);
    float size = CalcSize(CountBuf[color_index]);

    float3 center;
    switch (ColorSpace) {
    case 0: center = RgbToPosition(color); break;
    case 1: center = HsvToPosition(color); break;
    case 2: center = HslToPosition(color); break;
    case 3: center = YuvToPosition(color); break;
    }
    center = mul(Projection, float4(center, 1.f)).xyz;

    return GetVertexAttribute(vertex_id, color, center, size);
}

#endif // INDIRECT

struct PsInput {
    float4 position : SV_Position;
    float4 color : COLOR;
    float2 uv : TEXCOORD;
};

float4 DrawPs(PsInput input) : SV_Target
{
    clip(1.f - dot(input.uv, input.uv));
    return input.color;
}

#endif // DRAW
