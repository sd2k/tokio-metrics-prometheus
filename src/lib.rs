#![deny(missing_docs)]
//! Prometheus collectors for tokio task and runtime metrics.

macro_rules! register {
    ( $registry:expr, $metric:expr ) => {
        $registry.register($metric.name(), $metric.description(), Box::new($metric));
    };
}

mod runtime;
mod task;

pub use runtime::RuntimeCollector;
pub use task::{TaskCollector, MONITOR};

#[cfg(test)]
// We can't know what values many of these metrics will take, but assert what we can.
fn assert_approx_output(actual: &str, expected: &str) {
    let lines = actual.lines().zip(expected.lines());
    for (actual, expected) in lines {
        if !actual.starts_with('#') && (actual.contains("_ratio") || actual.contains("_seconds")) {
            let (actual_series, actual_value) = actual.split_once(' ').unwrap();
            let (expected_series, _) = expected.split_once(' ').unwrap();
            // The series name should be identical.
            assert_eq!(actual_series, expected_series);
            // The value will almost certainly be different, but it should always
            // be a non-zero float if we're measuring things correctly.
            assert_ne!(actual_value.trim().parse::<f64>().unwrap(), 0f64);
        } else {
            // The encoded line should be exactly what we expect.
            assert_eq!(actual, expected);
        }
    }
}
