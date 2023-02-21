#[repr(C)]
pub struct Globals {
    // The time since startup in seconds
    // Wraps to 0 after 1 hour.
    pub time: f32,
    // The delta time since the previous frame in seconds
    pub delta_time: f32,
    // Frame count since the start of the app.
    // It wraps to zero when it reaches the maximum value of a u32.
    pub frame_count: u32,

    #[cfg(feature = "sixteen_byte_alignment")]
    // WebGL2 structs must be 16 byte aligned.
    _wasm_padding: f32,
}

