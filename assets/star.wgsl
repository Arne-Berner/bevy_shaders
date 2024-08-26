// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_functions

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
};

struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    @builtin(position) clip_position: vec4<f32>,
};

fn plot(st:vec2f,pct:f32) -> f32{
  return  smoothstep( pct-0.02, pct, st.y) -
          smoothstep( pct, pct+0.02, st.y);
}

/// Entry point for the vertex shader
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position
    let model = mesh2d_functions::get_world_from_local(vertex.instance_index);
    out.clip_position = mesh2d_functions::mesh2d_position_local_to_clip(model, vec4<f32>(vertex.position*1280, 1.0));
    // Unpack the `u32` from the vertex buffer into the `vec4<f32>` used by the fragment shader
    return out;
}


/// Entry point for the fragment shader
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let res = vec2(1280, 720);
    let st = vec2(in.clip_position.x/1280.0, in.clip_position.y/720.0);
    let y = st.x;
    var color: vec3<f32>;
    color = vec3(y);
    let pct = plot(st,y);
    color = (1.0-pct)*color+pct*vec3(0.0,1.0,0.0);
    return vec4(color, 1.0);
}

// // Import the standard 2d mesh uniforms and set their bind groups
// #import bevy_sprite::mesh2d_functions
// 
// // The structure of the vertex buffer is as specified in `specialize()`
// struct Vertex {
//     @builtin(instance_index) instance_index: u32,
//     @location(0) position: vec3<f32>,
//     @location(1) color: u32,
// };
// 
// struct VertexOutput {
//     // The vertex shader must set the on-screen position of the vertex
//     @builtin(position) clip_position: vec4<f32>,
// };

/// Entry point for the vertex shader
// @vertex
// fn vertex(vertex: Vertex) -> VertexOutput {
//     var vertices = array<vec2<f32>, 3>(
//         vec2<f32>(-1., 1.),
//         vec2<f32>(-1., -3.),
//         vec2<f32>(3., 1.),
//     );
//     let x = vertices[vertex.instance_index].x;
//     let y = vertices[vertex.instance_index].y;
//     var out: VertexOutput;
//     // let x = f32(1 - i32(in_vertex_index)) * 0.5;
//     // let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
//     out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
//     return out;
// }
// 
// 
// /// Entry point for the fragment shader
// @fragment
// fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
//     return vec4(1.0);
// }
