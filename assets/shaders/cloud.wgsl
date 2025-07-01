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

const EPSILON: f32 = 2.718281828;
const PI: f32 = 3.141592654;
const FAR_CLIP: f32 = 10.0;
const TERMINATION_DIST: f32 = 0.001;
const U32_HIGHEST: u32 = 4294967295u;
const NOISE_RES: u32 = 10;
const CLOUD_COLOR: vec3<f32> = vec3<f32>(0.0);
const ABSORPTION: f32 = 1;

const TEST_LIGHT: LightInfo = LightInfo(1, vec3<f32>(1.5), 5000.0);
struct LightInfo {
 light_type: u32,
 custom_var: vec3<f32>,
 illuminance: f32,
}

@group(2) @binding(100) var<uniform> color: vec3<f32>;

@fragment
fn fragment(
    mesh: VertexOutput,
	    // @builtin(sample_index) sample_index: u32,
) -> FragmentOutput {

    var out: FragmentOutput;
    out.color = vec4<f32>(0); // pretend that we don't hit

    let cam_pos = view.world_position;
    var viewport_uv = coords_to_viewport_uv(mesh.position.xy, view.viewport) * 2.0 - 1.0;
    viewport_uv.y *= -1;
    let clip = vec4<f32>(viewport_uv, 1.0, 1.0);
    var world = view.world_from_clip * clip;
    world /= world.w;
    let ray_dir = normalize(world.xyz - cam_pos);
    var curr_pos = cam_pos;
    var dist_marched = 0.0;
    let enter_pos_march = perform_march(cam_pos, ray_dir);
    if !enter_pos_march.has_hit {
	return out;
      }
    // TODO: this should be twice the radius, not magic numbers
    let behind_cloud_pos = enter_pos_march.hit + ray_dir * 1.0 * 3.0;
    let exit_pos_march = perform_march(behind_cloud_pos, -ray_dir);
    if !exit_pos_march.has_hit {
	return out;
      }
    out.color = color_cloud(enter_pos_march.hit, exit_pos_march.hit);
    return out;
}

fn beers_law(distance: f32, absorption: f32) -> f32 {
  return exp(-distance * absorption);
}

fn powders_beers_law(distance: f32, absorption: f32) -> f32 {
  let beer = beers_law(distance, absorption);
  return beer * pow(EPSILON, -distance * absorption);
}

const ABSORPTION_SAMPLE_AMOUNT = 10;
fn color_cloud(enter_point: vec3<f32>, exit_point: vec3<f32>) -> vec4<f32> {
    let dir = normalize(exit_point - enter_point);
    let step_len = distance(enter_point, exit_point) / f32(ABSORPTION_SAMPLE_AMOUNT);
    var curr_point = enter_point;
    var absorption_sum = 0.0;
    for (var i = 0; i < ABSORPTION_SAMPLE_AMOUNT; i++) {
      absorption_sum += clamp(my_noise(curr_point), 0.0, 1.0);
      curr_point += (dir * step_len);
    }
    let absorption = absorption_sum / f32(ABSORPTION_SAMPLE_AMOUNT);
    var alpha = 1 - powders_beers_law(distance(enter_point, exit_point), absorption * ABSORPTION);
    // overdrive alpha so edges aren't of the sdf shape
    alpha -= 0.1;
    alpha *= 1.2;
    return vec4<f32>(1.0, 1.0, 1.0, alpha);
}

// Optional type please respond to my texts :(
struct MarchOutput {
 has_hit: bool,
 hit: vec3<f32>,
}
fn perform_march(start_pos: vec3<f32>, dir: vec3<f32>) -> MarchOutput {
  let ndir = normalize(dir);
  var marched_dist = 0.0;
  var curr_point = start_pos;
  while marched_dist < FAR_CLIP {
      let dist = sdf_world(curr_point);
      if dist < TERMINATION_DIST {
	  return MarchOutput(true, curr_point);
	}
      marched_dist += dist;
      curr_point += ndir * dist;
    }
  return MarchOutput(false, curr_point);
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
    let cube = sdBox(p, vec3<f32>(0.8));


    return circle;
}

