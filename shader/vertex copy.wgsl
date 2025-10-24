struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4f,
    @location(6) model_matrix_1: vec4f,
    @location(7) model_matrix_2: vec4f,
    @location(8) model_matrix_3: vec4f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
};

struct CameraUniform {
    view_proj: mat4x4f,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

// @group(2) @binding(0)
// var<storage, read_write> debug_storage: mat4x4f;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4f(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // debug_storage = mat4x4f(
    //     vec4f(1.0, 2.0, 3.0, 4.0),
    //     vec4f(model.position, 1.0),
    //     vec4f(model.position, 1.0),
    //     vec4f(model.position, 1.0),
    // );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4f(model.position, 1.0);
    return out;

    out.clip_position = camera.view_proj * vec4f(model.position.x, model.position.y, model.position.z, 1.0);
    // out.clip_position = camera.view_proj * vec4f(model.position.x, model.position.y, model.position.z, 1.0);
    return out;

    // var out: VertexOutput;
    // out.clip_position = vec4f(model, 0.0, 0.0, 1.0);
    // return out;
}
