struct Sphere {
	radius: f32,
	position: vec3<f32>,
	color: vec3<f32>
};

@group(0)
@binding(0)
var<storage, read> objects: array<Sphere>;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>
};

@vertex
fn vs_main(
    quad: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4(quad.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var v = vec3(0.2);
    for (var i = 0u; i < arrayLength(&objects); i += 1u) {
        v += objects[i].color * 0.5;
    }
    return vec4<f32>(v, 1.0);
}
