#version 300 es
precision highp float;
precision highp int;

#ifndef NUM_COLORS
#define NUM_COLORS 24
#endif

uniform float iTime;
uniform float iRandom;
uniform vec2 iResolution;
uniform float iCoverSize;
uniform vec3 iColorsOklab[NUM_COLORS];
uniform float iRatios[NUM_COLORS];
uniform int iCount;

uniform float iSpeed;
uniform float iZoom;
uniform float iBlur;
uniform float iEdgeBlur;
uniform float iGrain;

out vec4 fragColor;

const float SPEED = 0.02;
const float GRAIN_AMOUNT = 0.02;
const float N = 24.0; 

vec3 oklab_to_srgb(vec3 c) {
    float l_ = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
    float m_ = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
    float s_ = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;

    float l = l_ * l_ * l_;
    float m = m_ * m_ * m_;
    float s = s_ * s_ * s_;

    float r = +4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    float g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    float b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    r = r >= 0.0031308 ? 1.055 * pow(r, 1.0 / 2.4) - 0.055 : 12.92 * r;
    g = g >= 0.0031308 ? 1.055 * pow(g, 1.0 / 2.4) - 0.055 : 12.92 * g;
    b = b >= 0.0031308 ? 1.055 * pow(b, 1.0 / 2.4) - 0.055 : 12.92 * b;

    return clamp(vec3(r, g, b), 0.0, 1.0);
}

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

float fbm(vec2 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 4; i++) {
        v += a * noise(p);
        p *= 2.0;
        a *= 0.5;
    }
    return v;
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float aspect = iResolution.x / iResolution.y;
    vec2 p = uv;
    p.x *= aspect;
    
    float t = (iTime + iRandom) * SPEED;
    
    vec2 center = vec2(0.5 * aspect, 0.5);
    float dist = length(p - center);
    float cRad = (iCoverSize / iResolution.y) * 0.5;
    
    p += (p - center) * smoothstep(cRad + 0.5, cRad - 0.1, dist) * 0.3;

    float val = fbm(p * 1.5 + t);
    val = clamp(val, 0.0, 1.0);

    float totalWeight = 0.0;
    for(int i = 0; i < NUM_COLORS; i++) {
        if (i >= iCount) break;
        totalWeight += 1.0 / (float(i) + N);
    }

    vec3 finalColor = vec3(0.0);
    float softness = 0.125; 
    float cumulative = 0.0;

    for(int i = 0; i < NUM_COLORS; i++) {
        if (i >= iCount) break;
        
        float weight = (1.0 / (float(i) + N)) / totalWeight;
        float nextCumulative = cumulative + weight;
        
        float weightMask = smoothstep(cumulative - softness, cumulative + softness, val) - 
                           smoothstep(nextCumulative - softness, nextCumulative + softness, val);
        
        finalColor += oklab_to_srgb(iColorsOklab[i]) * max(0.0, weightMask);
        cumulative = nextCumulative;
    }

    float noiseFloor = (hash(uv + iTime) - 0.5) * GRAIN_AMOUNT;
    finalColor += noiseFloor;
    
    fragColor = vec4(finalColor, 1.0);
}
