#![no_std]
#![feature(asm_experimental_arch)]

pub use bevy_pbr_rust;

use rust_gpu_bridge::glam;

use bevy_pbr_rust::prelude::{Globals, Mesh, TextureDepth2d, View};
use permutate_macro::permutate;
use rust_gpu_bridge::{Mix, Mod, SmoothStep};
use rust_gpu_sdf::{
    default,
    prelude::{
        items::position::Position, AttrBoundError, AttrColor, AttrDistance, AttrNormal,
        AttrSupport, AttrTangent, AttrUv, BoundError, Capsule, CartesianToPolar, Checker, Circle,
        ErrorTerm, Extrude, FieldAttribute, FieldAttributes, FieldAttributesRegisterCons,
        FieldAttributesRegistersCons, NormalTetrahedron, PolarToCartesian, ProxyColor, Raycast,
        RaycastInput, SmoothSubtraction, Sphere, SphereTraceLipschitz, SupportFunction, Translate,
        UvTangent,
    },
    type_fields::{
        field::Field as TypeField,
        t_funk::{
            hlist::{PushFront, ToTList},
            tlist::ToHList,
            Pointed, Tagged,
        },
    },
};
use spirv_std::{
    arch::{ddx, ddy},
    glam::{Mat3, UVec3, Vec2, Vec3, Vec4, Vec4Swizzles},
    spirv,
};

#[allow(unused_imports)]
use spirv_std::num_traits::Float;

#[spirv(vertex)]
pub fn vertex_warp(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[spirv(uniform, descriptor_set = 0, binding = 9)] globals: &Globals,

    in_position: Vec3,
    in_normal: Vec3,

    #[spirv(position)] out_clip_position: &mut Vec4,
    out_world_normal: &mut Vec3,
) {
    let mut position_local = in_position.extend(1.0);

    position_local.x += position_local.x * position_local.z * globals.time.sin();
    position_local.y += position_local.y * position_local.z * globals.time.cos();
    position_local.z += position_local.z * globals.time.sin() * globals.time.cos();

    let position_world = mesh.model * position_local;
    let position_clip = view.view_proj * position_world;

    *out_clip_position = position_clip;
    *out_world_normal = in_normal;
}

#[spirv(fragment)]
#[allow(unused_variables)]
pub fn fragment_normal(
    #[spirv(frag_coord)] in_clip_position: Vec4,
    in_world_normal: Vec3,
    out_color: &mut Vec4,
) {
    *out_color = in_world_normal.extend(1.0);
}

#[spirv(vertex)]
pub fn vertex_sdf_2d(
    in_position: Vec3,

    #[spirv(position)] out_position: &mut Vec4,
    out_clip_position: &mut Vec4,
) {
    let position_local = in_position.extend(1.0);
    *out_position = position_local;
    *out_clip_position = position_local;
}

pub trait TriangleWave {
    fn triangle_wave(self) -> Self;
}

impl TriangleWave for f32 {
    fn triangle_wave(self) -> Self {
        1.0 - 2.0 * ((self / 2.0).round() - (self / 2.0)).abs()
    }
}

#[spirv(fragment)]
#[allow(unused_variables)]
pub fn fragment_sdf_2d(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] globals: &Globals,
    in_clip_position: Vec4,
    out_color: &mut Vec4,
) {
    const COLOR_EXTERIOR: Vec3 = Vec3::new(0.8, 0.31, 0.1);
    const COLOR_INTERIOR: Vec3 = Vec3::new(0.4, 0.67, 1.0);
    const COLOR_BOUND: Vec3 = Vec3::new(0.0, 0.0, 0.0);

    let mut pos = in_clip_position;
    let aspect = view.viewport.z / view.viewport.w;
    pos.x *= aspect;
    pos *= 4.0;

    //let sdf = Point::default();
    //let sdf = ChebyshevMetric::default();
    //let sdf = TaxicabMetric::default();

    //let sdf = Circle::default();
    //let sdf = Square::default();

    //let sdf = Triangle::triangle();
    //let sdf = Quadrilateral::quadrilateral();
    //let sdf = Pentagon::pentagon();
    //let sdf = Hexagon::hexagon();
    //let sdf = Septagon::septagon();
    //let sdf = Octagon::octagon();
    //let sdf = Nonagon::nonagon();
    //let sdf = Decagon::decagon();
    //let sdf = Squircle::default();

    /*
    let sdf = Slice::<Rotate3d<Capsule<Vec3>>>::default().with(
        (Slice::target, Rotate3d::rotation),
        Quat::from_axis_angle(
            Vec3::new(0.0, 1.0, 0.0).normalize(),
            globals.time.modulo(core::f32::consts::TAU),
        ),
    );
    */

    let sdf = Translate::<
        Vec2,
        PolarToCartesian<CartesianToPolar<Translate<Vec2, Capsule<Vec2>>>>,
    >::default()
    .with(Translate::translation, Vec2::ZERO)
    .with(
        (
            Translate::target,
            PolarToCartesian::target,
            CartesianToPolar::target,
            Translate::translation,
        ),
        Vec2::Y * 2.0,
    );

    //let sdf = Capsule::default();
    //let sdf = Isosurface::<TaxicabMetric>::default();
    //let sdf = Isosurface::<ChebyshevMetric>::default();

    /*
    let sdf =
        Isosurface::<Superellipse>::default().with((Isosurface::target, Superellipse::n), 2.45);
    */

    let sdf = NormalTetrahedron::default()
        .with(NormalTetrahedron::sdf, sdf)
        .with(NormalTetrahedron::epsilon, 0.01);

    let (dist, norm) =
        sdf.field_attributes::<(AttrDistance<Vec2>, AttrNormal<Vec2>)>(&pos.xy().into());
    let dist = *dist;
    let norm = *norm;

    let norm_remapped = norm * 0.5 + 0.5;

    let col = if dist > 0.0 {
        COLOR_EXTERIOR
    } else {
        COLOR_INTERIOR
    };

    let mut col = norm_remapped.extend(0.0);

    let sdf = BoundError {
        target: SupportFunction {
            target: sdf,
            ..default()
        },
        ..default()
    };

    let error_term = sdf.field_attribute::<AttrBoundError<Vec2>>(&pos.xy().into());

    // Blue interior
    col.z = -dist.signum();

    // Boundary fade
    col *= 1.0 - (-3.0 * dist.abs()).exp();

    // Annulation
    col *= 0.65 + 0.35 * (150.0 * dist).cos();

    // White boundary
    col = col.mix(
        Vec3::splat(1.0),
        Vec3::splat(1.0 - dist.abs().smooth_step(0.0, 0.01)),
    );

    // Visualize boundedness
    col = COLOR_BOUND.mix(
        col,
        Vec3::splat(1.0 - error_term.error.abs().clamp(0.0, 1.0)),
    );

    *out_color = col.extend(1.0);
}

