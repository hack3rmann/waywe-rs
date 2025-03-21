#version 460

layout(location = 0) in vec2 vertex_pos;

out vec3 vertex_color;

void main() {
    gl_Position = vec4(vertex_pos, 0.0, 1.0);

    vec3 colors[3] =  vec3[](
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0)
    );

    vertex_color = colors[gl_VertexIndex];
}
