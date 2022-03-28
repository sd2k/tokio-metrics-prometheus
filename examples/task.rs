use std::time::Duration;

use prometheus_client::{encoding::text::encode, registry::Registry};
use tokio_metrics_prometheus::TaskCollector;

#[tokio::main]
async fn main() {
    // construct a metrics monitor
    let metrics_monitor = tokio_metrics::TaskMonitor::new();
    // construct a Prometheus registry.
    let mut registry = Registry::default();
    TaskCollector::new("my_task", metrics_monitor.clone()).register(&mut registry);

    // encode Prometheus metrics and print to stdout every 1s.
    // generally you would want to do this in a /metrics handler.
    {
        tokio::spawn(async move {
            let mut buffer = vec![];
            loop {
                encode(&mut buffer, &registry).unwrap();
                println!("{}", String::from_utf8(buffer.clone()).unwrap());
                buffer.clear();
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    // instrument some tasks and await them
    // note that the same TaskMonitor can be used for multiple tasks
    tokio::join![
        metrics_monitor.instrument(do_work()),
        metrics_monitor.instrument(do_work()),
        metrics_monitor.instrument(do_work())
    ];
}

async fn do_work() {
    for _ in 0..25 {
        tokio::task::yield_now().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
