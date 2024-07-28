use std::{fmt::Display, sync::atomic};

pub enum MaxProgress {
    Inf,
    Finite(usize),
}

pub struct Progress {
    current: atomic::AtomicUsize,
    max: MaxProgress,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            current: Default::default(),
            max: MaxProgress::Inf,
        }
    }
}

impl Progress {
    pub fn new(max: usize) -> Self {
        Self {
            max: MaxProgress::Finite(max),
            ..Default::default()
        }
    }
    pub fn new_inf() -> Self {
        Self {
            max: MaxProgress::Inf,
            ..Default::default()
        }
    }
    pub fn add(&self, k: usize) -> usize {
        self.current.fetch_add(k, atomic::Ordering::SeqCst)
    }
    pub fn get_raw(&self) -> usize {
        self.current.load(atomic::Ordering::SeqCst)
    }

    pub fn print(&self) {
        print!("\r{self}");
    }
}

impl Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = 50;
        match self.max {
            MaxProgress::Inf => {
                let width = self.get_raw() % (n - 3);
                let val_formated = 'val_formated: {
                    let mut val = self.get_raw() as f32;
                    let mut prec = 0;
                    for suffix in ["", "K", "M", "G", "T"] {
                        if val < 1000. {
                            break 'val_formated format!("{val:.prec$} {suffix}");
                        }
                        prec = 1;
                        val /= 1000.;
                    }
                    format!("{} {}", val, "E")
                };
                write!(
                    f,
                    "[{empty: >width_left$}...{empty: <width_right$}] {val_formated}",
                    empty = "",
                    width_left = width,
                    width_right = n - width,
                )
            }
            MaxProgress::Finite(max) => {
                let percent = (self.get_raw() as f32 / max as f32).clamp(0.0, 1.0);
                PercentBar { percent, width: n }.fmt(f)
            }
        }
    }
}

pub struct PercentBar {
    pub percent: f32,
    pub width: usize,
}

impl Display for PercentBar {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let filled = ((self.width - 1) as f32 * self.percent).round() as usize;
        write!(
            f,
            "[{empty:=>width_left$}>{empty:.<width_right$}] {percent:.1}%",
            empty = "",
            width_left = filled,
            width_right = self.width - 1 - filled,
            percent = 100. * self.percent
        )
    }
}
