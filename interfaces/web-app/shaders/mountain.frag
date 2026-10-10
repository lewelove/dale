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
uniform float iGrain;

out vec4 fragColor;

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

    return clamp(vec3(r, g, b), 0.1, 0.8);
}

vec4 permute(vec4 x){return mod(((x*34.0)+1.0)*x, 289.0);}
vec4 taylorInvSqrt(vec4 r){return 1.79284291400159 - 0.85373472095314 * r;}

float snoise(vec3 v){ 
  const vec2  C = vec2(1.0/6.0, 1.0/3.0) ;
  const vec4  D = vec4(0.0, 0.5, 1.0, 2.0);

  vec3 i  = floor(v + dot(v, C.yyy) );
  vec3 x0 =   v - i + dot(i, C.xxx) ;

  vec3 g = step(x0.yzx, x0.xyz);
  vec3 l = 1.0 - g;
  vec3 i1 = min( g.xyz, l.zxy );
  vec3 i2 = max( g.xyz, l.zxy );

  vec3 x1 = x0 - i1 + 1.0 * C.xxx;
  vec3 x2 = x0 - i2 + 2.0 * C.xxx;
  vec3 x3 = x0 - D.yyy;

  i = mod(i, 289.0); 
  vec4 p = permute( permute( permute( 
             i.z + vec4(0.0, i1.z, i2.z, 1.0 ))
           + i.y + vec4(0.0, i1.y, i2.y, 1.0 )) 
           + i.x + vec4(0.0, i1.x, i2.x, 1.0 ));

  float n_ = 1.0/7.0;
  vec3  ns = n_ * D.wyz - D.xzx;

  vec4 j = p - 49.0 * floor(p * ns.z *ns.z);

  vec4 x_ = floor(j * ns.z);
  vec4 y_ = floor(j - 7.0 * x_ );

  vec4 x = x_ *ns.x + ns.yyyy;
  vec4 y = y_ *ns.x + ns.yyyy;
  vec4 h = 1.0 - abs(x) - abs(y);

  vec4 b0 = vec4( x.xy, y.xy );
  vec4 b1 = vec4( x.zw, y.zw );

  vec4 s0 = floor(b0)*2.0 + 1.0;
  vec4 s1 = floor(b1)*2.0 + 1.0;
  vec4 sh = -step(h, vec4(0.0));

  vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy ;
  vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww ;

  vec3 p0 = vec3(a0.xy,h.x);
  vec3 p1 = vec3(a0.zw,h.y);
  vec3 p2 = vec3(a1.xy,h.z);
  vec3 p3 = vec3(a1.zw,h.w);

  vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
  p0 *= norm.x;
  p1 *= norm.y;
  p2 *= norm.z;
  p3 *= norm.w;

  vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
  m = m * m;
  return 42.0 * dot( m*m, vec4( dot(p0,x0), dot(p1,x1), 
                                dot(p2,x2), dot(p3,x3) ) );
}

float fbm(vec3 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 3; i++) {
        v += a * snoise(p);
        p *= 2.0;
        a *= 0.5;
    }
    
    v = v / 0.875;
    v = v * 0.5 + 0.5;
    v = 0.5 - 0.5 * cos(3.14159265 * v);
    v = 0.5 - 0.5 * cos(3.14159265 * v);
    
    return clamp(v, 0.0, 1.0);
}

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution.xy;
    float aspect = iResolution.x / iResolution.y;
    vec2 p = (uv - 0.5);
    p.x *= aspect;
    
    float t = (iTime + iRandom) * iSpeed;

    float val = fbm(vec3(p * iZoom, t));

    float totalWeight = 0.0;
    for(int i = 0; i < NUM_COLORS; i++) {
        totalWeight += iRatios[i];
    }
    if (totalWeight <= 0.0) totalWeight = 1.0; 

    vec3 finalOklab = vec3(0.0);
    float cumulative = 0.00;
    float currentEdgeSoftness = 0.0;

    for(int i = 0; i < NUM_COLORS; i++) {
        float weight = iRatios[i] / totalWeight;
        float nextCumulative = cumulative + weight;
        
        float nextWeight = 0.0;
        if (i + 1 < NUM_COLORS) {
            nextWeight = iRatios[i+1] / totalWeight;
        }
        
        float nextEdgeSoftness = iBlur * min(weight, nextWeight); 
        
        float startMask = (i == 0) ? 1.0 : smoothstep(cumulative - currentEdgeSoftness, cumulative + currentEdgeSoftness, val);
        float endMask = (i == NUM_COLORS - 1) ? 0.0 : smoothstep(nextCumulative - nextEdgeSoftness, nextCumulative + nextEdgeSoftness, val);
        
        float weightMask = startMask - endMask;
        
        finalOklab += iColorsOklab[i] * max(0.0, weightMask);
        
        cumulative = nextCumulative;
        currentEdgeSoftness = nextEdgeSoftness;
    }
    
    finalOklab.x = 0.0 + (finalOklab.x * 0.7);
    
    vec3 finalColor = oklab_to_srgb(finalOklab);

    float grain = (fract(sin(dot(uv, vec2(12.9898, 78.233))) * 43758.5453) - 0.5) * iGrain;
    finalColor += grain;
    
    fragColor = vec4(finalColor, 1.0);
}
