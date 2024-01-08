use std::ops::{Deref, DerefMut};

pub struct TimedResult<T> {
    pub res: T,
    pub elapsed: std::time::Duration,
}

impl<T> Deref for TimedResult<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.res
    }
}

impl<T> DerefMut for TimedResult<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.res
    }
}

pub fn timed_scope<R, F: FnOnce() -> R>(f: F) -> TimedResult<R> {
    let begin = std::time::Instant::now();
    let res = f();

    let elapsed = begin.elapsed();

    TimedResult { res, elapsed }
}

pub fn timed_scope_log<R, F: FnOnce() -> R>(label: &'static str, f: F) -> TimedResult<R> {
    let time_res = timed_scope(f);
    log::log!(target: "scoped timer", log::Level::Info, "{}: {}", label, format_elapsed(time_res.elapsed));
    time_res
}

pub fn format_elapsed(elapsed: std::time::Duration) -> String {
    if elapsed < std::time::Duration::from_millis(1) {
        // Micro s
        let micro = elapsed.as_secs_f32() * 1000. * 1000.;
        format!("{micro:.7}µs")
    } else if elapsed < std::time::Duration::from_secs(1) {
        // Milli s
        let milli = elapsed.as_secs_f32() * 1000.;
        format!("{milli:.7}ms")
    } else if elapsed < std::time::Duration::from_secs(60) {
        // Seconds
        let s = elapsed.as_secs_f32();
        format!("{s:.3}s")
    } else {
        // Minutes and more
        let elapsed_secs = elapsed.as_secs_f32();
        let elapsed_minutes = elapsed_secs / 60.;
        let elapsed_hours = elapsed_minutes / 60.;
        let h = elapsed_hours as u32;
        let m = (elapsed_minutes % 60.0) as u32;
        let s = (elapsed_secs % 60.0) as u32;
        format!("{h}h{m}m{s}s")
    }
}

pub fn timed_scope_accumulate_<R, F: FnOnce() -> R>(timer: &CounterTime, f: F) -> TimedResult<R> {
    let timed_res = timed_scope(f);
    timer.add(timed_res.elapsed);
    timed_res
}

#[macro_export]
macro_rules! timed_scope_accumulate {
    ($descr:literal, $($arg: tt)+) => {
        if cfg!(feature = "counter_time") {
            use $crate::utils::counter::{Counter, CounterTime, insert_counter};
            use $crate::utils::timer::timed_scope_accumulate_;
            lazy_static::lazy_static! {
                static ref COUNTER_REF: std::sync::Arc<Counter> = {
                    insert_counter($descr, Counter::CounterTime(CounterTime::new()))
                };
            }
            if let Counter::CounterTime(c) = &**COUNTER_REF {
                timed_scope_accumulate_(c, $($arg)*).res
            } else {
                panic!("WTF")
            }
        } else {
            #[allow(clippy::redundant_closure_call)]
            ($($arg)+) ()
        }
    };
}

pub use timed_scope_accumulate;

use super::counter::CounterTime;
