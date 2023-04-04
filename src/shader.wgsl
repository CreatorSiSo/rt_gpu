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

struct Ray {
	origin: vec3<f32>,
	direction: vec3<f32>
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect_ratio = f32(camera.width) / f32(camera.height);
    let height = 1.0;
    let width = 1.0 * aspect_ratio;
		// centered coordinates
    let coord = in.uv - 0.5;

    var ray: Ray;
    ray.origin = vec3(width * coord.x, height * coord.y, - 1.0);
    ray.direction = vec3(0.0, 0.0, 1.0);

		// a = ray.origin
		// b = ray.direction
		// r = sphere.radius
		// t = hit_distance
		// p = hit_point

		// ray
		// p = a + b*t
		// px = ax + bx*t
		// py = ay + by*t

		// circle
		// r^2 = (x - ox)^2 + (y - oy)^2
		// unit circle at origin => ox; ox = 0
		// r^2 = (x - 0)^2 + (y - 0)^2
		// r^2 = x^2 + y^2
		// 0 = x^2 + y^2 - r^2

		// px = x; py = y
		// 0 = (ax + bx*t)^2 + (ax + bx*t)^2 - r^2
		// 0 = ax^2 + 2*ax*bx*t + bx^2*t^2 + ay^2 + 2*ay*by*t + by^2*t^2 - r^2
		// 0 = t^2(bx^2 + by^2) + 2t(ax*bx + ay*by) + ax^2 + ay^2 - r^2
		// 0 = t^2*e + t*f + g

		// e = (bx^2 + by^2)
		// f = 2(ax*bx + ay*by)
		// g = ax^2 + ay^2 - r^2
    let e = (ray.direction.x * ray.direction.x) + (ray.direction.y * ray.direction.y) + (ray.direction.z * ray.direction.z);
    let f = 2.0 * ((ray.origin.x * ray.direction.x) + (ray.origin.y * ray.direction.y) + (ray.origin.z * ray.direction.z));
    let g = (ray.origin.x * ray.origin.x) + (ray.origin.y * ray.origin.y) + (ray.origin.z * ray.origin.z) - 0.5 * 0.5; // TODO use correct radius

		// discriminant
		// d = f^2 - 4eg
    let d = f * f - 4.0 * e * g;

		// quadratic formula
		// t = (-f +/- sqrt(d)) / 2e
    // let t_far = (-f + sqrt(d)) / (2.0 * e);
    let t_near = (-f - sqrt(d)) / (2.0 * e);

    // for (var i = 0u; i < arrayLength(&objects); i += 1u) {
    //     if objects[i].position.x > ray.origin.x && objects[i].position.y > ray.origin.y {
    //         return vec4(1.0);
    //     }
    // }

    if d < 0.0 {
        return vec4(0.0);
    }
    return vec4(vec3(t_near / 10.0), 1.0);
}
