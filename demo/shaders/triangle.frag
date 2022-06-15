#version 450

layout(location = 0) in vec2 v_Uv;

layout(binding = 0) uniform sampler2D texSampler;

layout(location = 0) out vec4 o_Color;

void main() {
  o_Color = texture(texSampler, v_Uv);
}