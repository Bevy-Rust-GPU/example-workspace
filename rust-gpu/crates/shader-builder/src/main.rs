use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use notify_debouncer_mini::{
    new_debouncer, notify::RecursiveMode, DebounceEventResult, DebouncedEventKind,
};

use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn build_shader() {
    SpirvBuilder::new("crates/shader", "spirv-unknown-spv1.5")
        .print_metadata(MetadataPrintout::None)
        .build()
        .ok();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let running = Arc::new(AtomicBool::new(true));

    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut debouncer =
        new_debouncer(
            Duration::from_secs(1),
            None,
            |res: DebounceEventResult| match res {
                Ok(events) => {
                    for event in events {
                        println!("{event:#?}");
                        let build = match event.kind {
                            DebouncedEventKind::Any => true,
                            _ => false,
                        };

                        if build {
                            build_shader()
                        }
                    }
                }
                Err(e) => eprintln!("watch error: {:?}", e),
            },
        )?;

    debouncer.watcher().watch(
        Path::new("crates"),
        RecursiveMode::Recursive,
    )?;

    while running.load(Ordering::SeqCst) {}

    Ok(())
}
