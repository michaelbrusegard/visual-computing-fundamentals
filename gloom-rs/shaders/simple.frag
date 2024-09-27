#version 410 core

// Definitions
#define PI 3.1415926535897932384626433832795

// Internals
uniform float time;
in vec4 fColor;
in vec3 fNormal;
out vec4 color;

// Globals
vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

void main()
{
    // Default color
    vec3 colorRGB = vec3(fColor.x, fColor.y, fColor.z);
    float normLightDot = dot(fNormal, -lightDirection);

    color = vec4(colorRGB * max(0.0, normLightDot), fColor.w);
    // color = vec4(fNormal, 1.0);
}
