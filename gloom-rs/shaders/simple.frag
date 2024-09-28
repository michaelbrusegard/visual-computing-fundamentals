#version 410 core

// Definitions
#define PI 3.1415926535897932384626433832795

// Internals
uniform float time;
uniform vec3 cameraPosition; // for phon lighting
in vec3 fPosition; // for phong lighting
in vec4 fColor;
in vec3 fNormal;
out vec4 color;

// Globals
vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
vec3 lightColor = vec3(1.0, 0.905, 0.443);

void main()
{
    // Default color
    vec3 colorRGB = vec3(fColor.x, fColor.y, fColor.z);
    float normLightDot = dot(fNormal, -lightDirection);

    vec3 LightV = -lightDirection;
    vec3 ViewV = normalize(cameraPosition - fPosition);
    vec3 ReflectionV = reflect(-LightV, fNormal);

    // Ambient
    vec3 ambient = 0.5 * lightColor; //arbitrary ambient strength

    // Diffuse
    vec3 diffuse = 0.8 * max(dot(fNormal, LightV), 0.0) * vec3(1.0, 1.0, 1.0); //arbitrary diffuse strength

    // Specular
    vec3 specular = 0.8 * pow(max(dot(ViewV, ReflectionV), 0.0), 128.0) * vec3(1.0, 1.0, 1.0); //arbitrary shininess and specular strength

    vec3 phong = ambient + diffuse + specular;
    
    // Phong
    color = fColor * vec4(phong, 1.0);

    // Lambertian
    // color = vec4(colorRGB * max(0.0, normLightDot), fColor.w);

    // Normals
    // color = vec4(fNormal, 1.0);
}
