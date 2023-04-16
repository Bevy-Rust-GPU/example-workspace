use std::marker::PhantomData;

use bevy::{
    core_pipeline::prepass::DepthPrepass,
    prelude::{
        default, shape::Cube, AlphaMode, App, AssetPlugin, AssetServer, Assets, Camera3dBundle,
        ClearColor, Color, Commands, Component, DefaultPlugins, DirectionalLight,
        DirectionalLightBundle, Material, MaterialMeshBundle, Mesh, Msaa, PluginGroup, PointLight,
        PointLightBundle, Quat, Query, Res, ResMut, Transform, Vec3, Vec4, With,
    },
    reflect::TypeUuid,
    render::render_resource::AsBindGroup,
    time::Time,
    utils::Uuid,
};

use bevy_rust_gpu::{
    prelude::{RustGpu, RustGpuMaterialPlugin, RustGpuPlugin},
    EntryPoint, EntryPointTypes, RustGpuBuilderOutput, RustGpuMaterial,
};
use rust_gpu_bridge::Named;
use rust_gpu_sdf::{
    prelude::{
        AttrColor, AttrDistance, AttrNormal, AttrTangent, AttrUv, Car, Cdr, Checker, ColorNormal,
        ColorTangent, ColorUv, Cube as SdfCube, Distance, EuclideanMetric, Field,
        Isosurface, IsosurfaceOp, Normal, Octahedron, Position,
        ProxyColor, Raycast, RaycastInput, ScaleUv, Sphere, SphereTraceLipschitz, Tangent, Torus,
        Uv, UvTangent, White, FieldOperator,
    },
    type_fields::hlist::tuple::{Cons, ConsRef},
};

/// Workspace-relative path to SPIR-V shader
const SHADER_PATH: &'static str = "rust-gpu/shader.rust-gpu.msgpack";

const ENTRY_POINTS_PATH: &'static str = "crates/viewer/entry_points.json";

/// Marker type describing the `vertex_warp` entrypoint from the shader crate
pub enum VertexSdf3d {}

impl EntryPoint for VertexSdf3d {
    const NAME: &'static str = "vertex_sdf_3d";
}

/// Marker type describing the `fragment_normal` entrypoint from the shader crate
pub struct FragmentSdf3d<T> {
    pub _phantom: PhantomData<T>,
}

impl<T> EntryPoint for FragmentSdf3d<T>
where
    T: Named
        + Field<AttrDistance<Vec3>>
        + Field<AttrNormal<Vec3>>
        + Field<AttrTangent<Vec3>>
        + Field<AttrUv<Vec3>>
        + Field<AttrColor<Vec3>>
        + Field<Raycast>
        + Clone
        + Send
        + Sync
        + 'static,
{
    const NAME: &'static str = "fragment_sdf_3d";

    fn types() -> EntryPointTypes {
        vec![("Sdf".to_string(), T::name())]
    }
}

/// Example RustGpu material tying together [`VertexWarp`] and [`FragmentNormal`]
#[derive(Debug, Default, Copy, Clone, AsBindGroup)]
pub struct Sdf3dMaterial<T> {
    pub sdf: T,
    pub alpha_mode: AlphaMode,
}

impl<T> TypeUuid for Sdf3dMaterial<T> {
    const TYPE_UUID: Uuid = Uuid::from_u128(5467237301083018133);
}

impl<T> Material for Sdf3dMaterial<T>
where
    T: Named
        + Field<AttrDistance<Vec3>>
        + Field<AttrNormal<Vec3>>
        + Field<AttrTangent<Vec3>>
        + Field<AttrUv<Vec3>>
        + Field<AttrColor<Vec3>>
        + Field<Raycast>
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

impl<T> RustGpuMaterial for Sdf3dMaterial<T>
where
    T: Named
        + Field<AttrDistance<Vec3>>
        + Field<AttrNormal<Vec3>>
        + Field<AttrTangent<Vec3>>
        + Field<AttrUv<Vec3>>
        + Field<AttrColor<Vec3>>
        + Field<Raycast>
        + Clone
        + Send
        + Sync
        + 'static,
{
    type Vertex = VertexSdf3d;
    type Fragment = FragmentSdf3d<T>;
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Rotate;

pub type Sdf = SphereTraceLipschitz<400, ScaleUv<ColorUv<UvTangent<Sphere>>>>;

fn main() {
    let mut app = App::default();

    // Add default plugins
    app.add_plugins(DefaultPlugins.set(
        // Configure the asset plugin to watch the workspace path for changes
        AssetPlugin {
            watch_for_changes: true,
            ..default()
        },
    ));

    // Add the Rust-GPU plugin
    app.add_plugin(RustGpuPlugin::default());

    // Setup `RustGpu<ExampleMaterial>`
    app.add_plugin(RustGpuMaterialPlugin::<Sdf3dMaterial<Sdf>>::default());
    RustGpu::<Sdf3dMaterial<Sdf>>::export_to(ENTRY_POINTS_PATH);

    // Set clear color to black
    app.insert_resource(ClearColor(Color::BLACK));

    app.insert_resource(Msaa::Off);

    // Setup scene
    app.add_startup_system(setup);

    app.add_system(
        |time: Res<Time>, mut query: Query<&mut Transform, With<Rotate>>| {
            for mut transform in query.iter_mut() {
                transform.rotation = (transform.rotation
                    * Quat::from_axis_angle(Vec3::ONE, time.delta_seconds()))
                .normalize()
            }
        },
    );

    // Run
    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_3d_materials: ResMut<Assets<RustGpu<Sdf3dMaterial<Sdf>>>>,
) {
    // Spawn camera
    commands.spawn((Camera3dBundle::default(), DepthPrepass::default()));

    // Spawn lights
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            ..default()
        },
        transform: Transform::IDENTITY.looking_at(Vec3::new(0.0, -1.0, -1.0), Vec3::Y),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 400.0,
            range: 4.0,
            color: Color::BLUE,
            ..default()
        },
        transform: Transform::from_xyz(0.0, -2.0, -4.0),
        ..default()
    });

    // Load mesh and shader
    let mesh = meshes.add(Cube { size: 4.0 }.into());

    let shader = asset_server.load::<RustGpuBuilderOutput, _>(SHADER_PATH);

    // Create material
    let material = sdf_3d_materials.add(RustGpu {
        vertex_shader: Some(shader.clone()),
        fragment_shader: Some(shader),
        base: Sdf3dMaterial {
            sdf: default(),
            ..default()
        },
        ..default()
    });

    let sdf = Sphere::default();
    <IsosurfaceOp as FieldOperator<EuclideanMetric, AttrDistance<Vec3>>>::operator(
        &sdf.op,
        &sdf.target,
        &Position(Vec3::ZERO),
    );
    <IsosurfaceOp as FieldOperator<EuclideanMetric, AttrNormal<Vec3>>>::operator(
        &sdf.op,
        &sdf.target,
        &Position(Vec3::ZERO),
    );

    // Spawn example cubes
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform::from_xyz(0.0, 0.0, -4.0).with_rotation(
                Quat::from_axis_angle(Vec3::new(-1.0, 1.0, 1.0), std::f32::consts::FRAC_PI_4)
                    .normalize(),
            ),
            mesh,
            material,
            ..default()
        },
        Rotate,
    ));
}
