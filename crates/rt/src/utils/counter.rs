use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
pub enum Counter {
    CounterU64(CounterU64),
    CounterTime(CounterTime),
}

impl Counter {
    pub fn format(&self) -> String {
        match self {
            Counter::CounterU64(a) => a.format(),
            Counter::CounterTime(a) => a.format(),
        }
    }
}

#[derive(Default)]
pub struct CounterU64 {
    atomic: AtomicU64,
}
impl CounterU64 {
    pub const fn new() -> Self {
        Self {
            atomic: AtomicU64::new(0),
        }
    }
    pub fn inc(&self) {
        self.atomic.fetch_add(1, Ordering::Relaxed); // Adding one is associative and commutative
    }
    pub fn value(&self) -> u64 {
        self.atomic.load(Ordering::Acquire)
    }
    fn format(&self) -> String {
        format!("{}", self.value())
    }
}
#[derive(Default)]
pub struct CounterTime {
    // Note: can only store up to 213503 days of duration or 564 years
    nanos: AtomicU64,
}

impl CounterTime {
    pub const fn new() -> Self {
        Self {
            nanos: AtomicU64::new(0),
        }
    }
    pub fn add(&self, dur: std::time::Duration) {
        self.nanos
            .fetch_add(dur.as_nanos() as u64, Ordering::Relaxed);
    }
    pub fn value(&self) -> std::time::Duration {
        std::time::Duration::from_nanos(self.nanos.load(Ordering::Acquire))
    }
    pub fn format(&self) -> String {
        super::timer::format_elapsed(self.value())
    }
}

pub fn report_counters() {
    let counters = __COUNTERS.lock().unwrap();
    for (counter_name, counter) in counters.iter() {
        log::log!(target: "counter_report", log::Level::Info, "{}: {} ", counter_name, counter.format())
    }
}

lazy_static::lazy_static! {
    pub static ref __COUNTERS: Mutex<HashMap<&'static str, Arc<Counter>>> = Mutex::new(HashMap::new());
}

pub fn insert_counter(descr: &'static str, counter: Counter) -> Arc<Counter> {
    let mut counters = __COUNTERS.lock().unwrap();
    let arc = Arc::new(counter);

    counters.entry(descr).or_insert(arc).clone()
}

#[macro_export]
macro_rules! counter {
    ($descr:literal) => {
        if cfg!(feature = "counter") {
            use $crate::utils::counter::{insert_counter, lazy_static, Counter, CounterU64};
            lazy_static::lazy_static! {
                static ref COUNTER_REF: std::sync::Arc<Counter> = {
                    insert_counter($descr, Counter::CounterU64(CounterU64::new()))
                };
            }

            if let Counter::CounterU64(c) = &**COUNTER_REF {
                c.inc();
            } else {
                panic!("WTF")
            };
        };
    };
}

pub use counter;
// Reexport for ease of use
pub use lazy_static;
