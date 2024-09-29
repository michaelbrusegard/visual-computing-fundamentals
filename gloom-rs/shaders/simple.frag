#version 410 core

out vec4 color;
uniform vec2 resolution;

vec4 checkerbox(int x, int y, int box_size)
{
    bool isEven = bool(((x / box_size) + (y / box_size)) % 2);
    return (isEven ? vec4(1.0f, 0.0f, 0.0f, 1.0f) : vec4(1.0f, 1.0f, 1.0f, 1.0f));
}

float circle(vec2 point, float radius)
{
    return length(point) - radius;
}

void main()
{
    int x = int(gl_FragCoord.x);
    int y = int(gl_FragCoord.y);
    vec2 point = (2.0 * gl_FragCoord.xy - resolution.xy) / resolution.y;
    float d = circle(point, 1.4);
    if (d < 0.0) {
        color = checkerbox(x, y, 30);
    } else {
        color = vec4(0.0f, 1.0f, 0.0f, 1.0f);
    }
}
