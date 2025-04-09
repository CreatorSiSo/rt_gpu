// IEEE 754 maximum value for 32 bit floats
const f32_max = 3.4028235e38;

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
var<storage, read> spheres: array<Sphere>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect_ratio = f32(camera.width) / f32(camera.height);
    let height = 1.0;
    let width = 1.0 * aspect_ratio;

    let coord = vec2(in.uv.x * width, height * in.uv.y) * 2.0;

    var ray: Ray;
    ray.origin = vec3(0.0, 0.0, -2.0);
    ray.direction = normalize(vec3(coord.x, coord.y, 0.0) - ray.origin);
    // ray.origin = vec3(coord, -3.0);
    // ray.direction = vec3(0.0, 0.0, 1.0);

    var hit: Hit;
    hit.distance = f32_max;
    var nearest_sphere: Sphere;

    for (var i = 0u; i <= arrayLength(&spheres); i += 1u) {
        let sphere = spheres[i];
        let maybe_hit = hit_sphere(ray, sphere);
        if maybe_hit.intersected && hit.distance > maybe_hit.distance {
            hit = maybe_hit;
            nearest_sphere = sphere;
        }
    }

    if !hit.intersected {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }

    let hit_pos = position_on_ray(ray, hit.distance);
    let normal = sphere_normal(nearest_sphere, hit_pos);
    let light = dot(normal, normalize(vec3(1.0, 1.0, -1.0)));
    let color = nearest_sphere.color * light;
    return color;
}

struct Hit {
  intersected: bool,
  distance: f32,
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
    // p_x = a_x + b_x*t
    // p_y = a_y + b_y*t

    // circle
    // r^2 = (x - o_x)^2 + (y - o_y)^2
    // unit circle at origin => o = (0, 0)
    // r^2 = (x - 0)^2 + (y - 0)^2
    // r^2 = x^2 + y^2
    // 0 = x^2 + y^2 - r^2

    // x = p_x
    // y = p_y

    // 0 = p_x^2 + p_y^2 - r^2
    // 0 = (a_x + b_x*t)^2 + (a_y + b_y*t)^2 - r^2
    // 0 = a_x^2 + 2*a_x*b_x*t + b_x^2 * t^2 + a_y^2 + 2*a_y*b_y*t + b_y^2 * t^2 - r^2
    // 0 = t^2 * (b_x^2 + b_y^2) + 2t(a_x*b_x + a_y*b_y) + ax^2 + ay^2 - r^2
    // 0 = t^2 * (b.b) + 2t(a.b) + a.a - r^2
    // 0 = t^2 * e + t*f + g

    // e = b.b
    // f = 2*(a.b)
    // g = a.a - r^2
    let e = dot(b, b);
    let f = 2.0 * dot(a, b);
    let g = dot(a, a) - (sphere.radius * sphere.radius);

    // discriminant
    // d = f^2 - 4*e*g
    let d = f * f - 4.0 * e * g;

    var hit: Hit;

    if d < 0.0 {
        hit.intersected = false;
        return hit;
    }

    // quadratic formula
    // t = (-f +/- sqrt(d)) / 2*e
    // let t_far = (-f + sqrt(d)) / (2.0 * e);
    let t_near = (-f - sqrt(d)) / (2.0 * e);

    hit.intersected = true;
    hit.distance = t_near;
    return hit;
}

fn sphere_normal(sphere: Sphere, position: vec3<f32>) -> vec3<f32> {
    return normalize(position - sphere.position);
}

struct Ray {
  origin: vec3<f32>,
  direction: vec3<f32>
}

fn position_on_ray(ray: Ray, distance: f32) -> vec3<f32> {
    return ray.origin + (ray.direction * distance);
}
