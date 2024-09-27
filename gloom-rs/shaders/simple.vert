#version 410 core

layout(location=0) in vec3 position;
layout(location=1) in vec3 normal;
layout(location=2) in vec4 color;

uniform float oscVal;
uniform mat4 projectionMatrix;
uniform mat4 transformMatrix;

out vec4 fColor;
out vec3 fNormal;

void main()
{
    vec4 pos = vec4(position, 1.0);

    vec4 transformed_position = projectionMatrix * transformMatrix * pos;
    mat3 normalMatrix = mat3(transformMatrix);

    gl_Position = transformed_position;
    fColor = color;
    fNormal = normalize(normalMatrix * normal);

}
