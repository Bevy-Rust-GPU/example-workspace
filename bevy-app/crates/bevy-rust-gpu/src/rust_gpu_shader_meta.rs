pub static MODULE_METAS: Lazy<RwLock<ModuleMetas>> = Lazy::new(Default::default);

use std::sync::RwLock;

use once_cell::sync::Lazy;

use bevy::{
    prelude::{
        info, AssetEvent, Assets, Deref, DerefMut, EventReader, Handle, Plugin, Res, ResMut, Shader,
    },
    reflect::TypeUuid,
    utils::HashMap,
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
    }
}

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct ModuleMetas {
    pub metas: HashMap<Handle<ModuleMeta>, ModuleMeta>,
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
) where
    V: RustGpuEntryPoint,
    F: RustGpuEntryPoint,
{
    let mut reload = false;

    for event in shader_events.iter() {
        info!("Shader event {event:#?}");
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                for id in materials.ids().collect::<Vec<_>>() {
                    let material_handle = Handle::<RustGpuMaterial<V, F>>::weak(id);
                    if let Some(material) = materials.get_mut(&material_handle) {
                        if material.vertex_shader == Some(handle.clone_weak())
                            || material.fragment_shader == Some(handle.clone_weak())
                        {
                            {
                                let mut module_meta = MODULE_METAS.write().unwrap();
                                if let Some(meta) = &material.vertex_meta {
                                    info!("Removing vertex meta {meta:?}");
                                    module_meta.remove(meta);
                                }

                                if let Some(meta) = &material.fragment_meta {
                                    info!("Removing fragment meta {meta:?}");
                                    module_meta.remove(meta);
                                }
                            }

                            reload = true
                        }
                    }
                }
            }
            _ => (),
        }
    }

    for event in module_meta_events.iter() {
        info!("Module meta event {event:#?}");
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                info!("Updating shader meta for {handle:?}");
                if let Some(asset) = assets.get(handle) {
                    info!("Shader meta: {asset:#?}");
                    MODULE_METAS
                        .write()
                        .unwrap()
                        .insert(handle.clone(), asset.clone());
                    reload = true;
                }
            }
            AssetEvent::Removed { handle } => {
                info!("Removing shader meta for {handle:?}");
                MODULE_METAS.write().unwrap().remove(handle);
                reload = true;
            }
        }
    }

    if reload {
        for (_, material) in materials.iter_mut() {
            material.iteration += 1;
        }
    }
}
