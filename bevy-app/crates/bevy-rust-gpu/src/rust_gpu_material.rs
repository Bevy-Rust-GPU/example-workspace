use std::marker::PhantomData;

use bevy::{
    pbr::StandardMaterialUniform,
    prelude::{default, Handle, Image, Material, Shader, StandardMaterial},
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, AsBindGroupShaderType, Face, ShaderType},
    utils::Uuid,
};

use crate::rust_gpu_entry_point::RustGpuEntryPoint;

#[derive(ShaderType)]
pub struct BaseMaterial {
    base: StandardMaterialUniform,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ShaderMaterialKey {
    vertex_shader: Option<Handle<Shader>>,
    vertex_defs: Vec<String>,
    fragment_shader: Option<Handle<Shader>>,
    fragment_defs: Vec<String>,
    normal_map: bool,
    cull_mode: Option<Face>,
}

impl<V, F> From<&RustGpuMaterial<V, F>> for ShaderMaterialKey {
    fn from(value: &RustGpuMaterial<V, F>) -> Self {
        ShaderMaterialKey {
            vertex_shader: value.vertex_shader.clone(),
            vertex_defs: value.vertex_defs.clone(),
            fragment_shader: value.fragment_shader.clone(),
            fragment_defs: value.fragment_defs.clone(),
            normal_map: value.normal_map_texture.is_some(),
            cull_mode: value.base.cull_mode,
        }
    }
}

#[derive(Debug, AsBindGroup)]
#[bind_group_data(ShaderMaterialKey)]
#[uniform(0, BaseMaterial)]
pub struct RustGpuMaterial<V, F> {
    pub base: StandardMaterial,

    pub vertex_shader: Option<Handle<Shader>>,
    pub vertex_defs: Vec<String>,
    pub fragment_shader: Option<Handle<Shader>>,
    pub fragment_defs: Vec<String>,

    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,

    #[texture(3)]
    #[sampler(4)]
    pub emissive_texture: Option<Handle<Image>>,

    #[texture(5)]
    #[sampler(6)]
    pub metallic_roughness_texture: Option<Handle<Image>>,

    #[texture(7)]
    #[sampler(8)]
    pub occlusion_texture: Option<Handle<Image>>,

    #[texture(9)]
    #[sampler(10)]
    pub normal_map_texture: Option<Handle<Image>>,

    pub _phantom: PhantomData<(V, F)>,
}

impl<V, F> Default for RustGpuMaterial<V, F> {
    fn default() -> Self {
        RustGpuMaterial {
            base: default(),
            vertex_shader: default(),
            vertex_defs: default(),
            fragment_shader: default(),
            fragment_defs: default(),
            base_color_texture: default(),
            emissive_texture: default(),
            metallic_roughness_texture: default(),
            occlusion_texture: default(),
            normal_map_texture: default(),
            _phantom: default(),
        }
    }
}

impl<V, F> Clone for RustGpuMaterial<V, F> {
    fn clone(&self) -> Self {
        RustGpuMaterial {
            base: self.base.clone(),
            vertex_shader: self.vertex_shader.clone(),
            vertex_defs: self.vertex_defs.clone(),
            fragment_shader: self.fragment_shader.clone(),
            fragment_defs: self.fragment_defs.clone(),
            base_color_texture: self.base_color_texture.clone(),
            emissive_texture: self.emissive_texture.clone(),
            metallic_roughness_texture: self.metallic_roughness_texture.clone(),
            occlusion_texture: self.occlusion_texture.clone(),
            normal_map_texture: self.occlusion_texture.clone(),
            _phantom: default(),
        }
    }
}

impl<V, F> TypeUuid for RustGpuMaterial<V, F> {
    const TYPE_UUID: bevy::utils::Uuid = Uuid::from_fields(
        0x3bb0b1c8,
        0x5ff8,
        0x4085,
        &[0xa4, 0x48, 0x19, 0xda, 0xa3, 0x36, 0xc1, 0x0c],
    );
}

impl<V, F> AsBindGroupShaderType<BaseMaterial> for RustGpuMaterial<V, F> {
    fn as_bind_group_shader_type(
        &self,
        images: &bevy::render::render_asset::RenderAssets<bevy::prelude::Image>,
    ) -> BaseMaterial {
        BaseMaterial {
            base: self.base.as_bind_group_shader_type(images),
        }
    }
}

impl<V, F> Material for RustGpuMaterial<V, F>
where
    V: RustGpuEntryPoint,
    F: RustGpuEntryPoint,
    RustGpuMaterial<V, F>: AsBindGroup<Data = ShaderMaterialKey>,
{
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

            let shader_defs: Vec<_> = descriptor
                .vertex
                .shader_defs
                .iter()
                .cloned()
                .chain(key.bind_group_data.vertex_defs.iter().cloned())
                .collect();

            descriptor.vertex.entry_point = V::build(&shader_defs).into();
        }

        if let Some(fragment_descriptor) = descriptor.fragment.as_mut() {
            if key.bind_group_data.normal_map {
                fragment_descriptor
                    .shader_defs
                    .push(String::from("STANDARDMATERIAL_NORMAL_MAP"));
            }

            if let Some(fragment_shader) = key.bind_group_data.fragment_shader {
                fragment_descriptor.shader = fragment_shader;

                let shader_defs: Vec<_> = fragment_descriptor
                    .shader_defs
                    .iter()
                    .cloned()
                    .chain(key.bind_group_data.fragment_defs.iter().cloned())
                    .collect();

                fragment_descriptor.entry_point = F::build(&shader_defs).into();
            }
        }

        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;

        if let Some(label) = &mut descriptor.label {
            *label = format!("shader_{}", *label).into();
        }

        Ok(())
    }
}