#[spirv(vertex)]
pub fn vertex_sdf_3d(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    in_position: Vec3,

    #[spirv(position)] out_position: &mut Vec4,
    //out_clip_position: &mut Vec4,
    out_world_position: &mut Vec4,
) {
    let position_local = in_position.extend(1.0);

    let position_world = mesh.model * position_local;
    let position_clip = view.view_proj * position_world;

    *out_position = position_clip;
    //*out_clip_position = position_clip;
    *out_world_position = position_world;
}

#[permutate(
    parameters = {},
    constants = {},
    types = {
        Sdf
    },
    permutations = [
        file("../../entry_points.json", ""),
        env("RUST_GPU_SDF_FRAGMENT_3D_PERMUTATIONS", "")
    ]
)]
#[spirv(fragment)]
pub fn fragment_sdf_3d(
    #[spirv(uniform, descriptor_set = 0, binding = 0)] view: &View,
    #[spirv(uniform, descriptor_set = 0, binding = 9)] globals: &Globals,
    #[spirv(descriptor_set = 0, binding = 16)] depth_prepass_texture: &TextureDepth2d,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] mesh: &Mesh,
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(front_facing)] in_is_front: bool,
    //in_clip_position: Vec4,
    in_world_position: Vec4,
    out_color: &mut Vec4,
) {
    const MAX_STEPS: u32 = 400;

    let camera = view.view.col(3);
    let ray_delta = in_world_position - camera;
    let ray_dist = ray_delta.length();
    let ray_direction = ray_delta.normalize();
    let object = mesh.model.col(3);

    let inv_model_rot = Mat3::from_mat4(mesh.model).transpose();

    let mut start = 0.0;
    let mut end = 1000.0;

    // World depth
    let depth = ray_dist;

    // Prepass depth
    /*
    let mut deproj_pos = view.inverse_projection
        * in_clip_position
            .xy()
            .extend(depth_prepass_texture.read(in_frag_coord.xy().as_ivec2()).x)
            .extend(1.0);
    deproj_pos.z /= deproj_pos.w;
    let depth = deproj_pos.xyz().length();
    */

    if in_is_front {
        start = depth;
    } else {
        end = depth;
    }

    /*
    let sdf = Torus::default()
        .with((Torus::core, Circle::radius), 0.75)
        .with((Torus::shell, Circle::radius), 0.25);
    */

    //let sdf = Sphere::default();

    /*
    let sdf = Sweep::<Circle, Square>::default()
        .with((Sweep::core, Circle::radius), 0.75)
        .with((Sweep::shell, Square::extent), Vec2::ONE * 0.25);
    */

    //let sdf = Point::default();
    //let sdf = Octahedron::default();
    //let sdf = Cube::default();
    //let sdf = Plane::default();
    //let sdf = Capsule::<Vec3>::default();
    //let sdf = Isosurface::<ChebyshevMetric>::default();
    /*
    let sdf = Union::<Extrude<Circle>, Sphere>::default()
        .with((Union::sdf_a, Extrude::depth), 0.5)
        .with(
            (Union::sdf_b, Sphere::radius),
            (globals.time * 2.0).sin() + 1.0,
        );
    */
    //let sdf = Extrude::<Circle>::default().with(Extrude::depth, 0.5);
    /*
    let ofs = 1.75;
    let sdf = SphereTraceLipschitz::<
        400,
        NormalTetrahedron<SmoothSubtraction<Extrude<Circle>, Sphere>>,
    >::default()
    .with(
        (
            SphereTraceLipschitz::target,
            NormalTetrahedron::sdf,
            SmoothSubtraction::k,
        ),
        0.5,
    )
    .with(
        (
            SphereTraceLipschitz::target,
            NormalTetrahedron::sdf,
            SmoothSubtraction::sdf_a,
            Extrude::depth,
        ),
        0.5,
    )
    .with(
        (
            SphereTraceLipschitz::target,
            NormalTetrahedron::sdf,
            SmoothSubtraction::sdf_b,
            Sphere::radius,
        ),
        (globals.time.sin() + ofs) / ofs,
    );
    */
    //let sdf = ExtrudeInterior::<Circle>::default().with(ExtrudeInterior::depth, 0.85);

    let sdf = <Sdf>::default();

    let dir = inv_model_rot * ray_direction.truncate();

    let eye = inv_model_rot * (camera.truncate() - object.truncate());

    let inverse_transpose_rot = Mat3::from_mat4(mesh.inverse_transpose_model);

    let frag_size = ddx(in_frag_coord.x / (view.viewport.z - 1.0)).abs()
        + ddy(in_frag_coord.y / (view.viewport.w - 1.0)).abs();

    let context = (
        inverse_transpose_rot,
        frag_size,
        RaycastInput {
            start,
            end,
            eye,
            dir,
        },
    );

    let (out,) = sdf.field_attributes_register_cons::<(Raycast,)>(&context);
    let context = context.to_hlist().push_front(out).to_tlist();

    // Context parameters
    let context = context
        .to_hlist()
        .push_front(Position(eye + dir * out.closest_t))
        .to_tlist();

    // Evaluate
    let (normal,) = sdf.field_attributes_register_cons::<(AttrNormal<Vec3>,)>(&context);
    let normal = *normal;

    // TODO: Convert into ops that take extra context params
    let light = Vec3::new(-1.0, 1.0, 1.0)
        .normalize()
        .dot(inverse_transpose_rot * normal);

    let glow = (out.steps as f32 / MAX_STEPS as f32).powf(4.0) * MAX_STEPS as f32;

    // Scale antialias width in correspondence with screen resolution
    // Roughly corresponds to 1px per K with a min bound of 2
    // i.e. 1K / 2K screens get 2px, 4K get 4px, and so on
    let coverage_grad = (view.viewport.w / 540.0).max(2.0);
    let coverage =
        (out.closest_dist - frag_size * coverage_grad).smooth_step(frag_size * coverage_grad, 0.0);

    let (color,) = sdf.field_attributes_register_cons::<(AttrColor<Vec3>,)>(&context);
    let col = color.xyz();

    //let col = (inverse_transpose_rot * normal) * 0.5 + 0.5;

    let glow_col = col * Vec3::splat(glow);
    let glow_col = glow_col * (1.0 - coverage);

    let col = col * Vec3::splat(light);

    let col = col * coverage;

    let col = col + glow_col;

    //let col_exterior = col * glow;

    //let col = col_exterior.mix(col, Vec3::splat(coverage));

    *out_color = col.extend(coverage);

    // Type machine testing
    /*
    {
        use rust_gpu_sdf::{
            field_type_machine::{
                EuclideanMetric, RayDirection, RayEnd, RayPosition, RayStart, RaymarchSteps,
                SphereTraceLipschitz,
            },
            prelude::Distance,
            type_fields::type_machine::TypeMachine,
        };

        /*
        (SphereTraceLipschitz::<400, EuclideanMetric<Vec3>>::default(),).run((
            Position(Vec3::ZERO),
            Distance(0.0),
            RaymarchSteps(0),
            RayPosition(eye),
            RayDirection(dir),
            RayStart(start),
            RayEnd(end),
            EuclideanMetric::<Vec3>::default(),
        ));
        */

        (EuclideanMetric::<Vec3>::default(),)
            .run((Tagged::<rust_gpu_sdf::field_type_machine::Position, _>::point(Vec3::ZERO),));
    }
    */
}

pub fn collatz(mut n: u32) -> Option<u32> {
    let mut i = 0;
    if n == 0 {
        return None;
    }
    while n != 1 {
        n = if n % 2 == 0 {
            n / 2
        } else {
            // Overflow? (i.e. 3*n + 1 > 0xffff_ffff)
            if n >= 0x5555_5555 {
                return None;
            }
            // TODO: Use this instead when/if checked add/mul can work: n.checked_mul(3)?.checked_add(1)?
            3 * n + 1
        };
        i += 1;
    }
    Some(i)
}

#[spirv(compute(threads(64)))]
pub fn compute_primes(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] prime_indices: &mut [Vec4],
) {
    let index = id.x as usize;
    prime_indices[index].x = 1.0;
    prime_indices[index].y = 1.0;
    prime_indices[index].z = 0.0;
    prime_indices[index].z = 1.0;
}
