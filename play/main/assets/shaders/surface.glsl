//	Classic Perlin 2D Noise 
//	by Stefan Gustavson
//
vec4 permute(vec4 x){return mod(((x*34.0)+1.0)*x, 289.0);}
vec2 fade(vec2 t) {return t*t*t*(t*(t*6.0-15.0)+10.0);}

float cnoise(vec2 P){
  vec4 Pi = floor(P.xyxy) + vec4(0.0, 0.0, 1.0, 1.0);
  vec4 Pf = fract(P.xyxy) - vec4(0.0, 0.0, 1.0, 1.0);
  Pi = mod(Pi, 289.0); // To avoid truncation effects in permutation
  vec4 ix = Pi.xzxz;
  vec4 iy = Pi.yyww;
  vec4 fx = Pf.xzxz;
  vec4 fy = Pf.yyww;
  vec4 i = permute(permute(ix) + iy);
  vec4 gx = 2.0 * fract(i * 0.0243902439) - 1.0; // 1/41 = 0.024...
  vec4 gy = abs(gx) - 0.5;
  vec4 tx = floor(gx + 0.5);
  gx = gx - tx;
  vec2 g00 = vec2(gx.x,gy.x);
  vec2 g10 = vec2(gx.y,gy.y);
  vec2 g01 = vec2(gx.z,gy.z);
  vec2 g11 = vec2(gx.w,gy.w);
  vec4 norm = 1.79284291400159 - 0.85373472095314 * 
    vec4(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11));
  g00 *= norm.x;
  g01 *= norm.y;
  g10 *= norm.z;
  g11 *= norm.w;
  float n00 = dot(g00, vec2(fx.x, fy.x));
  float n10 = dot(g10, vec2(fx.y, fy.y));
  float n01 = dot(g01, vec2(fx.z, fy.z));
  float n11 = dot(g11, vec2(fx.w, fy.w));
  vec2 fade_xy = fade(Pf.xy);
  vec2 n_x = mix(vec2(n00, n01), vec2(n10, n11), fade_xy.x);
  float n_xy = mix(n_x.x, n_x.y, fade_xy.y);
  return 2.3 * n_xy;
}

// ^ Copypasted from https://gist.github.com/patriciogonzalezvivo/670c22f3966e662d2f83

varying vec2 v_vt;
varying vec2 v_world_pos;
varying vec2 v_camera_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_vt;
attribute vec2 a_normal;
attribute float a_flow;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float u_simulation_time;
uniform float u_height;
uniform float u_texture_shift;
void main() {
    v_vt = a_vt + vec2(u_texture_shift - a_flow * u_simulation_time, 0.0);
    vec2 tangent = vec2(-a_normal.y, a_normal.x);
    // vec2 wind_shift = tangent * sin(u_simulation_time * 3.0) * a_vt.y * 0.02;
    v_camera_pos = (u_view_matrix * vec3(a_pos, 1.0)).xy;
    v_world_pos = (u_projection_matrix * vec3(v_camera_pos, 1.0)).xy;
    gl_Position = vec4(v_world_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform float u_simulation_time;
uniform float u_flex_amplitude;
uniform float u_flex_frequency;
uniform vec4 u_layer_color;
uniform float u_reveal_radius;
void main() {
    gl_FragColor = texture2D(u_texture, v_vt + v_vt.y * vec2(cnoise(v_world_pos + vec2(0.0, u_simulation_time * u_flex_frequency)) * u_flex_amplitude, 0.0));
    gl_FragColor.a *= smoothstep(u_reveal_radius - 1.0, u_reveal_radius, length(v_camera_pos));
    gl_FragColor.rgb = gl_FragColor.rgb * (1.0 - u_layer_color.a) + u_layer_color.rbg * u_layer_color.a;
}
#endif