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
                let val = (self.get_raw() as f32 / max as f32).clamp(0.0, 1.0);
                let width = ((n - 1) as f32 * val).round() as usize;
                write!(
                    f,
                    "[{empty:=>width_left$}>{empty:.<width_right$}] {val:.1}%",
                    empty = "",
                    width_left = width,
                    width_right = n - 1 - width,
                    val = 100. * val
                )
            }
        }
    }
}
