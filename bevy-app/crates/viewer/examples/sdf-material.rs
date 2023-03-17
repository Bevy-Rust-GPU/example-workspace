use bevy::{
    prelude::{
        default, shape::Cube, App, AssetPlugin, AssetServer, Assets, Camera3dBundle, ClearColor,
        Color, Commands, Component, DefaultPlugins, DirectionalLight, DirectionalLightBundle,
        Material, MaterialMeshBundle, Mesh, PluginGroup, PointLight, PointLightBundle, Quat, Query,
        Res, ResMut, Transform, Vec3, With,
    },
    reflect::TypeUuid,
    render::{render_resource::AsBindGroup, RenderPlugin},
    time::Time,
};

use bevy_rust_gpu::{
    prelude::{LoadRustGpuShader, RustGpu, RustGpuMaterialPlugin, RustGpuPlugin},
    EntryPoint, RustGpuMaterial,
};

/// Workspace-relative path to SPIR-V shader
const SHADER_PATH: &'static str = "rust-gpu/target/spirv-unknown-spv1.5/release/deps/shader.spv";

const ENTRY_POINTS_PATH: &'static str = "crates/viewer/entry_points.json";

/// Marker type describing the `vertex_warp` entrypoint from the shader crate
pub enum VertexWarp {}

impl EntryPoint for VertexWarp {
    const NAME: &'static str = "vertex_sdf";
    const PARAMETERS: bevy_rust_gpu::EntryPointParameters = &[];
    const CONSTANTS: bevy_rust_gpu::EntryPointConstants = &[];
}

/// Marker type describing the `fragment_normal` entrypoint from the shader crate
pub enum FragmentNormal {}

impl EntryPoint for FragmentNormal {
    const NAME: &'static str = "fragment_sdf";
    const PARAMETERS: bevy_rust_gpu::EntryPointParameters = &[];
    const CONSTANTS: bevy_rust_gpu::EntryPointConstants = &[];
}

/// Example RustGpu material tying together [`VertexWarp`] and [`FragmentNormal`]
#[derive(Debug, Default, Copy, Clone, AsBindGroup, TypeUuid)]
#[uuid = "cbeff76a-27e9-42c8-bb17-73e81ba62a36"]
pub struct ExampleMaterial {}

impl Material for ExampleMaterial {
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
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
    app.add_plugins(
        DefaultPlugins
            .set(
                // Configure the asset plugin to watch the workspace path for changes
                AssetPlugin {
                    asset_folder: "../../../".into(),
                    watch_for_changes: true,
                    ..default()
                },
            )
            .set(
                // Configure the render plugin with RustGpuPlugin's recommended WgpuSettings
                RenderPlugin {
                    wgpu_settings: RustGpuPlugin::wgpu_settings(),
                },
            ),
    );

    // Add the Rust-GPU plugin
    app.add_plugin(RustGpuPlugin);

    // Setup `RustGpu<ExampleMaterial>`
    app.add_plugin(RustGpuMaterialPlugin::<ExampleMaterial>::default());
    RustGpu::<ExampleMaterial>::export_to(ENTRY_POINTS_PATH);

    // Set clear color to black
    app.insert_resource(ClearColor(Color::BLACK));

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
    mut example_materials: ResMut<Assets<RustGpu<ExampleMaterial>>>,
) {
    // Spawn camera
    commands.spawn(Camera3dBundle::default());

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
    let shader = asset_server.load_rust_gpu_shader(SHADER_PATH);

    // Create material
    let material = example_materials.add(RustGpu {
        vertex_shader: Some(shader.clone()),
        fragment_shader: Some(shader),
        ..default()
    });

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
