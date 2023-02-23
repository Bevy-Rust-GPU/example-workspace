#![no_std]

use shader_util::glam;

use self::glam::Vec3;

pub mod clustered_forward;
pub mod mesh;
pub mod mesh_view;
pub mod pbr;
pub mod prelude;
pub mod shadows;
pub mod skinning;
pub mod tonemapping_shared;

macro_rules ! entrypoint_permutations {
    ($stage:ident, $ident:ident, ($($ins:tt)*), { $($body:tt)* }) => {
        #[spirv_std::spirv($stage)]
        pub fn $ident($($ins)*) {
            $($body)*
        }
    };
}

entrypoint_permutations!(fragment, foo, (in_pos: Vec3, out_pos: &mut Vec3), {
    *out_pos = in_pos;
});

bitflags::bitflags! {
    pub struct Flags: usize {
        const A = 1;
        const B = 1 << 1;
        const C = 1 << 2;
        const D = 1 << 3;
    }
}

pub fn op_a() {}
pub fn op_b() {}
pub fn op_c() {}
pub fn op_d() {}

pub trait Op {
    fn op();
}

pub enum A {}
impl Op for A {
    fn op() {
        op_a();
    }
}

pub enum B {}
impl Op for B {
    fn op() {
        op_b();
    }
}

pub enum C {}
impl Op for C {
    fn op() {
        op_c();
    }
}

pub enum D {}
impl Op for D {
    fn op() {
        op_d();
    }
}

pub fn generic_const_usize<const N: usize>() {
    let flags = Flags::from_bits(N).unwrap();
    if flags.contains(Flags::A) {
        op_a();
    }

    if flags.contains(Flags::B) {
        op_b();
    }

    if flags.contains(Flags::C) {
        op_c();
    }

    if flags.contains(Flags::D) {
        op_d();
    }
}

pub fn generic_trait<OP: Op>() {
    OP::op();
}

pub fn non_generic() {
    generic_const_usize::<{ Flags::A.bits() | Flags::B.bits() }>();
}
