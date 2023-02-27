use std::{error::Error, path::Path};

use futures::{
    channel::mpsc::{self, channel, Receiver, UnboundedSender},
    executor::{self, ThreadPool},
    select, SinkExt, StreamExt,
};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use spirv_builder::{MetadataPrintout, SpirvBuilder};

use tracing::{error, info};

fn build_shader() {
    SpirvBuilder::new("crates/shader", "spirv-unknown-spv1.5")
        .print_metadata(MetadataPrintout::None)
        .build()
        .ok();
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Default::default(),
    )?;

    Ok((watcher, rx))
}

async fn async_watch<P: AsRef<Path>>(
    path: P,
    mut change_tx: UnboundedSender<()>,
) -> Result<(), Box<dyn Error>> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(_) => {
                change_tx.send(()).await.unwrap();
            }
            Err(e) => error!("Watch error: {:?}", e),
        }
    }

    Ok(())
}

fn main() {
    tracing_subscriber::fmt().init();

    println!();
    info!("Shader Builder");

    let pool = ThreadPool::new().expect("Failed to build pool");
    let (change_tx, mut change_rx) = mpsc::unbounded::<()>();
    let (build_tx, mut build_rx) = mpsc::unbounded::<()>();

    let mut building = false;

    let fut_values = async move {
        info!("Watching for changes...");
        println!();
        {
            pool.spawn_ok(async move {
                async_watch(".", change_tx).await.unwrap();
            });
        }

        loop {
            let mut file_change = change_rx.next();
            let mut build_complete = build_rx.next();
            select! {
                _ = file_change => {
                    if !building {
                        building = true;
                        info!("Building shader...");
                        pool.spawn_ok({
                            let mut build_tx = build_tx.clone();
                            async move {
                                build_shader();
                                build_tx.send(()).await.unwrap();
                            }
                        })
                    }
                },
                _ = build_complete => {
                    info!("Build complete!");
                    println!();
                    building = false;
                }
            };
        }
    };

    executor::block_on(fut_values);
}
