static const float Pi = 3.14159265358979323846;

Texture2D Tex : register(t0);

float Max3(float a, float b, float c) 
{
    return max(a, max(b, c));
}

float Min3(float a, float b, float c) 
{
    return min(a, min(b, c));
}

float3 ToRgb(float3 rgb) 
{
    return rgb;
}

float3 ToHsv(float3 rgb)
{
    float ma = Max3(rgb.r, rgb.g, rgb.b);
    float mi = Min3(rgb.r, rgb.g, rgb.b);

    float H = 0.f;
    if (mi == ma) {
        H = 0.f;
    }
    else if (mi == rgb.b) {
        H = ((rgb.g - rgb.r) / (ma - mi) + 1.f) / 6.f;
    }
    else if (mi == rgb.r) {
        H = ((rgb.b - rgb.g) / (ma - mi) + 3.f) / 6.f;
    }
    else {
        H = ((rgb.r - rgb.b) / (ma - mi) + 5.f) / 6.f;
    }

    float S = ma - mi;

    float V = ma;

    return float3(H, S, V);
}

float3 ToHsl(float3 rgb) 
{
    float ma = Max3(rgb.r, rgb.g, rgb.b);
    float mi = Min3(rgb.r, rgb.g, rgb.b);

    float H = 0.f;
    if (mi == ma) {
        H = 0.f;
    }
    else if (mi == rgb.b) {
        H = ((rgb.g - rgb.r) / (ma - mi) + 1.f) / 6.f;
    }
    else if (mi == rgb.r) {
        H = ((rgb.b - rgb.g) / (ma - mi) + 3.f) / 6.f;
    }
    else {
        H = ((rgb.r - rgb.b) / (ma - mi) + 5.f) / 6.f;
    }

    float S = ma - mi;

    float L = (ma + mi) / 2.f;

    return float3(H, S, L);
}

float3 ToYuv(float3 rgb) 
{
    static const float3x3 RgbToYuv = {
        +0.212600, +0.715200, +0.072200,
        -0.114572, -0.385428, +0.500000,
        +0.500000, -0.451453, -0.045847
    };

    return mul(RgbToYuv, rgb) + float3(0.f, 0.5f, 0.5f);
}

float ToLuma(float3 rgb)
{
    static const float3 Luma = {
        0.299f, 0.587f, 0.114f
    };

    return dot(rgb, Luma);
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

float3 HslToRgb(float hue, float saturation, float luminance)
{
    float h = 360.f * frac(hue + 1.f);
    float ma = luminance + 0.5f * saturation;
    float mi = luminance - 0.5f * saturation;
    float mm = ma - mi;

    if (h < 60.f) {
        return float3(ma, mi + mm * h / 60.f, mi);
    } else if (h < 120.f) {
        return float3(mi + mm * (120.f - h) / 60.f, ma, mi);
    } else if (h < 180.f) {
        return float3(mi, ma, mi + mm * (h - 120.f) / 60.f);
    } else if (h < 240.f) {
        return float3(mi, mi + mm * (240.f - h)/60.f, ma);
    } else if (h < 300.f) {
        return float3(mi + mm * (h-240.f) / 60.f, mi, ma);
    } else {
        return float3(ma, mi, mi + mm*(360.f - h) / 60.f);
    }
}