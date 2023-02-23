pub mod rust_gpu_entry_point;
pub mod rust_gpu_material;
pub mod rust_gpu_shaders;

use bevy::{
    prelude::{
        default, shape::Cube, App, AssetPlugin, AssetServer, Assets, Camera3dBundle, Color,
        Commands, DefaultPlugins, DirectionalLight, DirectionalLightBundle, MaterialMeshBundle,
        MaterialPlugin, Mesh, PluginGroup, PointLight, PointLightBundle, Quat, Res, ResMut, Shader,
        StandardMaterial, Transform, Vec3,
    },
    render::settings::{WgpuLimits, WgpuSettings},
};
use rust_gpu_entry_point::rust_gpu_shader_defs;
use rust_gpu_material::RustGpuMaterial;
use rust_gpu_shaders::{MeshVertex, PbrFragment};

fn main() {
    let mut app = App::default();

    // Force the app not to use storage buffers
    app.insert_resource(WgpuSettings {
        constrained_limits: Some(WgpuLimits {
            max_storage_buffers_per_shader_stage: 0,
            ..default()
        }),
        ..default()
    });

    // Configure the asset plugin to watch the workspace path for changes
    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        asset_folder: "../../../".into(),
        watch_for_changes: true,
        ..default()
    }));

    // Setup ShaderMaterial
    app.add_plugin(MaterialPlugin::<RustGpuMaterial<MeshVertex, PbrFragment>>::default());

    // Setup scene
    app.add_startup_system(setup);

    // Run
    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shader_materials: ResMut<Assets<RustGpuMaterial<MeshVertex, PbrFragment>>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn camera
    commands.spawn(Camera3dBundle::default());

    // Spawn lights
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            ..default()
        },
        transform: Transform::IDENTITY.looking_at(Vec3::new(1.0, -1.0, -1.0), Vec3::Y),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 800.0,
            color: Color::BLUE,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, -6.0),
        ..default()
    });

    // Create mesh, shader and material assets
    let standard_material = standard_materials.add(default());

    let mesh = meshes.add(Cube { size: 1.0 }.into());

    let shader = asset_server.load::<Shader, _>(
        "rust-gpu/target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv",
    );

    let extra_defs = rust_gpu_shader_defs();
    let shader_material = shader_materials.add(RustGpuMaterial {
        vertex_shader: Some(shader.clone()),
        vertex_defs: extra_defs.clone(),
        fragment_shader: Some(shader),
        fragment_defs: extra_defs,
        ..default()
    });

    // Spawn example cubes
    commands.spawn(MaterialMeshBundle {
        transform: Transform::from_xyz(-1.0, 0.0, -6.0)
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, 45.0).normalize()),
        mesh: mesh.clone(),
        material: standard_material,
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        transform: Transform::from_xyz(1.0, 0.0, -6.0)
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -45.0).normalize()),
        mesh,
        material: shader_material,
        ..default()
    });
}
