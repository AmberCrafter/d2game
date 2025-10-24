struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_coords: vec2f,
    @location(2) norm: vec3f,
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
    @location(1) norm: vec3f,
    @location(2) view_pos: vec3f,
};

struct CameraUniform {
    view_pos: vec4f,
    view_proj: mat4x4f,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> transform: mat4x4f;

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

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * transform * vec4f(model.position, 1.0);
    out.norm = model.norm;
    out.view_pos = camera.view_pos.xyz;
    return out;
}


@group(0) @binding(0)
var<uniform> base_color: vec4f;
@group(0) @binding(1)
var<uniform> metallic: f32;
@group(0) @binding(2)
var<uniform> roughness: f32;
@group(0) @binding(11)
var<uniform> emissive: f32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var norm: vec3f = normalize(in.norm);
    var viewDir: vec3f = normalize(in.view_pos - in.clip_position.xyz);  // FragPos -> viewPos
    var lightDir: vec3f = normalize(in.view_pos);  // FragPos -> lightPos
    var reflectDir: vec3f = reflect(lightDir, norm);

    var lightWeight: f32 = max(dot(norm, lightDir), max(emissive, 0.1));
    var specWeight: f32 = 1.5 * pow(max(dot(viewDir, reflectDir), 0.0), roughness);


    // calculate
    var cal_diffuse: vec3f = lightWeight * base_color.xyz;// * vec3(texture(materialDiffuse, TexCoord));
    // var cal_specular: f32 = specWeight * metallic;// * vec3(texture(materialSpecular, TexCoord));

    var cal_val: vec3f = cal_diffuse;
    return vec4f(cal_val, 1.0);

    // return base_color;
}
