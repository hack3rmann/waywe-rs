
varying vec4 v_TexCoord;
varying vec2 v_Direction;

uniform sampler2D g_Texture0; // {"material":"framebuffer","label":"ui_editor_properties_framebuffer","hidden":true}
uniform sampler2D g_Texture1; // {"material":"mask","label":"ui_editor_properties_opacity_mask","mode":"opacitymask","default":"util/white","paintdefaultcolor":"0 0 0 1"}
uniform float g_Time;

uniform float g_Speed; // {"material":"speed","label":"ui_editor_properties_speed","default":5,"range":[0.01,50]}
uniform float g_Scale; // {"material":"scale","label":"ui_editor_properties_scale","default":200,"range":[0.01,1000]}
uniform float g_Strength; // {"material":"strength","label":"ui_editor_properties_strength","default":0.1,"range":[0.01,1]}
uniform float g_Perspective; // {"material":"perspective","label":"ui_editor_properties_perspective","default":0,"range":[0,0.2]}

void main() {
	float mask = texSample2D(g_Texture1, v_TexCoord.zw).r;
	vec2 texCoord = v_TexCoord.xy;
	
	float pos = abs(dot((texCoord - 0.5), v_Direction));
	
	float distance = g_Time * g_Speed + dot(texCoord, v_Direction) * (g_Scale + g_Perspective * pos);
	vec2 offset = vec2(v_Direction.y, -v_Direction.x);
	float strength = g_Strength * g_Strength + g_Perspective * pos;
	texCoord += sin(distance) * offset * strength * mask;
	
	gl_FragColor = texSample2D(g_Texture0, texCoord);
}
