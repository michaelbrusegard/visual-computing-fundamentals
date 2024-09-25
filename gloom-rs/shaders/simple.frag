#version 410 core

#define PI 3.1415926535897932384626433832795

uniform float time;

out vec4 color;
in vec4 fColor;
in vec3 fNormal;

void main()
{
    // Default color
    color = vec4(fNormal, 1.0);
}
