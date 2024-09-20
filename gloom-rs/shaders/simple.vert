#version 410 core

layout(location=0) in vec3 position;
layout(location=1) in vec4 color;

uniform float oscVal;
uniform mat4 matrix;

out vec4 vertColor;

void main()
{
    vec4 pos = vec4(position, 1.0);

    // When defining our matrix like this
    vec4 transformed_position = matrix * pos;


    gl_Position = transformed_position;
    vertColor = color;
}
