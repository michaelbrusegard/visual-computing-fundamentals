#version 410 core

#define PI 3.1415926535897932384626433832795

uniform float time;

out vec4 color;
in vec4 vertColor;

vec4 pastel_night_sky = vec4(0.305, 0.454, 0.494, 1.0);
vec4 moon_white = vec4(0.94, 0.917, 0.66, 1.0);

vec4 color1 = vec4(1.0, 1.0, 1.0, 1.0);
vec4 color2 = vec4(0.737, 0.0, 0.176, 1.0);

vec4 night_sky = vec4(0.218, 0.156, 0.312, 1.0);
vec4 waves = vec4(66.0, 52.0, 109.0, 255.0)/255.0;

vec4 checkerbox(int x, int y, int box_size)
{
    // Check if x + y is even
    bool isEven = bool(((x / box_size) + (y / box_size)) % 2);
    return (isEven ? pastel_night_sky : moon_white);
}

// Signed distance circle
// https://iquilezles.org/articles/distfunctions2d/
// vec2 p - position.
// float r - radius / size of circle.
float sdCircle(vec2 p, float r)
{
    return length(p) - r;
}

// Spiral
// vec2 p - position.
// float a - amount of spiral lines "arms".
// float k - spiral intensity.
float spiral(vec2 p, float a, float k)
{
    float theta = atan(p.y, p.x);
    float r = length(p);
    return fract(a * theta / 3.14 + k * r - 2.5 * time);
}

// https://iquilezles.org/articles/distfunctions2d/ signed distance Triangle from Inigo Quilez.
// Also credits to Stand-Up Maths https://www.youtube.com/watch?v=A0L_PZwuGHY&t=1194s
float sdTriangle(vec2 p1, vec2 p2, vec2 p3, vec2 p)
{
    // Quilez defines a triangle function such as in the timestamped video from Stand-Up maths.
    vec2 e0 = p2 - p1;
    vec2 e1 = p3 - p2;
    vec2 e2 = p1 - p3;

    vec2 f0 = p - p1;
    vec2 f1 = p - p2;
    vec2 f2 = p - p3;

    vec2 pq0 = f0 - e0 * clamp( dot(f0, e0)/dot(e0, e0), 0.0, 1.0);
    vec2 pq1 = f1 - e1 * clamp( dot(f1, e1)/dot(e1, e1), 0.0, 1.0);
    vec2 pq2 = f2 - e2 * clamp( dot(f2, e2)/dot(e2, e2), 0.0, 1.0);

    float s = sign( e0.x*e2.y - e0.y*e2.x);
    vec2 d = min(min(vec2(dot(pq0, pq0), s*(f0.x*e0.y - f0.y*e0.x)),
                     vec2(dot(pq1, pq1), s*(f1.x*e1.y - f1.y*e1.x))),
                     vec2(dot(pq2, pq2), s*(f2.x*e2.y - f2.y*e2.x)));

    return -sqrt(d.x)*sign(d.y);
}

void main()
{
    // Position for spiral, circle, and triangle.
    vec2 p = vec2(2.0 * gl_FragCoord.xy) / 750.0;

    // Spiral
    // float theta = atan(p.y, p.x);
    // color = spiral(p, 5.0, 12.0) < 0.6 ? pastel_night_sky : moon_white;

    // Burgundy-red
    // color = vec4(0.5, 0.0, 0.13, 1.0);

    //Checker-pattern
    //int x = int(gl_FragCoord.x);
    //int y = int(gl_FragCoord.y);
    //color = checkerbox(x, y, 30);

    // Drawn Circle
    // float r = 0.3;
    // color = sdCircle(p, r);

    // Color over time
    // color = vec4(cos(time+24), sin(time * 0.2), atan(time+24, time), 1.0);

    // Manual triangle
    // vec2 pos1 = 2.0 * vec2(120.0, 200.0) / 750.0;
    // vec2 pos2 = 2.0 * vec2(630.0, 200.0) / 750.0;
    // vec2 pos3 = 2.0 * vec2(375.0, 550.0) / 750.0;
    // float d = sdTriangle(pos1, pos2, pos3, p);

    // color = mix(night_sky, moon_white, cos(160.0 * d));
    // color *= exp(-2.3 * abs(d));

    // color = d > 0.0 ? color : mix(color1, vec4(0.5, 0.0, 0.13, 1.0), 1.0 - smoothstep(0.01, 0.3, abs(d)));

    // if (d >= -0.005 && d <= 0.005)
    // {
    //     color = color1;
    // }

    // A sine wave
    float y = 0.05*sin(10*p.x - PI*time/2) + 0.7;
    float white_wave = 0.01*sin(30*p.x - PI*time/1.5) + 0.7;
    float blue_wave = 0.02*cos(12*p.x + PI*time) + 0.63;
    float light_wave = 0.03*cos(7*p.x - PI*time/1.8) + 0.58;

    if (y > p.y)
    {
        color = waves;
        if (p.y > light_wave - 0.06)
        {
            color = vec4( 84, 136, 200, 255) / 255;
        }
        if (p.y > blue_wave)
        {
            color = vec4( 143, 182, 228, 255) / 255;
        }
        if (p.y > white_wave)
        {
            color = color1;
        }
    }
    else
    {
        color = night_sky;
    }

    vec2 moon_p = vec2(p.x - 1.0, p.y - 1.2);
    float sd = sdCircle(moon_p, 0.15);
    vec2 moon_rp = vec2(p.x - 0.92, p.y - 1.24);
    float sdrp = sdCircle(moon_rp, 0.09);

    if (sd < 0.0 && sdrp > 0.0)
    {
        color = vec4( 243, 249, 169, 255) / 255;
    }

    // Default color
    color = vertColor;
}
