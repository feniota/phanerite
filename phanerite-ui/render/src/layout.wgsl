struct Viewport {
    physical_size: vec2<f32>,
    scale_factor: f32,
    padding: f32,
}
@group(0) @binding(0) var<uniform> viewport: Viewport;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex: u32,
    @location(0) rect: vec4<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    // Accumulated logical edges are snapped only after physical scaling.
    let low = round(rect.xy * viewport.scale_factor);
    let high = round((rect.xy + rect.zw) * viewport.scale_factor);
    var edge_low = low;
    var edge_high = high;
    switch vertex / 6u {
        case 0u: { edge_high.y = min(low.y + 1.0, high.y); }
        case 1u: { edge_low.y = max(high.y - 1.0, low.y); }
        case 2u: { edge_high.x = min(low.x + 1.0, high.x); }
        default: { edge_low.x = max(high.x - 1.0, low.x); }
    }
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let physical = mix(edge_low, edge_high, corners[vertex % 6u]);
    let ndc = physical / viewport.physical_size * vec2<f32>(2.0, -2.0)
        + vec2<f32>(-1.0, 1.0);
    var output: VertexOutput;
    output.position = vec4<f32>(ndc, 0.0, 1.0);
    output.color = color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
