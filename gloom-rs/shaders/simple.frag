#version 410 core

out vec4 color;

vec4 pastel_night_sky = vec4(0.305, 0.454, 0.494, 1.0);
vec4 moon_white = vec4(0.94, 0.917, 0.66, 1.0);


vec4 checkerbox(int x, int y, int box_size)
{
    // Check if x + y is even
    bool isEven = bool(((x / box_size) + (y / box_size)) % 2);
    return (isEven ? pastel_night_sky : moon_white);
}

// Signed distance circle
// https://iquilezles.org/articles/distfunctions2d/
float sdCircle(vec2 p, float r)
{
    return length(p) - r;
}

void main()
{
    vec2 p = vec2((2.0 * gl_FragCoord.x - 750.0) / 750.0, (2.0 * gl_FragCoord.y - 750.0) / 750.0);
    float r = 0.5;

    float d = sdCircle(p, r);
    color = d > 0.0 ? pastel_night_sky : moon_white;

    // Default white color
    // color = vec4(1.0, 1.0, 1.0, 1.0);
}