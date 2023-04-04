struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
		@location(1) uv: vec2<f32>
};

@vertex
fn vs_main(
    in: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4(in.position, 1.0);
    out.uv = in.uv;
    return out;
}

struct Camera {
	width: u32,
	height: u32,
}

@group(0)
@binding(0)
var<uniform> camera: Camera;

struct Sphere {
	radius: f32,
	position: vec3<f32>,
	color: vec3<f32>
};

@group(1)
@binding(0)
var<storage, read> objects: array<Sphere>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let position = vec2(in.uv.x * f32(camera.width), in.uv.y * f32(camera.height));
    // var v = vec3(0.2);
    // for (var i = 0u; i < arrayLength(&objects); i += 1u) {
    //     v += objects[i].color * 0.5;
    // }
    // return vec4<f32>(v, 1.0);
    return vec4(position, 0.0, 1.0);
}
