#[macro_export]
macro_rules! log_once {
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => {
        use std::sync::Once;
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            if log::log_enabled!(target: $target, $lvl) {
                log::log!(target: $target, $lvl, $($arg)+);
            }
        });
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
