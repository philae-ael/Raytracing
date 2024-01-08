use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

pub struct Counter {
    atomic: AtomicU64,
}
impl Counter {
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
}

pub fn report_counters() {
    let counters = __COUNTERS.lock().unwrap();
    for (counter_name, counter) in counters.iter() {
        log::log!(target: "counter_report", log::Level::Info, "{}: {} ", counter_name, counter.value())
    }
}

lazy_static::lazy_static! {
    pub static ref __COUNTERS: Mutex<HashMap<&'static str, Arc<Counter>>> = Mutex::new(HashMap::new());
}

#[macro_export]
macro_rules! counter {
    ($descr:literal) => {
        use crate::utils::counter::{Counter, __COUNTERS};
        use std::sync::Arc;
        lazy_static::lazy_static! {
            static ref COUNTER_REF: Arc<Counter> = {
                let mut counters = __COUNTERS.lock().unwrap();
                counters.entry($descr).or_insert(Arc::new(Counter::new())).clone()
            };
        }
        COUNTER_REF.inc();
    };
}

pub use counter;
