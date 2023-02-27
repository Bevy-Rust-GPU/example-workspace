use std::{
    error::Error,
    path::{Path, PathBuf},
};

use clap::{error::ErrorKind, Parser};

use futures::{
    channel::mpsc::{self, channel, Receiver, UnboundedSender},
    executor::{self, ThreadPool},
    select, SinkExt, StreamExt,
};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use spirv_builder::{
    CompileResult, MetadataPrintout, SpirvBuilder, SpirvBuilderError, SpirvMetadata,
};

use tracing::{error, info};

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

fn spirv_metadata(s: &str) -> Result<SpirvMetadata, clap::Error> {
    match s {
        "none" => Ok(SpirvMetadata::None),
        "name-variables" => Ok(SpirvMetadata::NameVariables),
        "full" => Ok(SpirvMetadata::Full),
        _ => Err(clap::Error::new(ErrorKind::InvalidValue)),
    }
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
struct ShaderBuilder {
    /// Shader crate to compile
    path_to_crate: PathBuf,
    /// rust-gpu compile target
    #[arg(short, long, default_value = "spirv-unknown-spv1.5")]
    target: String,
    #[arg(long, default_value = "false")]
    deny_warnings: bool,
    #[arg(long, default_value = "true")]
    release: bool,
    #[arg(long, default_value = "false")]
    multimodule: bool,
    #[arg(long, value_parser=spirv_metadata, default_value = "none")]
    spirv_metadata: SpirvMetadata,
    #[arg(long, default_value = "false")]
    relax_struct_store: bool,
    #[arg(long, default_value = "false")]
    relax_logical_pointer: bool,
    #[arg(long, default_value = "false")]
    relax_block_layout: bool,
    #[arg(long, default_value = "false")]
    uniform_buffer_standard_layout: bool,
    #[arg(long, default_value = "false")]
    scalar_block_layout: bool,
    #[arg(long, default_value = "false")]
    skip_block_layout: bool,
    #[arg(long, default_value = "false")]
    preserve_bindings: bool,
    /// If set, will watch the provided path and recompile on change
    #[arg(short, long)]
    watch_path: Option<String>,
}

impl ShaderBuilder {
    pub fn build_shader(&self) -> Result<CompileResult, SpirvBuilderError> {
        SpirvBuilder::new(&self.path_to_crate, &self.target)
            .deny_warnings(self.deny_warnings)
            .release(self.release)
            .multimodule(self.multimodule)
            .spirv_metadata(self.spirv_metadata)
            .relax_struct_store(self.relax_struct_store)
            .relax_logical_pointer(self.relax_logical_pointer)
            .relax_block_layout(self.relax_block_layout)
            .uniform_buffer_standard_layout(self.uniform_buffer_standard_layout)
            .scalar_block_layout(self.scalar_block_layout)
            .skip_block_layout(self.skip_block_layout)
            .preserve_bindings(self.preserve_bindings)
            .print_metadata(MetadataPrintout::None)
            .build()
    }
}

fn main() {
    tracing_subscriber::fmt().init();

    let args = ShaderBuilder::parse();

    println!();
    info!("Shader Builder");
    println!();

    info!("Building shader...");
    if args.build_shader().is_ok() {
        info!("Build complete!");
    } else {
        error!("Build failed!");
    }
    println!();

    if args.watch_path.is_none() {
        return;
    };

    let pool = ThreadPool::new().expect("Failed to build pool");
    let (change_tx, mut change_rx) = mpsc::unbounded::<()>();
    let (build_tx, mut build_rx) = mpsc::unbounded::<bool>();

    let mut building = false;

    let fut_values = async move {
        let mut args = args;

        let Some(watch_path) = args.watch_path.take() else {
            unreachable!();
        };

        info!("Watching {watch_path:} for changes...");
        println!();
        {
            pool.spawn_ok(async move {
                async_watch(watch_path, change_tx).await.unwrap();
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
                            let args = args.clone();
                            async move {
                                build_tx.send(args.build_shader().is_ok()).await.unwrap();
                            }
                        })
                    }
                },
                result = build_complete => {
                    let result = result.unwrap();
                    if result {
                        info!("Build complete!");
                    }
                    else {
                        error!("Build failed!");
                    }
                    println!();
                    building = false;
                }
            };
        }
    };

    executor::block_on(fut_values);
}
