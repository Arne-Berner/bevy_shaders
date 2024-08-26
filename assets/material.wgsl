/// ***************************** ///
/// This is a shadertoy port of 'Tileable Water Caustic' by Dave_Hoskins, who claims to of sound it on glsl sandbox, by 'joltz0r' 
/// I have been unable to find the original.
/// ***************************** ///

#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals 
#import bevy_render::view::View

// @group(0) @binding(0) var<uniform> view: View;

const MAX_ITER: i32 = 3;
const SPEED:f32 = 1.0;
    
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // let time: f32 = globals.time * 0.5 + 23.0;
    // let resolution = view.viewport.zw;
    var uv: vec2<f32> = in.uv;
    
    let srgb = vec3(uv.x);
    let linear = SRGBToLinear(srgb);
    let y_rev = (1- srgb.x);
    let y = srgb.x;
    var color = vec3(y);
    let pct = plot(uv, y_rev);
    color = (1.0-pct)*linear+pct*vec3(0.0,1.0,0.0);

    return vec4<f32>(color, 1.0);
    // return vec4f(1.0);
}


fn plot(st: vec2f, pct: f32) -> f32{
  return  smoothstep( pct-0.01, pct, st.y) -
          smoothstep( pct, pct+0.01, st.y);
}

fn LessThan(f: vec3<f32>, value:f32) -> vec3<f32> {
    var x: f32 = 0.0;
    if f.x < value {
       x = 1.0; 
    }
    var y: f32 = 0.0;
    if f.y < value {
       y = 1.0; 
    }
    var z: f32 = 0.0;
    if f.z < value {
       z = 1.0; 
    }
    return vec3(x,y,z);
}

fn LinearToSRGB(rgb: vec3<f32>) -> vec3<f32> {
	var rgb_var: vec3f = rgb;
	rgb_var = clamp(rgb_var, vec3f(0.0,0.0,0.0) , vec3f(1.0, 1.0, 1.0));
	return mix(pow(rgb_var, vec3<f32>(1. / 2.4)) * 1.055 - 0.055, rgb_var * 12.92, LessThan(rgb_var, 0.0031308));
} 

fn SRGBToLinear(rgb: vec3<f32>) -> vec3<f32> {
	var rgb_var = rgb;
	rgb_var = clamp(rgb_var, vec3f(0.0), vec3f(1.0));
	return mix(pow((rgb_var + 0.055) / 1.055, vec3<f32>(2.4)), rgb_var / 12.92, LessThan(rgb_var, 0.04045));
} 

