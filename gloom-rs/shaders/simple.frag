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
    // Default color
    color = vertColor;
}
