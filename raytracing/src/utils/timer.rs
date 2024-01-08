pub fn timed_scope<R, F: FnOnce() -> R>(label: &'static str, f: F) -> R{
    let begin = std::time::Instant::now();
    let res = f();

    let elapsed = begin.elapsed();
    log::log!(target: "scoped timer", log::Level::Info, "{}: {}", label, format_elapsed(elapsed));

    res 
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
        format!("{h}H{m}m{s}s")
    }
}
