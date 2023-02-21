use std::borrow::Cow;

use bevy::{
    pbr::StandardMaterialUniform,
    prelude::{Handle, Material, Shader, StandardMaterial},
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, AsBindGroupShaderType, Face, ShaderType},
};

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
    pub vertex_shader: Option<Handle<Shader>>,
    pub vertex_entry_point: Option<Cow<'static, str>>,
    pub fragment_shader: Option<Handle<Shader>>,
    pub fragment_entry_point: Option<Cow<'static, str>>,
    pub base: StandardMaterial,
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
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        if let Some(vertex_shader) = key.bind_group_data.vertex_shader {
            descriptor.vertex.shader = vertex_shader;
        }

        if let Some(vertex_entry_point) = key.bind_group_data.vertex_entry_point {
            descriptor.vertex.entry_point = vertex_entry_point;
        }

        if let Some(fragment_descriptor) = descriptor.fragment.as_mut() {
            if let Some(fragment_shader) = key.bind_group_data.fragment_shader {
                fragment_descriptor.shader = fragment_shader;
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
