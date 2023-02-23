pub trait WasmPadding {}

/// No WASM padding
impl WasmPadding for () {}

/// Single float WASM padding
impl WasmPadding for f32 {}

#[repr(C)]
pub struct Globals<P: WasmPadding> {
    // The time since startup in seconds
    // Wraps to 0 after 1 hour.
    pub time: f32,
    // The delta time since the previous frame in seconds
    pub delta_time: f32,
    // Frame count since the start of the app.
    // It wraps to zero when it reaches the maximum value of a u32.
    pub frame_count: u32,

    // WebGL2 structs must be 16 byte aligned.
    _wasm_padding: P,
}
