use std::{fmt::Display, sync::atomic};

pub enum MaxProgress {
    Inf,
    Finite(usize),
}

pub struct Progress {
    current: atomic::AtomicUsize,
    done: atomic::AtomicBool,
    max: MaxProgress,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            current: Default::default(),
            done: Default::default(),
            max: MaxProgress::Inf,
        }
    }
}

enum DoneState {
    Done,
    FirstDone,
    NotDone,
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
    pub fn inc(&self) -> usize {
        self.current.fetch_add(1, atomic::Ordering::SeqCst)
    }
    pub fn get_raw(&self) -> usize {
        self.current.load(atomic::Ordering::SeqCst)
    }
    pub fn print(&self) {
        use std::io::Write;
        match self.done_state() {
            DoneState::Done => (),
            DoneState::FirstDone => {
                println!("\r{}", self);
                let _ = std::io::stdout().flush();
            }
            DoneState::NotDone => {
                print!("\r{}", self);
                let _ = std::io::stdout().flush();
            }
        }
    }

    fn get_done(&self) -> bool {
        self.done.load(atomic::Ordering::SeqCst)
    }
    fn set_done(&self) {
        self.done.store(true, atomic::Ordering::SeqCst);
    }

    fn done_state(&self) -> DoneState {
        if self.get_done() {
            return DoneState::Done;
        }
        match self.max {
            MaxProgress::Inf => DoneState::NotDone,
            MaxProgress::Finite(m) => {
                if self.get_raw() >= m {
                    self.set_done();
                    DoneState::FirstDone
                } else {
                    DoneState::NotDone
                }
            }
        }
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
                let val = self.get_raw() as f32 / max as f32;
                let width = ((n - 1) as f32 * val).round() as usize;
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
    }
}
