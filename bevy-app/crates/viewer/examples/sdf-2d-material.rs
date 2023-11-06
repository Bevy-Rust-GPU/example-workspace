use bevy::{
    core_pipeline::prepass::DepthPrepass,
    prelude::{
        default,
        shape::{Cube, Quad},
        App, AssetPlugin, AssetServer, Assets, Camera2dBundle, ClearColor, Color, Commands,
        Component, DefaultPlugins, DirectionalLight, DirectionalLightBundle, Material,
        MaterialMeshBundle, Mesh, Msaa, PluginGroup, PointLight, PointLightBundle, Quat, Query,
        Res, ResMut, Transform, Vec2, Vec3, With,
    },
    reflect::{TypeUuid, Reflect},
    render::render_resource::AsBindGroup,
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
    time::Time, asset::ChangeWatcher,
};

use bevy_rust_gpu::{
    prelude::{RustGpu, RustGpuMaterialPlugin, RustGpuPlugin},
    EntryPoint, RustGpuBuilderOutput, RustGpuMaterial, RustGpuMaterial2dPlugin,
};

/// Workspace-relative path to SPIR-V shader
const SHADER_PATH: &'static str = "rust-gpu/shader.rust-gpu.msgpack";

const ENTRY_POINTS_PATH: &'static str = "crates/viewer/entry_points.json";

/// Marker type describing the `vertex_warp` entrypoint from the shader crate
pub enum VertexWarp {}

impl EntryPoint for VertexWarp {
    const NAME: &'static str = "vertex_sdf_2d";
}

/// Marker type describing the `fragment_normal` entrypoint from the shader crate
pub enum FragmentNormal {}

impl EntryPoint for FragmentNormal {
    const NAME: &'static str = "fragment_sdf_2d";
}

/// Example RustGpu material tying together [`VertexWarp`] and [`FragmentNormal`]
#[derive(Debug, Default, Copy, Clone, AsBindGroup, TypeUuid, Reflect)]
#[uuid = "cbeff76a-27e9-42c8-bb17-73e81ba62a36"]
pub struct ExampleMaterial {}

impl Material2d for ExampleMaterial {
    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

impl RustGpuMaterial for ExampleMaterial {
    type Vertex = VertexWarp;
    type Fragment = FragmentNormal;
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct Rotate;

fn main() {
    let mut app = App::default();

    // Add default plugins
    app.add_plugins(DefaultPlugins.set(
        // Configure the asset plugin to watch the workspace path for changes
        AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(std::time::Duration::from_secs(1)),
            ..default()
        },
    ));

    // Add the Rust-GPU plugin
    app.add_plugins(RustGpuPlugin::default());

    // Setup `RustGpu<ExampleMaterial>`
    app.add_plugins(RustGpuMaterial2dPlugin::<ExampleMaterial>::default());
    RustGpu::<ExampleMaterial>::export_to(ENTRY_POINTS_PATH);

    // Set clear color to black
    app.insert_resource(ClearColor(Color::BLACK));

    app.insert_resource(Msaa::Off);

    // Setup scene
    app.add_startup_system(setup);

    // Run
    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut example_materials: ResMut<Assets<RustGpu<ExampleMaterial>>>,
) {
    // Spawn camera
    commands.spawn((Camera2dBundle::default(), DepthPrepass::default()));

    // Load mesh and shader
    let mesh = meshes.add(
        Quad {
            size: Vec2::ONE * 200.0,
            flip: false,
        }
        .into(),
    );

    let mesh = Mesh2dHandle::from(mesh);

    let shader = asset_server.load::<RustGpuBuilderOutput, _>(SHADER_PATH);

    // Create material
    let material = example_materials.add(RustGpu {
        vertex_shader: Some(shader.clone()),
        fragment_shader: Some(shader),
        ..default()
    });

    // Spawn example cubes
    commands.spawn((
        MaterialMesh2dBundle {
            mesh,
            material,
            ..default()
        },
        Rotate,
    ));
}
