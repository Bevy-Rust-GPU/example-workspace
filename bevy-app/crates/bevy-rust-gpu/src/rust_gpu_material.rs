use std::marker::PhantomData;

use bevy::{
    pbr::StandardMaterialUniform,
    prelude::{default, info, warn, Handle, Image, Material, Shader, StandardMaterial},
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, AsBindGroupShaderType, Face, ShaderType},
    utils::Uuid,
};

use crate::{
    prelude::{MissingEntryPoint, ModuleMeta, MODULE_METAS},
    rust_gpu_entry_point::RustGpuEntryPoint,
    rust_gpu_missing_entry_points::MissingEntryPointSender,
};

#[derive(ShaderType)]
pub struct BaseMaterial {
    base: StandardMaterialUniform,
}

#[derive(Debug, Default, Clone)]
pub struct ShaderMaterialKey {
    vertex_shader: Option<Handle<Shader>>,
    vertex_meta: Option<Handle<ModuleMeta>>,
    vertex_defs: Vec<String>,
    fragment_shader: Option<Handle<Shader>>,
    fragment_meta: Option<Handle<ModuleMeta>>,
    fragment_defs: Vec<String>,
    normal_map: bool,
    cull_mode: Option<Face>,
    sender: Option<MissingEntryPointSender>,
}

impl PartialEq for ShaderMaterialKey {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_shader.eq(&other.vertex_shader)
            && self.vertex_meta.eq(&other.vertex_meta)
            && self.vertex_defs.eq(&other.vertex_defs)
            && self.fragment_shader.eq(&other.fragment_shader)
            && self.fragment_meta.eq(&other.fragment_meta)
            && self.fragment_defs.eq(&other.fragment_defs)
            && self.normal_map.eq(&other.normal_map)
            && self.cull_mode.eq(&other.cull_mode)
    }
}

impl std::hash::Hash for ShaderMaterialKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vertex_shader.hash(state);
        self.vertex_meta.hash(state);
        self.vertex_defs.hash(state);
        self.fragment_shader.hash(state);
        self.fragment_meta.hash(state);
        self.fragment_defs.hash(state);
        self.normal_map.hash(state);
        self.cull_mode.hash(state);
    }
}

impl Eq for ShaderMaterialKey {}

impl<V, F> From<&RustGpuMaterial<V, F>> for ShaderMaterialKey {
    fn from(value: &RustGpuMaterial<V, F>) -> Self {
        ShaderMaterialKey {
            vertex_shader: value.vertex_shader.clone(),
            vertex_meta: value.vertex_meta.clone(),
            vertex_defs: value.vertex_defs.clone(),
            fragment_shader: value.fragment_shader.clone(),
            fragment_meta: value.fragment_meta.clone(),
            fragment_defs: value.fragment_defs.clone(),
            normal_map: value.normal_map_texture.is_some(),
            cull_mode: value.base.cull_mode,
            sender: value.sender.clone(),
        }
    }
}

#[derive(Debug, AsBindGroup)]
#[bind_group_data(ShaderMaterialKey)]
#[uniform(0, BaseMaterial)]
pub struct RustGpuMaterial<V, F> {
    pub base: StandardMaterial,

    pub vertex_shader: Option<Handle<Shader>>,
    pub vertex_meta: Option<Handle<ModuleMeta>>,
    pub vertex_defs: Vec<String>,
    pub fragment_shader: Option<Handle<Shader>>,
    pub fragment_meta: Option<Handle<ModuleMeta>>,
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

    pub sender: Option<MissingEntryPointSender>,
    pub _phantom: PhantomData<(V, F)>,
}

impl<V, F> Default for RustGpuMaterial<V, F> {
    fn default() -> Self {
        RustGpuMaterial {
            base: default(),
            vertex_shader: default(),
            vertex_meta: default(),
            vertex_defs: default(),
            fragment_shader: default(),
            fragment_meta: default(),
            fragment_defs: default(),
            base_color_texture: default(),
            emissive_texture: default(),
            metallic_roughness_texture: default(),
            occlusion_texture: default(),
            normal_map_texture: default(),
            sender: default(),
            _phantom: default(),
        }
    }
}

