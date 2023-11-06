use bevy::{prelude::{
    default, shape::Cube, App, AssetPlugin, AssetServer, Assets, Camera3dBundle, Color, Commands,
    DefaultPlugins, DirectionalLight, DirectionalLightBundle, MaterialMeshBundle, Mesh,
    PluginGroup, PointLight, PointLightBundle, Quat, Res, ResMut, StandardMaterial, Transform,
    Vec3,
}, asset::ChangeWatcher};

use bevy_rust_gpu::prelude::{RustGpu, RustGpuMaterialPlugin, RustGpuPlugin};

/// Workspace-relative path to SPIR-V shader
const SHADER_PATH: &'static str = "rust-gpu/shader.rust-gpu.msgpack";

/// Workspace-relative path to entry points file
const ENTRY_POINTS_PATH: &'static str = "crates/viewer/entry_points.json";

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
    app.add_plugin(RustGpuPlugin::default());

    // Setup `RustGpu<StandardMaterial>`
    app.add_plugin(RustGpuMaterialPlugin::<StandardMaterial>::default());
    RustGpu::<StandardMaterial>::export_to(ENTRY_POINTS_PATH);

    // Setup scene
    app.add_startup_system(setup);

    // Run
    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut rust_gpu_standard_materials: ResMut<Assets<RustGpu<StandardMaterial>>>,
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

    // Create mesh, shader and material assets
    let standard_material = standard_materials.add(default());

    let mesh = meshes.add(Cube { size: 1.0 }.into());

    let shader = asset_server.load(SHADER_PATH);

    // Spawn example cubes
    commands.spawn(MaterialMeshBundle {
        transform: Transform::from_xyz(-1.0, 0.0, -6.0)
            .with_rotation(Quat::from_axis_angle(Vec3::new(1.0, 1.0, 1.0), 45.0).normalize()),
        mesh: mesh.clone(),
        material: standard_material,
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        transform: Transform::from_xyz(1.0, 0.0, -6.0)
            .with_rotation(Quat::from_axis_angle(Vec3::new(-1.0, 1.0, 1.0), -45.0).normalize()),
        mesh: mesh.clone(),
        material: rust_gpu_standard_materials.add(RustGpu {
            vertex_shader: Some(shader.clone()),
            fragment_shader: Some(shader.clone()),
            ..default()
        }),
        ..default()
    });
}
