use std::{
    collections::HashSet,
    sync::{LockResult, Mutex, MutexGuard},
};

pub struct LogSet {
    inner: Mutex<HashSet<String>>,
}

impl LogSet {
    fn new() -> Self {
        Self {
            inner: Mutex::new(HashSet::new()),
        }
    }

    pub fn lock(&self) -> LockResult<MutexGuard<'_, HashSet<String>>> {
        self.inner.lock()
    }
}

lazy_static::lazy_static! {
    pub static ref __SET: LogSet = LogSet::new();
}

#[macro_export]
macro_rules! log_once {
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => {
        let message = format!($($arg)+);
        if log::log_enabled!(target: $target, $lvl) {
            let event = format!("[{}::{}] {}", $target, $lvl, message);
            use crate::utils::log_once::__SET;
            if __SET.lock().unwrap().insert(event){
                log::log!(target: $target, $lvl, "{}", message);
            }
        }
    };
    ($lvl:expr, $($arg:tt)+) => ($crate::log_once!(target: module_path!(), $lvl,  $($arg)+));
}

macro_rules! gen_lvl {
    (@inner $macro: ident, $lvl: expr, $d:tt) => {
        #[macro_export]
        macro_rules! $macro{
            (target: $d target:expr, $d($d arg:tt)*) => (
                $crate::log_once!(target: $d target, $lvl, $d ($d arg)*);
            );
            ($d ($d arg:tt)*) => (
                $crate::log_once!($lvl, $d ($d arg)*);
            );
        }

        pub use $macro;

    };
    ($macro_n: ident, $lvl: expr) => {
        gen_lvl!(@inner $macro_n, $lvl, $);
    };
}

gen_lvl!(error_once, log::Level::Error);
gen_lvl!(warn_once, log::Level::Warn);