fn sdBox(p: vec3f, b: vec3f) -> f32 {
  let q = abs(p) - b;
  return length(max(q, vec3f(0.))) + min(max(q.x, max(q.y, q.z)), 0.);
}

// ------- noise -------

// sample this for the cloud - the rest are helpers
fn my_noise(p: vec3<f32>) -> f32 {
    var out: f32;
    let pt = p * 2 + globals.time / 3;
    // let pt = p;
    out += worley_noise(pt) * 2;
    out -= noise3(pt * 5) * 0.5;
    // swivel = clamp(swivel, 0, 1);
    // out = 0.5;
    return out;
}

fn worley_noise(p: vec3<f32>) -> f32 {
    let points = permutation_points(p);
    var closest = distance(p, points[0]);
    for (var i = 1; i < 27; i++) {
        let dist = distance(p, points[i]);
        if dist < closest {
            closest = dist;
        }
    }
    return 1-closest;
}


const WORLEY_WORLD_SIZE = 20;
fn permutation_points(p: vec3<f32>) -> array<vec3<f32>, 27> {
    var points = array<vec3<f32>, 27>();
    var i = 0;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            for (var z = -1; z <= 1; z++) {
                let offset = vec3<f32>(f32(x), f32(y), f32(z));
                let floored = floor(p) + offset;
                let p_floored_worley_world = vec3<u32> (
                    u32(floored_mod(floored.x, f32(WORLEY_WORLD_SIZE))),
                    u32(floored_mod(floored.y, f32(WORLEY_WORLD_SIZE))),
                    u32(floored_mod(floored.z, f32(WORLEY_WORLD_SIZE))),
                );
                let hash = hashed_pos_f32(p_floored_worley_world);
                points[i] = floored + hash;
                i++;
            }
        }
    }
    return points;
}

fn floored_mod(x: f32, m: f32) -> f32 {
    return fract(x / m) * m;
}


// MIT License. © Stefan Gustavson, Munrocket
//
fn mod289(x: vec4f) -> vec4f { return x - floor(x * (1. / 289.)) * 289.; }
fn perm4(x: vec4f) -> vec4f { return mod289(((x * 34.) + 1.) * x); }

fn noise3(p: vec3f) -> f32 {
    let a = floor(p);
    var d: vec3f = p - a;
    d = d * d * (3. - 2. * d);

    let b = a.xxyy + vec4f(0., 1., 0., 1.);
    let k1 = perm4(b.xyxy);
    let k2 = perm4(k1.xyxy + b.zzww);

    let c = k2 + a.zzzz;
    let k3 = perm4(c);
    let k4 = perm4(c + 1.);

    let o1 = fract(k3 * (1. / 41.));
    let o2 = fract(k4 * (1. / 41.));

    let o3 = o2 * d.z + o1 * (1. - d.z);
    let o4 = o3.yw * d.x + o3.xz * (1. - d.x);

    return o4.y * d.y + o4.x * (1. - d.y);
}

fn hashed_pos_f32(p: vec3u) -> vec3<f32> {
    let hashed_pos = pcg3d(p);
    let hashed_pos_f32 = vec3<f32>(
        f32(hashed_pos.x) / f32(U32_HIGHEST),
        f32(hashed_pos.y) / f32(U32_HIGHEST),
        f32(hashed_pos.z) / f32(U32_HIGHEST)
    );
    return hashed_pos_f32;
}

// http://www.jcgt.org/published/0009/03/02/
fn pcg3d(p: vec3u) -> vec3u {
    var v = p * 1664525u + 1013904223u;
    v.x += v.y * v.z; v.y += v.z * v.x; v.z += v.x * v.y;
    v ^= v >> vec3u(16u);
    v.x += v.y * v.z; v.y += v.z * v.x; v.z += v.x * v.y;
    return v;
}

fn henry_greenstein(g: f32, costh: f32) -> f32 {
  return (1.0 / (4.0 * PI)) *
    ((1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * costh, 1.5));
}
