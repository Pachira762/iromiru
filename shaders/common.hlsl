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

float3 HsvToRgb(float hue, float saturation, float luminance)
{
    float r = luminance;
    float g = luminance;
    float b = luminance;

    float h = frac(hue + 1.f);
    uint i = (uint)((359.9999f * h) / 60.f);
    float f = 6.f * h - (float)i;
    float s = saturation;

    switch (i) {
    case 0:
        g *= 1.f - s * (1.f - f);
        b *= 1.f - s;
        break;
    case 1:
        r *= 1.f - s * f;
        b *= 1.f - s;
        break;
    case 2:
        r *= 1.f - s;
        b *= 1.f - s * (1.f - f);
        break;
    case 3:
        r *= 1.f - s;
        g *= 1.f - s * f;
        break;
    case 4:
        r *= 1.f - s * (1.f - f);
        g *= 1.f - s;
        break;
    case 5:
        g *= 1.f - s;
        b *= 1.f - s * f;
        break;
    }

    return float3(r, g, b);
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