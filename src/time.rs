#[derive(Debug, Default, Clone, Copy)]
pub struct TimeState {
    /// Milliseconds since the previous frame.
    pub dt_ms: i64,
    /// Milliseconds since the program started.
    pub elapsed_ms: i64,
    /// Frames elapsed since startup.
    pub frame: i64,
}
