varying vec2 v_vt;
varying vec3 v_side_distances;
varying vec3 v_corner_distances;
varying vec2 v_camera_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec3 a_side_distances;
attribute vec3 a_corner_distances;
attribute vec2 a_flow;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float u_simulation_time;
uniform vec2 u_texture_shift;
uniform vec2 u_texture_scale;

void main() {
    v_side_distances = a_side_distances;
    v_corner_distances = a_corner_distances;
    v_vt = a_pos / u_texture_scale - a_flow * u_simulation_time + u_texture_shift;
    v_camera_pos = (u_view_matrix * vec3(a_pos, 1.0)).xy;
    gl_Position = vec4((u_projection_matrix * vec3(v_camera_pos, 1.0)).xy, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform float u_reveal_radius;

float d2(vec3 v) {
    v = max(v, 0.0);
    return dot(v, v);
}

void main() {
    gl_FragColor = texture2D(u_texture, v_vt);
    gl_FragColor.a *= smoothstep(u_reveal_radius - 1.0, u_reveal_radius, length(v_camera_pos));
    float approx_distance_to_edge = sqrt(d2(v_side_distances) + d2(v_corner_distances));
    gl_FragColor.a *= 1.0 - min(approx_distance_to_edge, 1.0);
    
}
#endif