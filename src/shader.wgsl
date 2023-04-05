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
  position: vec3<f32>,
  radius: f32,
  color: vec4<f32>,
}

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

    let coord = vec2(in.uv.x * width, height * in.uv.y) * 2.0;

    if (coord.x >= -0.01 && coord.x <= 0.01) || (coord.y >= -0.01 && coord.y <= 0.01) {
        return vec4(0.1);
    }

    var ray: Ray;
    // ray.origin = vec3(0.0, 0.0, -2.0);
    // ray.direction = normalize(vec3(coord.x, coord.y, 0.0) - ray.origin);
    ray.origin = vec3(coord, -3.0);
    ray.direction = vec3(0.0, 0.0, 1.0);

    var hit: Hit;
    for (var i = 0u; i <= arrayLength(&objects); i += 1u) {
        let maybe_hit = hit_sphere(ray, objects[i]);
        if maybe_hit.valid {
            hit = maybe_hit;
        }
    }

    let light = dot(hit.normal, normalize(vec3(1.0, 1.0, -1.0)));
    let color = vec3(light);
    return vec4(color, 1.0);
}

struct Hit {
  valid: bool,
  pos: vec3<f32>,
	normal: vec3<f32>
}

fn hit_sphere(ray: Ray, sphere: Sphere) -> Hit {
    // a = ray.origin
    // b = ray.direction
    // r = sphere.radius
    // t = hit_distance
    // p = hit_point

		// Accord for the sphere not beeing centered
    let a = ray.origin - sphere.position;
    let b = ray.direction;


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

    // e = (bx^2 + by^2) = b*b
    // f = 2(ax*bx + ay*by) = 2*a*b
    // g = ax^2 + ay^2 - r^2 = a*a - r^2
    let e = dot(b, b);
    let f = 2.0 * dot(a, b);
    let g = dot(a, a) - (sphere.radius * sphere.radius);

    // discriminant
    // d = f^2 - 4eg
    let d = f * f - 4.0 * e * g;

    var hit: Hit;

    if d < 0.0 {
        hit.valid = false;
        return hit;
    }

    // quadratic formula
    // t = (-f +/- sqrt(d)) / 2e
    let t_far = (-f + sqrt(d)) / (2.0 * e);
    let t_near = (-f - sqrt(d)) / (2.0 * e);

    hit.valid = true;
    hit.pos = ray.origin + (ray.direction * t_near);
    hit.normal = normalize(hit.pos - sphere.position);
    return hit;
}
