#version 460

layout(push_constant) uniform struct PushConst {
    vec2 resolution;
    float time;
} push;

in vec2 position;
out vec4 surface_color;

//
// Description : Array and textureless GLSL 2D/3D/4D simplex
//							 noise functions.
//			Author : Ian McEwan, Ashima Arts.
//	Maintainer : ijm
//		 Lastmod : 20110822 (ijm)
//		 License : Copyright (C) 2011 Ashima Arts. All rights reserved.
//							 Distributed under the MIT License. See LICENSE file.
//							 https://github.com/ashima/webgl-noise
//

vec3 mod289(vec3 x) {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

vec4 mod289(vec4 x) {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

vec4 permute(vec4 x) {
    return mod289(((x * 34.0) + 1.0) * x);
}

vec4 taylorInvSqrt(vec4 r)
{
    return 1.79284291400159 - 0.85373472095314 * r;
}

float snoise(vec3 v)
{
    const vec2 C = vec2(1.0 / 6.0, 1.0 / 3.0);
    const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);

    // First corner
    vec3 i = floor(v + dot(v, C.yyy));
    vec3 x0 = v - i + dot(i, C.xxx);

    // Other corners
    vec3 g = step(x0.yzx, x0.xyz);
    vec3 l = 1.0 - g;
    vec3 i1 = min(g.xyz, l.zxy);
    vec3 i2 = max(g.xyz, l.zxy);

    //	 x0 = x0 - 0.0 + 0.0 * C.xxx;
    //	 x1 = x0 - i1	+ 1.0 * C.xxx;
    //	 x2 = x0 - i2	+ 2.0 * C.xxx;
    //	 x3 = x0 - 1.0 + 3.0 * C.xxx;
    vec3 x1 = x0 - i1 + C.xxx;
    vec3 x2 = x0 - i2 + C.yyy; // 2.0*C.x = 1/3 = C.y
    vec3 x3 = x0 - D.yyy; // -1.0+3.0*C.x = -0.5 = -D.y

    // Permutations
    i = mod289(i);
    vec4 p = permute(permute(permute(
                    i.z + vec4(0.0, i1.z, i2.z, 1.0))
                    + i.y + vec4(0.0, i1.y, i2.y, 1.0))
                + i.x + vec4(0.0, i1.x, i2.x, 1.0));

    // Gradients: 7x7 points over a square, mapped onto an octahedron.
    // The ring size 17*17 = 289 is close to a multiple of 49 (49*6 = 294)
    float n_ = 0.142857142857; // 1.0/7.0
    vec3 ns = n_ * D.wyz - D.xzx;

    vec4 j = p - 49.0 * floor(p * ns.z * ns.z); //	mod(p,7*7)

    vec4 x_ = floor(j * ns.z);
    vec4 y_ = floor(j - 7.0 * x_); // mod(j,N)

    vec4 x = x_ * ns.x + ns.yyyy;
    vec4 y = y_ * ns.x + ns.yyyy;
    vec4 h = 1.0 - abs(x) - abs(y);

    vec4 b0 = vec4(x.xy, y.xy);
    vec4 b1 = vec4(x.zw, y.zw);

    //vec4 s0 = vec4(lessThan(b0,0.0))*2.0 - 1.0;
    //vec4 s1 = vec4(lessThan(b1,0.0))*2.0 - 1.0;
    vec4 s0 = floor(b0) * 2.0 + 1.0;
    vec4 s1 = floor(b1) * 2.0 + 1.0;
    vec4 sh = -step(h, vec4(0.0));

    vec4 a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    vec4 a1 = b1.xzyw + s1.xzyw * sh.zzww;

    vec3 p0 = vec3(a0.xy, h.x);
    vec3 p1 = vec3(a0.zw, h.y);
    vec3 p2 = vec3(a1.xy, h.z);
    vec3 p3 = vec3(a1.zw, h.w);

    //Normalise gradients
    //vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
    vec4 norm = inversesqrt(vec4(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;

    // Mix final noise value
    vec4 m = max(0.6 - vec4(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), 0.0);
    m = m * m;
    return 42.0 * dot(m * m, vec4(dot(p0, x0), dot(p1, x1),
                dot(p2, x2), dot(p3, x3)));
}

//////////////////////////////////////////////////////////////

// PRNG
// From https://www.shadertoy.com/view/4djSRW
float prng(in vec2 seed) {
    seed = fract(seed * vec2(5.3983, 5.4427));
    seed += dot(seed.yx, seed.xy + vec2(21.5351, 14.3137));
    return fract(seed.x * seed.y * 95.4337);
}

//////////////////////////////////////////////////////////////

float PI = 3.1415926535897932384626433832795;

float noiseStack(vec3 pos, int octaves, float falloff) {
    float noise = snoise(vec3(pos));
    float off = 1.0;
    if (octaves > 1) {
        pos *= 2.0;
        off *= falloff;
        noise = (1.0 - off) * noise + off * snoise(vec3(pos));
    }
    if (octaves > 2) {
        pos *= 2.0;
        off *= falloff;
        noise = (1.0 - off) * noise + off * snoise(vec3(pos));
    }
    if (octaves > 3) {
        pos *= 2.0;
        off *= falloff;
        noise = (1.0 - off) * noise + off * snoise(vec3(pos));
    }
    return (1.0 + noise) / 2.0;
}

vec2 noiseStackUV(vec3 pos, int octaves, float falloff, float diff) {
    float displaceA = noiseStack(pos, octaves, falloff);
    float displaceB = noiseStack(pos + vec3(3984.293, 423.21, 5235.19), octaves, falloff);
    return vec2(displaceA, displaceB);
}

void main() {
    vec2 fragCoord = (0.5 * position + 0.5) * push.resolution;
    float time = push.time;
    vec2 resolution = push.resolution.xy;
    vec2 drag = vec2(0.0);
    vec2 offset = vec2(0.0);
    //
    float xpart = fragCoord.x / resolution.x;
    float ypart = fragCoord.y / resolution.y;
    //
    float clip = 210.0;
    float ypartClip = fragCoord.y / clip;
    float ypartClippedFalloff = clamp(2.0 - ypartClip, 0.0, 1.0);
    float ypartClipped = min(ypartClip, 1.0);
    float ypartClippedn = 1.0 - ypartClipped;
    //
    float xfuel = 1.0 - abs(2.0 * xpart - 1.0); //pow(1.0-abs(2.0*xpart-1.0),0.5);
    //
    float timeSpeed = 0.5;
    float realTime = timeSpeed * time;
    //
    vec2 coordScaled = 0.01 * fragCoord - 0.02 * vec2(offset.x, 0.0);
    vec3 position = vec3(coordScaled, 0.0) + vec3(1223.0, 6434.0, 8425.0);
    vec3 flow = vec3(4.1 * (0.5 - xpart) * pow(ypartClippedn, 4.0), -2.0 * xfuel * pow(ypartClippedn, 64.0), 0.0);
    vec3 timing = realTime * vec3(0.0, -1.7, 1.1) + flow;
    //
    vec3 displacePos = vec3(1.0, 0.5, 1.0) * 2.4 * position + realTime * vec3(0.01, -0.7, 1.3);
    vec3 displace3 = vec3(noiseStackUV(displacePos, 2, 0.4, 0.1), 0.0);
    //
    vec3 noiseCoord = (vec3(2.0, 1.0, 1.0) * position + timing + 0.4 * displace3) / 1.0;
    float noise = noiseStack(noiseCoord, 3, 0.4);
    //
    float flames = pow(ypartClipped, 0.3 * xfuel) * pow(noise, 0.3 * xfuel);
    //
    float f = ypartClippedFalloff * pow(1.0 - flames * flames * flames, 8.0);
    float fff = f * f * f;
    vec3 fire = 1.5 * vec3(f, fff, fff * fff);
    //
    // smoke
    float smokeNoise = 0.5 + snoise(0.4 * position + timing * vec3(1.0, 1.0, 0.2)) / 2.0;
    vec3 smoke = vec3(0.3 * pow(xfuel, 3.0) * pow(ypart, 2.0) * (smokeNoise + 0.4 * (1.0 - noise)));
    //
    // sparks
    float sparkGridSize = 30.0;
    vec2 sparkCoord = fragCoord - vec2(2.0 * offset.x, 190.0 * realTime);
    sparkCoord -= 30.0 * noiseStackUV(0.01 * vec3(sparkCoord, 30.0 * time), 1, 0.4, 0.1);
    sparkCoord += 100.0 * flow.xy;
    if (mod(sparkCoord.y / sparkGridSize, 2.0) < 1.0) sparkCoord.x += 0.5 * sparkGridSize;
    vec2 sparkGridIndex = vec2(floor(sparkCoord / sparkGridSize));
    float sparkRandom = prng(sparkGridIndex);
    float sparkLife = min(10.0 * (1.0 - min((sparkGridIndex.y + (190.0 * realTime / sparkGridSize)) / (24.0 - 20.0 * sparkRandom), 1.0)), 1.0);
    vec3 sparks = vec3(0.0);
    if (sparkLife > 0.0) {
        float sparkSize = xfuel * xfuel * sparkRandom * 0.08;
        float sparkRadians = 999.0 * sparkRandom * 2.0 * PI + 2.0 * time;
        vec2 sparkCircular = vec2(sin(sparkRadians), cos(sparkRadians));
        vec2 sparkOffset = (0.5 - sparkSize) * sparkGridSize * sparkCircular;
        vec2 sparkModulus = mod(sparkCoord + sparkOffset, sparkGridSize) - 0.5 * vec2(sparkGridSize);
        float sparkLength = length(sparkModulus);
        float sparksGray = max(0.0, 1.0 - sparkLength / (sparkSize * sparkGridSize));
        sparks = sparkLife * sparksGray * vec3(1.0, 0.3, 0.0);
    }
    //
    surface_color = vec4(max(fire, sparks) + smoke, 1.0);
}