impl<V, F> Clone for RustGpuMaterial<V, F> {
    fn clone(&self) -> Self {
        RustGpuMaterial {
            base: self.base.clone(),
            vertex_shader: self.vertex_shader.clone(),
            vertex_meta: self.vertex_meta.clone(),
            vertex_defs: self.vertex_defs.clone(),
            fragment_shader: self.fragment_shader.clone(),
            fragment_meta: self.fragment_meta.clone(),
            fragment_defs: self.fragment_defs.clone(),
            base_color_texture: self.base_color_texture.clone(),
            emissive_texture: self.emissive_texture.clone(),
            metallic_roughness_texture: self.metallic_roughness_texture.clone(),
            occlusion_texture: self.occlusion_texture.clone(),
            normal_map_texture: self.occlusion_texture.clone(),
            sender: self.sender.clone(),
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
        info!("Specializing RustGpuMaterial");
        if let Some(vertex_shader) = key.bind_group_data.vertex_shader {
            let shader_defs: Vec<_> = descriptor
                .vertex
                .shader_defs
                .iter()
                .cloned()
                .chain(key.bind_group_data.vertex_defs.iter().cloned())
                .collect();

            let entry_point = V::build(&shader_defs);

            let mut apply = true;

            if let Some(vertex_meta) = key.bind_group_data.vertex_meta {
                let metas = MODULE_METAS.read().unwrap();
                if let Some(vertex_meta) = metas.get(&vertex_meta) {
                    if !vertex_meta.entry_points.contains(&entry_point) {
                        warn!("Missing entry point {entry_point:}");
                        warn!("Falling back to default vertex shader.");
                        apply = false;
                    }
                }
            }

            if let Some(sender) = &key.bind_group_data.sender {
                sender
                    .send(MissingEntryPoint {
                        shader: V::NAME,
                        permutation: V::permutation(&shader_defs),
                    })
                    .unwrap();
            }

            if apply {
                descriptor.vertex.shader = vertex_shader;
                descriptor.vertex.entry_point = entry_point.into();
            }
        }

        if let Some(fragment_descriptor) = descriptor.fragment.as_mut() {
            if key.bind_group_data.normal_map {
                fragment_descriptor
                    .shader_defs
                    .push(String::from("STANDARDMATERIAL_NORMAL_MAP"));
            }

            if let Some(fragment_shader) = key.bind_group_data.fragment_shader {
                let shader_defs: Vec<_> = fragment_descriptor
                    .shader_defs
                    .iter()
                    .cloned()
                    .chain(key.bind_group_data.fragment_defs.iter().cloned())
                    .collect();

                let entry_point = F::build(&shader_defs).into();

                let mut apply = true;

                if let Some(fragment_meta) = key.bind_group_data.fragment_meta {
                    let metas = MODULE_METAS.read().unwrap();
                    if let Some(fragment_meta) = metas.get(&fragment_meta) {
                        if !fragment_meta.entry_points.contains(&entry_point) {
                            warn!("Missing entry point {entry_point:}, falling back to default fragment shader.");
                            warn!("Falling back to default fragment shader.");
                            apply = false;
                        }
                    }
                }

                if let Some(sender) = &key.bind_group_data.sender {
                    sender
                        .send(MissingEntryPoint {
                            shader: F::NAME,
                            permutation: F::permutation(&shader_defs),
                        })
                        .unwrap();
                }

                if apply {
                    fragment_descriptor.shader = fragment_shader;
                    fragment_descriptor.entry_point = entry_point.into();
                }
            }
        }

        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;

        if let Some(label) = &mut descriptor.label {
            *label = format!("shader_{}", *label).into();
        }

        Ok(())
    }
}
