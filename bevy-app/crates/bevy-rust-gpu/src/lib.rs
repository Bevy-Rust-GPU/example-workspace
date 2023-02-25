use bevy::prelude::{
    Bundle, ComputedVisibility, GlobalTransform, Handle, Mesh, Transform, Visibility,
};
use prelude::{RustGpuEntryPoint, RustGpuMaterial};

pub mod rust_gpu_entry_point;
pub mod rust_gpu_material;

pub mod rust_gpu_shader_meta {
    pub static MODULE_METAS: Lazy<RwLock<ModuleMetas>> = Lazy::new(Default::default);

    use std::sync::RwLock;

    use once_cell::sync::Lazy;

    use bevy::{
        asset::HandleId,
        prelude::{
            default, AssetEvent, Assets, Deref, DerefMut, EventReader, Handle, Plugin, Res, ResMut,
            Resource, CoreStage,
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

            app.add_system_to_stage(CoreStage::PreUpdate, load_shader_meta);
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

    pub fn load_shader_meta(
        mut events: EventReader<AssetEvent<ModuleMeta>>,
        assets: Res<Assets<ModuleMeta>>,
    ) {
        for event in events.iter() {
            match event {
                AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                    let asset = assets.get(handle).unwrap();
                    MODULE_METAS
                        .write()
                        .unwrap()
                        .insert(handle.clone(), asset.clone());
                }
                AssetEvent::Removed { handle } => {
                    MODULE_METAS.write().unwrap().remove(handle);
                }
            }
        }
    }

    #[derive(Debug, Resource)]
    pub struct RustGpuMaterials<V: RustGpuEntryPoint, F: RustGpuEntryPoint> {
        pub materials: HashMap<Handle<RustGpuMaterial<V, F>>, RustGpuMaterial<V, F>>,
    }

    impl<V, F> RustGpuMaterials<V, F>
    where
        V: RustGpuEntryPoint,
        F: RustGpuEntryPoint,
    {
        pub fn add(
            &mut self,
            materials: &Assets<RustGpuMaterial<V, F>>,
            material: RustGpuMaterial<V, F>,
        ) -> Handle<RustGpuMaterial<V, F>> {
            let handle_id = HandleId::random::<RustGpuMaterial<V, F>>();
            let mut handle = Handle::<RustGpuMaterial<V, F>>::weak(handle_id);
            handle.make_strong(&materials);
            self.materials.insert(handle.clone(), material);
            handle
        }
    }

    impl<V, F> Default for RustGpuMaterials<V, F>
    where
        V: RustGpuEntryPoint,
        F: RustGpuEntryPoint,
    {
        fn default() -> Self {
            RustGpuMaterials {
                materials: default(),
            }
        }
    }

    impl<V, F> Clone for RustGpuMaterials<V, F>
    where
        V: RustGpuEntryPoint,
        F: RustGpuEntryPoint,
    {
        fn clone(&self) -> Self {
            RustGpuMaterials {
                materials: self.materials.clone(),
            }
        }
    }

    pub fn rust_gpu_materials<V, F>(
        mut storage: ResMut<RustGpuMaterials<V, F>>,
        mut materials: ResMut<Assets<RustGpuMaterial<V, F>>>,
    ) where
        V: RustGpuEntryPoint,
        F: RustGpuEntryPoint,
    {
        let module_meta = MODULE_METAS.read().unwrap();

        let mut to_apply = vec![];
        for (handle, material) in storage.materials.iter() {
            match (&material.vertex_meta, &material.fragment_meta) {
                (None, None) => to_apply.push(handle.clone()),
                (None, Some(meta)) | (Some(meta), None) => {
                    if module_meta.contains_key(&meta) {
                        to_apply.push(handle.clone());
                    }
                }
                (Some(meta_v), Some(meta_f)) => {
                    if module_meta.contains_key(&meta_v) && module_meta.contains_key(&meta_f) {
                        to_apply.push(handle.clone());
                    }
                }
            }
        }

        for (handle, material) in storage
            .materials
            .drain_filter(|handle, _| to_apply.contains(&handle))
        {
            materials.set_untracked(handle, material);
        }
    }
}

pub mod prelude;
