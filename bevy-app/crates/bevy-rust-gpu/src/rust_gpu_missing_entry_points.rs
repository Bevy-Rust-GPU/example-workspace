use std::sync::mpsc::{Receiver, SyncSender};

use bevy::{
    prelude::{default, info, CoreStage, Deref, DerefMut, NonSend, Plugin, Res, ResMut, Resource},
    tasks::IoTaskPool,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

pub struct RustGpuMissingEntryPointPlugin;

#[derive(Debug, Deref, DerefMut)]
pub struct MissingEntryPointReceiver(pub Receiver<MissingEntryPoint>);

#[derive(Debug, Clone, Deref, DerefMut, Resource)]
pub struct MissingEntryPointSender(pub SyncSender<MissingEntryPoint>);

#[derive(Debug, Default, Clone, Serialize, Deserialize, Resource)]
pub struct MissingEntryPoints {
    pub entry_points: HashMap<String, Vec<Vec<String>>>,
}

impl Plugin for RustGpuMissingEntryPointPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let (tx, rx) = std::sync::mpsc::sync_channel::<MissingEntryPoint>(32);

        app.world.insert_resource(MissingEntryPointSender(tx));
        app.world
            .insert_non_send_resource(MissingEntryPointReceiver(rx));
        app.world.init_resource::<MissingEntryPoints>();

        app.add_system_to_stage(
            CoreStage::Last,
            |rx: NonSend<MissingEntryPointReceiver>,
             mut missing_entry_points: ResMut<MissingEntryPoints>| {
                while let Ok(missing_entry_point) = rx.try_recv() {
                    if !missing_entry_points
                        .entry_points
                        .contains_key(missing_entry_point.shader)
                    {
                        info!("New entry point");
                        missing_entry_points
                            .entry_points
                            .insert(missing_entry_point.shader.to_string(), default());
                    }

                    let entry = &missing_entry_points.entry_points[missing_entry_point.shader];
                    if !entry.contains(&missing_entry_point.permutation) {
                        info!("New permutation");
                        missing_entry_points
                            .entry_points
                            .get_mut(missing_entry_point.shader)
                            .unwrap()
                            .push(missing_entry_point.permutation);
                    }
                }
            },
        );

        app.add_system_to_stage(
            CoreStage::PreUpdate,
            |missing_entry_points: Res<MissingEntryPoints>| {
                if missing_entry_points.is_changed() {
                    let missing_entry_points = missing_entry_points.clone();
                    let io_pool = IoTaskPool::get();
                    io_pool
                        .spawn(async move {
                            info!("Writing entrypoints.json");
                            let writer = std::fs::File::create(
                                "../rust-gpu/crates/bevy-pbr-rust/entry_points.json",
                            )
                            .unwrap();
                            serde_json::to_writer_pretty(writer, &missing_entry_points).unwrap();
                        })
                        .detach();
                }
            },
        );
    }
}

#[derive(Debug, Default, Clone)]
pub struct MissingEntryPoint {
    pub shader: &'static str,
    pub permutation: Vec<String>,
}
