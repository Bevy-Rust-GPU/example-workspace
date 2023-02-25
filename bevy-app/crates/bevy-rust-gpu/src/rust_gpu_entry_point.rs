pub type RustGpuEntryPointName = &'static str;
pub type RustGpuEntryPointMappings =
    &'static [(&'static [(&'static str, &'static str)], &'static str)];

pub trait RustGpuEntryPoint: 'static + Send + Sync {
    const NAME: &'static str;
    const PARAMETERS: RustGpuEntryPointMappings;

    fn is_defined(shader_defs: &Vec<String>, def: &String) -> bool {
        let def = def.into();
        shader_defs.contains(def)
    }

    fn build(shader_defs: &Vec<String>) -> String {
        let mut entry_point = Self::NAME.to_string();

        for (defined, undefined) in Self::PARAMETERS.iter() {
            entry_point += "__";
            entry_point += if let Some(mapping) = defined.iter().find_map(|(def, mapping)| {
                if Self::is_defined(shader_defs, &def.to_string()) {
                    Some(mapping)
                } else {
                    None
                }
            }) {
                mapping
            } else {
                undefined
            };
        }

        entry_point
    }
}

impl RustGpuEntryPoint for () {
    const NAME: &'static str = "";
    const PARAMETERS: RustGpuEntryPointMappings = &[];
}

/// Manually compose `bevy_render` shader defs that aren't available during specialization
pub fn rust_gpu_shader_defs() -> Vec<String> {
    // NO_STORAGE_BUFFERS_SUPPORT is implied for now,
    // since `rust-gpu` can't bind read-only storage buffers yet
    let extra_defs = vec!["NO_STORAGE_BUFFERS_SUPPORT".to_string()];

    // Same webgl logic as `bevy_render/src/render_resource/pipeline_cache.rs`
    #[cfg(feature = "webgl")]
    let extra_defs = extra_defs + vec!["NO_TEXTURE_ARRAYS_SUPPORT", "SIXTEEN_BYTE_ALIGNMENT"];

    extra_defs
}
