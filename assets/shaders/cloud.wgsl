#import bevy_pbr::{
      mesh_view_bindings::view,
      forward_io::VertexOutput,
      forward_io::FragmentOutput,
      utils::coords_to_viewport_uv,
      pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
      pbr_types::pbr_input_new,
      pbr_types::StandardMaterial,
      pbr_types::standard_material_new,
      pbr_functions::alpha_discard,
      pbr_fragment::pbr_input_from_standard_material,
      mesh_view_bindings::globals,
}

const FAR_CLIP: f32 = 10.0;
const TERMINATION_DIST: f32 = 0.001;
const U32_HIGHEST: u32 = 4294967295u;
const NOISE_RES: u32 = 10;
const CLOUD_COLOR: vec3<f32> = vec3<f32>(0.5, 0.1, 1.0);
const ABSORPTION: f32 = 1.5;

@group(2) @binding(100) var<uniform> color: vec3<f32>;

@fragment
fn fragment(
	    mesh: VertexOutput,
	    // @builtin(sample_index) sample_index: u32,
	    ) ->  FragmentOutput {

  var out: FragmentOutput;
  out.color = vec4<f32>(color, 0);

  // march
  let cam_pos = view.world_position;
  var viewport_uv = coords_to_viewport_uv(mesh.position.xy, view.viewport) * 2.0 - 1.0;
  viewport_uv.y *= -1;
  let clip = vec4<f32>(viewport_uv, 1.0, 1.0);
  var world = view.world_from_clip * clip;
  world /= world.w;
  let ray_dir = normalize(world.xyz - cam_pos);
  var curr_pos = cam_pos;
  var dist_marched = 0.0;
  // march
  while dist_marched < FAR_CLIP {
      let dist = sdf_world(curr_pos);
      // hit enter point
      if dist < TERMINATION_DIST {
	  let enter_point = curr_pos;
	  // march for exit point
	  curr_pos += ray_dir * 1.0 * 2.0; // start from the other side
	  let ray_dir_back = -ray_dir;
	  dist_marched = 0.0;
	  while dist_marched < FAR_CLIP {
	      let dist = sdf_world(curr_pos);
	      if dist < TERMINATION_DIST {
		  let exit_point = curr_pos;
		  let color = color_cloud(enter_point, exit_point);
		  out.color = vec4<f32>(color);
		  return out;
	      }
	      dist_marched += dist;
	      curr_pos += ray_dir_back * dist;
	    }
	}
      curr_pos += ray_dir * dist;
      dist_marched += dist;
    }
  return out;
}

fn color_cloud(enter_point: vec3<f32>, exit_point: vec3<f32>) -> vec4<f32> {
  let val = 1.0 - exp(-distance(enter_point, exit_point) * ABSORPTION);
  return vec4<f32>(CLOUD_COLOR, val);
}

fn sdf_world(ray_position: vec3<f32>) -> f32 {
  // our cloud is at point (0.0, 1.5, 0.0)
  // TODO: pass it from material, don't hardcode
  let rp = translate_ray(ray_position, vec3<f32>(0.0, 1.5, 0.0));

  return sdf_cloud(rp, 1.0);
}

fn translate_ray(r: vec3<f32>, world_position: vec3<f32>) -> vec3<f32> {
  var out = r;
  // translation
  out -= world_position;

  // TODO: rotation
  // TODO: scale
  return out;
}

//  ,---.  ,------.  ,------.    ,------.                        ,--.  ,--.
// '   .-' |  .-.  \ |  .---'    |  .---',--.,--.,--,--,  ,---.,-'  '-.`--' ,---. ,--,--,  ,---.
// `.  `-. |  |  \  :|  `--,     |  `--, |  ||  ||      \| .--''-.  .-',--.| .-. ||      \(  .-'
// .-'    ||  '--'  /|  |`       |  |`   '  ''  '|  ||  |\ `--.  |  |  |  |' '-' '|  ||  |.-'  `)
// `-----' `-------' `--'        `--'     `----' `--''--' `---'  `--'  `--' `---' `--''--'`----'
fn sdf_cloud(p: vec3<f32>, rad: f32) -> f32 {
  let circle = length(p) - rad;
  let circle_normal = p * 2.0;

  return circle;
}

// https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39
//  <https://www.shadertoy.com/view/Xd23Dh>
//  by Inigo Quilez
//
fn hash23(p: vec2f) -> vec3f {
    let q = vec3f(dot(p, vec2f(127.1, 311.7)),
        dot(p, vec2f(269.5, 183.3)),
        dot(p, vec2f(419.2, 371.9)));
    return fract(sin(q) * 43758.5453);
}
fn voroNoise2(x: vec2f, u: f32, v: f32) -> f32 {
    let p = floor(x);
    let f = fract(x);
    let k = 1. + 63. * pow(1. - v, 4.);
    var va: f32 = 0.;
    var wt: f32 = 0.;
    for(var j: i32 = -2; j <= 2; j = j + 1) {
        for(var i: i32 = -2; i <= 2; i = i + 1) {
            let g = vec2f(f32(i), f32(j));
            let o = hash23(p + g) * vec3f(u, u, 1.);
            let r = g - f + o.xy;
            let d = dot(r, r);
            let ww = pow(1. - smoothstep(0., 1.414, sqrt(d)), k);
            va = va + o.z * ww;
            wt = wt + ww;
        }
    }
    return va / wt;
}

// ------- noise -------

fn hashed_pos_f32(p: vec3u) -> vec3<f32> {
  let hashed_pos = pcg3d(p);
  let hashed_pos_f32 = vec3<f32>(f32(hashed_pos.x) / f32(U32_HIGHEST),
			       f32(hashed_pos.y) / f32(U32_HIGHEST),
			       f32(hashed_pos.z) / f32(U32_HIGHEST));
  return hashed_pos_f32;
}

// http://www.jcgt.org/published/0009/03/02/
fn pcg3d(p: vec3u) -> vec3u {
    var v = p * 1664525u + 1013904223u;
    v.x += v.y*v.z; v.y += v.z*v.x; v.z += v.x*v.y;
    v ^= v >> vec3u(16u);
    v.x += v.y*v.z; v.y += v.z*v.x; v.z += v.x*v.y;
    return v;
}
