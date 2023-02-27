pub static MODULE_METAS: Lazy<RwLock<ShaderMetas>> = Lazy::new(Default::default);

use std::sync::RwLock;

use once_cell::sync::Lazy;

use bevy::{
    prelude::{
        info, AssetEvent, Assets, Deref, DerefMut, EventReader, Handle, Plugin, Res, ResMut,
        Resource, Shader,
    },
    reflect::TypeUuid,
    utils::{HashMap, HashSet},
};
use bevy_common_assets::json::JsonAssetPlugin;

use serde::{Deserialize, Serialize};

use crate::prelude::{RustGpuEntryPoint, RustGpuMaterial};

pub struct RustGpuShaderPlugin;

impl Plugin for RustGpuShaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        if !app.is_plugin_added::<JsonAssetPlugin<ModuleMeta>>() {
            app.add_plugin(JsonAssetPlugin::<ModuleMeta>::new(&["spv.json"]));
        }

        app.init_resource::<ShaderMetaMap>();
    }
}

#[derive(Debug, Default, Clone, Resource)]
pub struct ShaderMetaMap {
    pub shader_to_meta: HashMap<Handle<Shader>, Handle<ModuleMeta>>,
    pub meta_to_shader: HashMap<Handle<ModuleMeta>, Handle<Shader>>,
}

impl ShaderMetaMap {
    pub fn add(&mut self, shader: Handle<Shader>, meta: Handle<ModuleMeta>) {
        self.shader_to_meta.insert(shader.clone(), meta.clone());
        self.meta_to_shader.insert(meta, shader);
    }

    pub fn meta(&self, shader: &Handle<Shader>) -> Option<&Handle<ModuleMeta>> {
        self.shader_to_meta.get(shader)
    }

    pub fn shader(&self, meta: &Handle<ModuleMeta>) -> Option<&Handle<Shader>> {
        self.meta_to_shader.get(meta)
    }

    pub fn remove_by_shader(&mut self, shader: Handle<Shader>) {
        let meta = self.shader_to_meta.remove(&shader).unwrap();
        self.meta_to_shader.remove(&meta).unwrap();
    }

    pub fn remove_by_meta(&mut self, shader: Handle<ModuleMeta>) {
        let shader = self.meta_to_shader.remove(&shader).unwrap();
        self.shader_to_meta.remove(&shader).unwrap();
    }
}

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct ShaderMetas {
    pub metas: HashMap<Handle<Shader>, ModuleMeta>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "64a45932-95c4-42c7-a212-0598949d798f"]
pub struct ModuleMeta {
    pub entry_points: Vec<String>,
    pub module: String,
}

pub fn module_meta_events<V, F>(
    mut shader_events: EventReader<AssetEvent<Shader>>,
    mut module_meta_events: EventReader<AssetEvent<ModuleMeta>>,
    assets: Res<Assets<ModuleMeta>>,
    mut materials: ResMut<Assets<RustGpuMaterial<V, F>>>,
    shader_meta_map: ResMut<ShaderMetaMap>,
) where
    V: RustGpuEntryPoint,
    F: RustGpuEntryPoint,
{
    let mut changed_shaders = HashSet::default();

    for event in shader_events.iter() {
        info!("Shader event {event:#?}");
        match event {
            AssetEvent::Created {
                handle: shader_handle,
            }
            | AssetEvent::Modified {
                handle: shader_handle,
            } => {
                // Remove meta in case the shader and meta load on different frames
                MODULE_METAS.write().unwrap().remove(shader_handle);

                // Mark this shader for material reloading
                changed_shaders.insert(shader_handle);
            }
            _ => (),
        }
    }

    for event in module_meta_events.iter() {
        info!("Module meta event {event:#?}");
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                // If this meta has an associated shader, mark it for material reloading
                if let Some(shader) = shader_meta_map.shader(handle) {
                    changed_shaders.insert(shader);

                    // Update module meta
                    info!("Updating shader meta for {handle:?}");
                    if let Some(asset) = assets.get(handle) {
                        info!("Shader meta: {asset:#?}");
                        MODULE_METAS
                            .write()
                            .unwrap()
                            .insert(shader.clone_weak(), asset.clone());
                    }
                }
            }
            _ => (),
        }
    }

    // Reload all materials with shaders that have changed
    for (_, material) in materials.iter_mut() {
        let mut reload = false;

        if let Some(vertex_shader) = &material.vertex_shader {
            if changed_shaders.contains(vertex_shader) {
                reload = true;
            }
        }

        if let Some(fragment_shader) = &material.fragment_shader {
            if changed_shaders.contains(fragment_shader) {
                reload = true;
            }
        }

        if reload {
            material.iteration += 1;
        }
    }
}
