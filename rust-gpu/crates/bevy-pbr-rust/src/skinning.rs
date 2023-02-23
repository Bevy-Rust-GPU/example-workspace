use spirv_std::glam::{Mat3, Mat4, Vec3};

pub fn inverse_transpose_3x3(input: Mat3) -> Mat3 {
    let x = input.y_axis.cross(input.z_axis);
    let y = input.z_axis.cross(input.x_axis);
    let z = input.x_axis.cross(input.y_axis);
    let det = input.z_axis.dot(z);
    Mat3 {
        x_axis: x / det,
        y_axis: y / det,
        z_axis: z / det,
    }
}

pub fn skin_normals(model: Mat4, normal: Vec3) -> Vec3 {
    (inverse_transpose_3x3(Mat3 {
        x_axis: model.x_axis.truncate(),
        y_axis: model.y_axis.truncate(),
        z_axis: model.z_axis.truncate(),
    }) * normal)
        .normalize()
}

