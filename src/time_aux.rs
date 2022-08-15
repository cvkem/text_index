use std::time::{Duration, Instant};


pub fn duration_sec_float(dur: Duration) -> f64 {
    dur.as_secs() as f64
        + dur.subsec_nanos() as f64 * 1e-9
}


pub fn saved_duration(t1: Duration, t2: Duration) -> (Duration, f64) {
    if t1 >  t2 {
        let saved = t1 - t2;
         (saved, -100_f64 *duration_sec_float(saved) / duration_sec_float(t1))
    } else {
        let extra = t2 - t1;
         (extra, 100_f64 *duration_sec_float(extra) / duration_sec_float(t1))
    }
}
