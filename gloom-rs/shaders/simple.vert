#version 410 core

in vec3 position;

void main()
{   
    // vec3 flipped_position = vec3(position.x * -1, position.y * -1, position.z);
    gl_Position = vec4(position, 1.0f);
}
