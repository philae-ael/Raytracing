use std::{fmt::Display, sync::atomic};

#[derive(Debug)]
pub struct Progress {
    current: atomic::AtomicUsize,
    max: usize,
}

impl Progress {
    pub fn new(max: usize) -> Self {
        Self {
            current: Default::default(),
            max,
        }
    }
    pub fn inc(&self) -> usize {
        self.current.fetch_add(1, atomic::Ordering::SeqCst)
    }
    pub fn get_raw(&self) -> usize {
        self.current.load(atomic::Ordering::SeqCst)
    }
    pub fn get(&self) -> f32 {
        self.get_raw() as f32 / self.max as f32
    }
    pub fn print(&self) {
        use std::io::Write;
        print!("\r{}", self);
        let _ = std::io::stdout().flush();
    }
    pub fn done(&self) -> bool {
        self.get_raw() >= self.max
    }
}

impl Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = 50;
        let val = self.get();
        let width = (n as f32 * val).round() as usize;
        write!(
            f,
            "[{empty:=>width_left$}>{empty:.<width_right$}] {val:.1}%",
            empty = "",
            width_left = width,
            width_right = n - width,
            val = 100. * val
        )
    }
}
