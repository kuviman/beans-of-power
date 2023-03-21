varying vec2 v_vt;
varying vec2 v_camera_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_flow;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float u_simulation_time;
uniform vec2 u_texture_shift;
uniform vec2 u_texture_scale;

void main() {
    v_vt = a_pos / u_texture_scale - a_flow * u_simulation_time + u_texture_shift;
    v_camera_pos = (u_view_matrix * vec3(a_pos, 1.0)).xy;
    gl_Position = vec4((u_projection_matrix * vec3(v_camera_pos, 1.0)).xy, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform float u_reveal_radius;

void main() {
    gl_FragColor = texture2D(u_texture, v_vt);
    gl_FragColor.a *= smoothstep(u_reveal_radius - 1.0, u_reveal_radius, length(v_camera_pos));
}
#endif