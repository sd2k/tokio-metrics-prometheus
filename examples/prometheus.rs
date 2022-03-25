use std::time::Duration;

use prometheus_client::{encoding::text::encode, registry::Registry};
use tokio_metrics::RuntimeMonitor;
use tokio_metrics_prometheus::PrometheusCollector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let handle = tokio::runtime::Handle::current();

    // encode Prometheus metrics and print to stdout every 1s.
    // generally you would want to do this in a /metrics handler.
    {
        let runtime_monitor = RuntimeMonitor::new(&handle);
        let collector = PrometheusCollector::new(&runtime_monitor);
        let mut registry = Registry::default();
        collector.register(&mut registry);
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

    // await some tasks
    tokio::join![do_work(), do_work(), do_work(),];

    Ok(())
}

async fn do_work() {
    for _ in 0..25 {
        tokio::task::yield_now().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
