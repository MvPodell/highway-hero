// Where is the nth vertex in normalized device coordinates?
var<private> VERTICES:array<vec4<f32>,6> = array<vec4<f32>,6>(
    // In WGPU, the bottom left corner is -1,-1 and the top right is 1,1.
    vec4<f32>(-1., -1., 0., 1.),
    vec4<f32>(1., -1., 0., 1.),
    vec4<f32>(-1., 1., 0., 1.),
    vec4<f32>(-1., 1., 0., 1.),
    vec4<f32>(1., -1., 0., 1.),
    vec4<f32>(1., 1., 0., 1.)
);

// How does each vertex map onto the texture's corners?
var<private> TEX_COORDS:array<vec2<f32>,6> = array<vec2<f32>,6>(
    // Texture coordinates are a bit different---they go from 0,0 at the top left to 1,1 at the bottom right,
    // but if they are outside that bound they may clamp, or repeat the texture, or something else
    // depending on the sampler.
    vec2<f32>(0., 1.),
    vec2<f32>(1., 1.),
    vec2<f32>(0., 0.),
    vec2<f32>(0., 0.),
    vec2<f32>(1., 1.),
    vec2<f32>(1., 0.)
);

// Now we're outputting more than just a position,
// so we'll define a struct
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// vs_main now produces an instance of that struct...
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    // We'll just look up the vertex data in those constant arrays
    return VertexOutput(
        VERTICES[in_vertex_index],
        TEX_COORDS[in_vertex_index]
    );
}

// Now our fragment shader needs two "global" inputs to be bound:
// A texture...
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
// And a sampler.
@group(0) @binding(1)
var s_diffuse: sampler;
// Both are in the same binding group here since they go together naturally.

// Our fragment shader takes an interpolated `VertexOutput` as input now
@fragment
fn fs_main(in:VertexOutput) -> @location(0) vec4<f32> {
    // And we use the tex coords from the vertex output to sample from the texture.
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
