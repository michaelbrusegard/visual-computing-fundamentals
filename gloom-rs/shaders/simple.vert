#version 410 core

layout(location=0) in vec3 position;
layout(location=1) in vec4 color;

out vec4 vertColor;

void main()
{
    // vec3 flipped_position = vec3(position.x * -1, position.y * -1, position.z);
    gl_Position = vec4(position, 1.0f);
    vertColor = color;
}
