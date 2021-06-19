#version 410 core

layout(location = 0) out vec4 color;

uniform vec3 object_color;
uniform vec3 light_color;
uniform vec3 light_pos;

in vec3 fs_normal;
in vec3 frag_pos;

void main()
{
    vec3 light_direction = normalize(light_pos - frag_pos);

    float ambient_strength = 0.2;
    vec3 ambient = ambient_strength * light_color;

    vec3 norm = normalize(fs_normal);

    float diff = max(dot(norm, light_direction), 0.0);

    vec3 diffuse = diff * light_color;

    vec3 result = (ambient + diffuse) * object_color;

    color = vec4(result, 1.0);
}
