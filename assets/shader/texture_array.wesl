import bevy_pbr::render::forward_io::VertexOutput;

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var tiles: texture_2d_array<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var tiles_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let layer = i32(round(in.color.r));
    let texel = textureSample(tiles, tiles_sampler, in.uv, layer);

    let n = normalize(in.world_normal);
    var shade = 0.8;
    if n.y > 0.5 {
        shade = 1.0;
    } else if n.y < -0.5 {
        shade = 0.5;
    } else if abs(n.x) > 0.5 {
        shade = 0.7;
    }

    return vec4<f32>(texel.rgb * shade, 1.0);
}