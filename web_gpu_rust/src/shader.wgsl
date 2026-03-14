struct Uniforms {
    angle: f32
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VSOut {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec3<f32>,
};

fn rotate(v: vec2<f32>, a: f32) -> vec2<f32> {

    let c = cos(a);
    let s = sin(a);

    return vec2<f32>(
        v.x * c - v.y * s,
        v.x * s + v.y * c
    );
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>
) -> VSOut {

    var out: VSOut;

    let p = rotate(position, uniforms.angle);

    out.position = vec4<f32>(p, 0.0, 1.0);
    out.color = color;

    return out;
}

@fragment
fn fs_main(
    @location(1) color: vec3<f32>
) -> @location(0) vec4<f32> {

    return vec4<f32>(color, 1.0);
}
