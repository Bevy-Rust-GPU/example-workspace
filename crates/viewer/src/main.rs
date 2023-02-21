use std::borrow::Cow;

use bevy::{
    pbr::StandardMaterialUniform,
    prelude::{
        default, info, shape::Cube, App, AssetPlugin, AssetServer, Assets, Camera3dBundle,
        Commands, DefaultPlugins, Handle, Material, MaterialMeshBundle, MaterialPlugin, Mesh,
        PluginGroup, PointLightBundle, Quat, Res, ResMut, Shader, StandardMaterial, Transform,
        Vec3,
    },
    reflect::TypeUuid,
    render::{
        render_resource::{AsBindGroup, AsBindGroupShaderType, Face, ShaderType},
        settings::{WgpuLimits, WgpuSettings},
    },
};

fn main() {
    let mut app = App::default();

    app.insert_resource(WgpuSettings {
        constrained_limits: Some(WgpuLimits {
            max_storage_buffers_per_shader_stage: 0,
            ..default()
        }),
        ..default()
    });

    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        asset_folder: "/mnt/projects/personal/rust/Projects/rust-gpu-test".into(),
        watch_for_changes: true,
        ..default()
    }))
    .add_plugin(MaterialPlugin::<ShaderMaterial>::default());

    app.add_startup_system(setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shader_materials: ResMut<Assets<ShaderMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle::default());

    //commands.spawn(DirectionalLightBundle::default());

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, -6.0),
        ..default()
    });

    let mesh = meshes.add(Cube { size: 1.0 }.into());

    let shader = asset_server
        .load::<Shader, _>("target/spirv-builder/spirv-unknown-spv1.5/release/deps/shader.spv");
    let shader_material = shader_materials.add(ShaderMaterial {
        vertex_shader: Some(shader.clone()),
        vertex_entry_point: Some("bevy_pbr::mesh::vertex".into()),
        fragment_shader: Some(shader),
        fragment_entry_point: Some("bevy_pbr::pbr::fragment".into()),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        transform: Transform::from_xyz(-1.0, 0.0, -6.0)
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, 45.0).normalize()),
        mesh: mesh.clone(),
        material: standard_materials.add(default()),
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

#[derive(ShaderType)]
pub struct BaseMaterial {
    base: StandardMaterialUniform,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ShaderMaterialKey {
    vertex_shader: Option<Handle<Shader>>,
    vertex_entry_point: Option<Cow<'static, str>>,
    fragment_shader: Option<Handle<Shader>>,
    fragment_entry_point: Option<Cow<'static, str>>,
    cull_mode: Option<Face>,
}

impl From<&ShaderMaterial> for ShaderMaterialKey {
    fn from(value: &ShaderMaterial) -> Self {
        ShaderMaterialKey {
            vertex_shader: value.vertex_shader.clone(),
            vertex_entry_point: value.vertex_entry_point.clone(),
            fragment_shader: value.fragment_shader.clone(),
            fragment_entry_point: value.fragment_entry_point.clone(),
            cull_mode: value.base.cull_mode,
        }
    }
}

#[derive(Debug, Default, Clone, AsBindGroup, TypeUuid)]
#[uuid = "3bb0b1c8-5ff8-4085-a448-19daa336c10c"]
#[bind_group_data(ShaderMaterialKey)]
#[uniform(0, BaseMaterial)]
pub struct ShaderMaterial {
    vertex_shader: Option<Handle<Shader>>,
    vertex_entry_point: Option<Cow<'static, str>>,
    fragment_shader: Option<Handle<Shader>>,
    fragment_entry_point: Option<Cow<'static, str>>,
    base: StandardMaterial,
}

impl AsBindGroupShaderType<BaseMaterial> for ShaderMaterial {
    fn as_bind_group_shader_type(
        &self,
        images: &bevy::render::render_asset::RenderAssets<bevy::prelude::Image>,
    ) -> BaseMaterial {
        BaseMaterial {
            base: self.base.as_bind_group_shader_type(images),
        }
    }
}

impl Material for ShaderMaterial {
    fn alpha_mode(&self) -> bevy::prelude::AlphaMode {
        self.base.alpha_mode
    }

    fn depth_bias(&self) -> f32 {
        self.base.depth_bias
    }

    fn specialize(
        pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        if let Some(vertex_shader) = key.bind_group_data.vertex_shader {
            descriptor.vertex.shader = vertex_shader;
            info!(
                "Clearing vertex shader defs {:#?}",
                descriptor.vertex.shader_defs
            );
            descriptor.vertex.shader_defs.clear();
        }

        if let Some(vertex_entry_point) = key.bind_group_data.vertex_entry_point {
            descriptor.vertex.entry_point = vertex_entry_point;
        }

        if let Some(fragment_descriptor) = descriptor.fragment.as_mut() {
            if let Some(fragment_shader) = key.bind_group_data.fragment_shader {
                fragment_descriptor.shader = fragment_shader;
                info!(
                    "Clearing fragment shader defs {:#?}",
                    fragment_descriptor.shader_defs
                );
                fragment_descriptor.shader_defs.clear();
            }

            if let Some(fragment_entry_point) = key.bind_group_data.fragment_entry_point {
                fragment_descriptor.entry_point = fragment_entry_point;
            }
        }

        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;

        if let Some(label) = &mut descriptor.label {
            *label = format!("shader_{}", *label).into();
        }

        Ok(())
    }
}
