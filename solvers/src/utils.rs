pub struct NamedTimer {
    name: &'static str,
    #[cfg(not(target_arch = "wasm32"))]
    timer: std::time::Instant,
}

impl NamedTimer {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            #[cfg(not(target_arch = "wasm32"))]
            timer: std::time::Instant::now(),
        }
    }
}

impl Drop for NamedTimer {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        eprintln!("{}: (timer not available on WASM)", self.name);
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!(
            "{}: {} seconds",
            self.name,
            self.timer.elapsed().as_secs_f32()
        );
    }
}
