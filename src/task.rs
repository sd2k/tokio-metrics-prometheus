use std::{
    borrow::Cow,
    sync::{Arc, RwLock},
};

use prometheus_client::{
    encoding::text::{EncodeMetric, Encoder},
    metrics::MetricType,
    registry::Registry,
};
use tokio_metrics::{TaskMetrics, TaskMonitor};

/// A wrapper around a task monitor, and the most recent metrics.
struct CachedMonitor {
    monitor: TaskMonitor,
    current: TaskMetrics,
}

impl CachedMonitor {
    fn new(monitor: TaskMonitor) -> Self {
        let current = monitor.cumulative();
        Self { monitor, current }
    }

    fn refresh(&mut self) {
        self.current = self.monitor.cumulative();
    }

    fn get(&self) -> &TaskMetrics {
        &self.current
    }
}

/// The name of the label used to identify a monitor.
pub static MONITOR: &str = "monitor";

/// This macro creates a struct representing one of the
/// `tokio_metrics::TaskMetrics` metrics.
///
/// Each struct contains a thread-safe reference to the `CachedMonitor`
/// which it will use to actually get its current value at encode-time.
///
/// Only the first one of the patterns here (requiring `first` as the last argument)
/// produces a metric that actually triggers a _refresh_ of the metrics
/// (by calling the inner `CachedMonitor`'s `refresh` method). The others will
/// just read the current value assuming it to be true.
/// This means that the first metric to be registered should
/// be the one created using this `first` pattern, and _only_ this metric!
///
/// Users won't have to worry about this, fortunately.
macro_rules! metric_struct {
    ( $struct_name:ident, $metric_name:ident, $description:expr, $metric_type:expr, first ) => {
        struct $struct_name {
            state: Arc<RwLock<CachedMonitor>>,
            monitor_name: Cow<'static, str>,
        }

        impl $struct_name {
            fn new(state: Arc<RwLock<CachedMonitor>>, monitor_name: Cow<'static, str>) -> Self {
                Self {
                    state,
                    monitor_name,
                }
            }

            fn name(&self) -> &'static str {
                stringify!($metric_name)
            }

            fn description(&self) -> &'static str {
                $description
            }
        }

        impl EncodeMetric for $struct_name {
            fn encode(&self, mut encoder: Encoder) -> Result<(), std::io::Error> {
                self.state.write().unwrap().refresh();
                encoder
                    .with_label_set(&(MONITOR, self.monitor_name.as_ref()))
                    .no_suffix()?
                    .no_bucket()?
                    .encode_value(self.state.read().unwrap().get().$metric_name as u64)?
                    .no_exemplar()
            }

            fn metric_type(&self) -> MetricType {
                $metric_type
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                f.debug_struct(stringify!($struct_name))
                    .field("state", &"<monitor>")
                    .field("monitor_name", &self.monitor_name)
                    .finish()
            }
        }
    };
    ( $struct_name:ident, $metric_name:ident, $description:expr, $metric_type:expr$(,)? ) => {
        struct $struct_name {
            state: Arc<RwLock<CachedMonitor>>,
            monitor_name: Cow<'static, str>,
        }

        impl $struct_name {
            fn new(state: Arc<RwLock<CachedMonitor>>, monitor_name: Cow<'static, str>) -> Self {
                Self {
                    state,
                    monitor_name,
                }
            }

            fn name(&self) -> &'static str {
                stringify!($metric_name)
            }

            fn description(&self) -> &'static str {
                $description
            }
        }

        impl EncodeMetric for $struct_name {
            fn encode(&self, mut encoder: Encoder) -> Result<(), std::io::Error> {
                let metrics = self.state.read().unwrap();
                encoder
                    .with_label_set(&(MONITOR, self.monitor_name.as_ref()))
                    .no_suffix()?
                    .no_bucket()?
                    .encode_value(metrics.get().$metric_name as u64)?
                    .no_exemplar()
            }

            fn metric_type(&self) -> MetricType {
                $metric_type
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                f.debug_struct(stringify!($struct_name))
                    .field("state", &"<monitor>")
                    .field("monitor_name", &self.monitor_name)
                    .finish()
            }
        }
    };
    ( $struct_name:ident, $metric_name:ident, $description:expr, $metric_type:expr, $extract:expr$(,)?) => {
        struct $struct_name {
            state: Arc<RwLock<CachedMonitor>>,
            monitor_name: Cow<'static, str>,
        }

        impl $struct_name {
            fn new(state: Arc<RwLock<CachedMonitor>>, monitor_name: Cow<'static, str>) -> Self {
                Self {
                    state,
                    monitor_name,
                }
            }

            fn name(&self) -> &'static str {
                stringify!($metric_name)
            }

            fn description(&self) -> &'static str {
                $description
            }
        }

        impl EncodeMetric for $struct_name {
            fn encode(&self, mut encoder: Encoder) -> Result<(), std::io::Error> {
                let metrics = self.state.read().unwrap();
                encoder
                    .with_label_set(&(MONITOR, self.monitor_name.as_ref()))
                    .no_suffix()?
                    .no_bucket()?
                    .encode_value($extract(metrics.get()))?
                    .no_exemplar()
            }

            fn metric_type(&self) -> MetricType {
                $metric_type
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                f.debug_struct(stringify!($struct_name))
                    .field("state", &"<monitor>")
                    .field("monitor_name", &self.monitor_name)
                    .finish()
            }
        }
    };
    ( $struct_name:ident, $metric_name:ident, $description:expr, $metric_type:expr, $extract_min:expr, $extract_max:expr, $extract_total:expr$(,)?) => {
        struct $struct_name {
            state: Arc<RwLock<CachedMonitor>>,
            monitor_name: Cow<'static, str>,
        }

        impl $struct_name {
            fn new(state: Arc<RwLock<CachedMonitor>>, monitor_name: Cow<'static, str>) -> Self {
                Self {
                    state,
                    monitor_name,
                }
            }

            fn name(&self) -> &'static str {
                stringify!($metric_name)
            }

            fn description(&self) -> &'static str {
                $description
            }
        }

        impl EncodeMetric for $struct_name {
            fn encode(&self, mut encoder: Encoder) -> Result<(), std::io::Error> {
                let metrics = self.state.read().unwrap();
                encoder
                    .with_label_set(&MinMaxTotalLabels {
                        measurement: Measurement::min,
                    })
                    .with_label_set(&(MONITOR, self.monitor_name.as_ref()))
                    .no_suffix()?
                    .no_bucket()?
                    .encode_value($extract_min(metrics.get()))?
                    .no_exemplar()?;
                encoder
                    .with_label_set(&(MONITOR, self.monitor_name.as_ref()))
                    .with_label_set(&MinMaxTotalLabels {
                        measurement: Measurement::max,
                    })
                    .no_suffix()?
                    .no_bucket()?
                    .encode_value($extract_max(metrics.get()))?
                    .no_exemplar()?;
                encoder
                    .with_label_set(&(MONITOR, self.monitor_name.as_ref()))
                    .with_label_set(&MinMaxTotalLabels {
                        measurement: Measurement::total,
                    })
                    .no_suffix()?
                    .no_bucket()?
                    .encode_value($extract_total(metrics.get()))?
                    .no_exemplar()
            }

            fn metric_type(&self) -> MetricType {
                $metric_type
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                f.debug_struct(stringify!($struct_name))
                    .field("state", &"<monitor>")
                    .field("monitor_name", &self.monitor_name)
                    .finish()
            }
        }
    };
}

// Create the various metric structs.
// Note that only `instrumented_count` includes 'first'. This metric is later on
// also registered first.

// Raw metrics.
metric_struct!(
    InstrumentedCount,
    instrumented_count,
    "The number of tasks instrumented",
    MetricType::Counter,
    first
);
metric_struct!(
    DroppedCount,
    dropped_count,
    "The number of tasks dropped",
    MetricType::Counter,
);
metric_struct!(
    FirstPollCount,
    first_poll_count,
    "The number of tasks polled for the first time",
    MetricType::Counter,
);
metric_struct!(
    FirstPollDelay,
    first_poll_delay_seconds,
    "The total duration elapsed between the instant tasks are instrumented, and the instant they are first polled",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_first_poll_delay.as_secs_f64(),
);
metric_struct!(
    IdledCount,
    idled_count,
    "The total number of times that tasks idled, waiting to be awoken",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_idled_count,
);
metric_struct!(
    IdleDuration,
    idle_duration_seconds,
    "The total duration that tasks idled",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_idle_duration.as_secs_f64(),
);
metric_struct!(
    ScheduledCount,
    scheduled_count,
    "The total number of times that tasks were awoken (and then, presumably, scheduled for execution)",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_scheduled_count,
);
metric_struct!(
    ScheduledDuration,
    scheduled_duration_seconds,
    "The total duration that tasks spent waiting to be polled after awakening",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_scheduled_duration.as_secs_f64(),
);
metric_struct!(
    PollCount,
    poll_count,
    "The total number of times that tasks were polled",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_poll_count,
);
metric_struct!(
    PollDuration,
    poll_duration_seconds,
    "The total duration elapsed during polls",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_poll_duration.as_secs_f64(),
);
metric_struct!(
    FastPollCount,
    fast_poll_count,
    "The total number of times that polling tasks completed swiftly",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_fast_poll_count,
);
metric_struct!(
    FastPollDuration,
    fast_poll_duration_seconds,
    "The total duration of fast polls",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_fast_poll_duration.as_secs_f64(),
);
metric_struct!(
    SlowPollCount,
    slow_poll_count,
    "The total number of times that polling tasks completed slowly",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_slow_poll_count,
);
metric_struct!(
    SlowPollDuration,
    slow_poll_duration_seconds,
    "The total duration of slow polls",
    MetricType::Counter,
    |metrics: &TaskMetrics| metrics.total_slow_poll_duration.as_secs_f64(),
);

// Derived metrics.
metric_struct!(
    MeanFirstPollDelay,
    mean_first_poll_delay_seconds,
    "The mean duration elapsed between the instant tasks are instrumented, and the instant they are first polled",
    MetricType::Gauge,
    |metrics: &TaskMetrics| metrics.mean_first_poll_delay().as_secs_f64(),
);
metric_struct!(
    MeanIdleDuration,
    mean_idle_duration_seconds,
    "The mean duration of idles",
    MetricType::Gauge,
    |metrics: &TaskMetrics| metrics.mean_idle_duration().as_secs_f64(),
);
metric_struct!(
    MeanScheduledDuration,
    mean_scheduled_duration_seconds,
    "The mean duration that tasks spent waiting to be executed after awakening",
    MetricType::Gauge,
    |metrics: &TaskMetrics| metrics.mean_scheduled_duration().as_secs_f64(),
);
metric_struct!(
    MeanPollDuration,
    mean_poll_duration_seconds,
    "The mean duration of polls",
    MetricType::Gauge,
    |metrics: &TaskMetrics| metrics.mean_poll_duration().as_secs_f64(),
);
metric_struct!(
    SlowPollRatio,
    slow_poll_ratio,
    "The ratio between the number of polls categorized as slow or fast",
    MetricType::Gauge,
    |metrics: &TaskMetrics| metrics.slow_poll_ratio(),
);
metric_struct!(
    MeanFastPollDuration,
    mean_fast_poll_duration_seconds,
    "The mean duration of fast_polls",
    MetricType::Gauge,
    |metrics: &TaskMetrics| metrics.mean_fast_poll_duration().as_secs_f64(),
);
metric_struct!(
    MeanSlowPollDuration,
    mean_slow_poll_duration_seconds,
    "The mean duration of slow_polls",
    MetricType::Gauge,
    |metrics: &TaskMetrics| metrics.mean_slow_poll_duration().as_secs_f64(),
);

/// A Prometheus collector for a tokio task.
#[derive(Debug)]
#[must_use]
pub struct TaskCollector {
    // Raw metrics.
    instrumented_count: InstrumentedCount,
    dropped_count: DroppedCount,
    first_poll_count: FirstPollCount,
    first_poll_delay: FirstPollDelay,
    idled_count: IdledCount,
    idle_duration: IdleDuration,
    scheduled_count: ScheduledCount,
    scheduled_duration: ScheduledDuration,
    poll_count: PollCount,
    poll_duration: PollDuration,
    fast_poll_count: FastPollCount,
    fast_poll_duration: FastPollDuration,
    slow_poll_count: SlowPollCount,
    slow_poll_duration: SlowPollDuration,

    // Derived metrics.
    mean_first_poll_delay: MeanFirstPollDelay,
    mean_idle_duration: MeanIdleDuration,
    mean_scheduled_duration: MeanScheduledDuration,
    mean_poll_duration: MeanPollDuration,
    slow_poll_ratio: SlowPollRatio,
    mean_fast_poll_duration: MeanFastPollDuration,
    mean_slow_poll_duration: MeanSlowPollDuration,
}

impl TaskCollector {
    /// Create a new `TaskCollector ` to gather metrics for the given `TaskMonitor`.
    pub fn new(name: &str, monitor: TaskMonitor) -> Self {
        let name: Cow<str> = name.to_string().into();
        let cached = Arc::new(RwLock::new(CachedMonitor::new(monitor)));
        Self {
            instrumented_count: InstrumentedCount::new(Arc::clone(&cached), name.clone()),
            dropped_count: DroppedCount::new(Arc::clone(&cached), name.clone()),
            first_poll_count: FirstPollCount::new(Arc::clone(&cached), name.clone()),
            first_poll_delay: FirstPollDelay::new(Arc::clone(&cached), name.clone()),
            idled_count: IdledCount::new(Arc::clone(&cached), name.clone()),
            idle_duration: IdleDuration::new(Arc::clone(&cached), name.clone()),
            scheduled_count: ScheduledCount::new(Arc::clone(&cached), name.clone()),
            scheduled_duration: ScheduledDuration::new(Arc::clone(&cached), name.clone()),
            poll_count: PollCount::new(Arc::clone(&cached), name.clone()),
            poll_duration: PollDuration::new(Arc::clone(&cached), name.clone()),
            fast_poll_count: FastPollCount::new(Arc::clone(&cached), name.clone()),
            fast_poll_duration: FastPollDuration::new(Arc::clone(&cached), name.clone()),
            slow_poll_count: SlowPollCount::new(Arc::clone(&cached), name.clone()),
            slow_poll_duration: SlowPollDuration::new(Arc::clone(&cached), name.clone()),

            mean_first_poll_delay: MeanFirstPollDelay::new(Arc::clone(&cached), name.clone()),
            mean_idle_duration: MeanIdleDuration::new(Arc::clone(&cached), name.clone()),
            mean_scheduled_duration: MeanScheduledDuration::new(Arc::clone(&cached), name.clone()),
            mean_poll_duration: MeanPollDuration::new(Arc::clone(&cached), name.clone()),
            slow_poll_ratio: SlowPollRatio::new(Arc::clone(&cached), name.clone()),
            mean_fast_poll_duration: MeanFastPollDuration::new(Arc::clone(&cached), name.clone()),
            mean_slow_poll_duration: MeanSlowPollDuration::new(cached, name),
        }
    }

    /// Register the metrics for this `TaskCollector ` into a registry.
    ///
    /// The given registry must have `Box<EncodeMetric>` or `Box<SendEncodeMetric>`
    /// as it's `M` generic paraneter..
    pub fn register(self, registry: &mut Registry) {
        register!(registry, self.instrumented_count);
        register!(registry, self.dropped_count);
        register!(registry, self.first_poll_count);
        register!(registry, self.first_poll_delay);
        register!(registry, self.idled_count);
        register!(registry, self.idle_duration);
        register!(registry, self.scheduled_count);
        register!(registry, self.scheduled_duration);
        register!(registry, self.poll_count);
        register!(registry, self.poll_duration);
        register!(registry, self.fast_poll_count);
        register!(registry, self.fast_poll_duration);
        register!(registry, self.slow_poll_count);
        register!(registry, self.slow_poll_duration);
        register!(registry, self.mean_first_poll_delay);
        register!(registry, self.mean_idle_duration);
        register!(registry, self.mean_scheduled_duration);
        register!(registry, self.mean_poll_duration);
        register!(registry, self.slow_poll_ratio);
        register!(registry, self.mean_fast_poll_duration);
        register!(registry, self.mean_slow_poll_duration);
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    #[tokio::test]
    async fn output_approx_eq() {
        let monitor = tokio_metrics::TaskMonitor::new();
        let mut registry = prometheus_client::registry::Registry::default();

        crate::TaskCollector::new("my_monitor", monitor.clone()).register(&mut registry);

        tokio::spawn(monitor.instrument(async {
            for i in 0..25 {
                tokio::task::yield_now().await;
                if i % 5 == 0 {
                    std::thread::sleep(Duration::from_millis(10));
                } else {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }))
        .await
        .unwrap();

        let mut buffer = vec![];
        prometheus_client::encoding::text::encode(&mut buffer, &registry).unwrap();
        let expected = r#"# HELP instrumented_count The number of tasks instrumented.
# TYPE instrumented_count counter
instrumented_count{monitor="my_monitor"} 1
# HELP dropped_count The number of tasks dropped.
# TYPE dropped_count counter
dropped_count{monitor="my_monitor"} 1
# HELP first_poll_count The number of tasks polled for the first time.
# TYPE first_poll_count counter
first_poll_count{monitor="my_monitor"} 1
# HELP first_poll_delay_seconds The total duration elapsed between the instant tasks are instrumented, and the instant they are first polled.
# TYPE first_poll_delay_seconds counter
first_poll_delay_seconds{monitor="my_monitor"} 0.000007
# HELP idled_count The total number of times that tasks idled, waiting to be awoken.
# TYPE idled_count counter
idled_count{monitor="my_monitor"} 20
# HELP idle_duration_seconds The total duration that tasks idled.
# TYPE idle_duration_seconds counter
idle_duration_seconds{monitor="my_monitor"} 0.296472543
# HELP scheduled_count The total number of times that tasks were awoken (and then, presumably, scheduled for execution).
# TYPE scheduled_count counter
scheduled_count{monitor="my_monitor"} 45
# HELP scheduled_duration_seconds The total duration that tasks spent waiting to be polled after awakening.
# TYPE scheduled_duration_seconds counter
scheduled_duration_seconds{monitor="my_monitor"} 0.000163977
# HELP poll_count The total number of times that tasks were polled.
# TYPE poll_count counter
poll_count{monitor="my_monitor"} 46
# HELP poll_duration_seconds The total duration elapsed during polls.
# TYPE poll_duration_seconds counter
poll_duration_seconds{monitor="my_monitor"} 0.000160446
# HELP fast_poll_count The total number of times that polling tasks completed swiftly.
# TYPE fast_poll_count counter
fast_poll_count{monitor="my_monitor"} 41
# HELP fast_poll_duration_seconds The total duration of fast polls.
# TYPE fast_poll_duration_seconds counter
fast_poll_duration_seconds{monitor="my_monitor"} 0.000160446
# HELP slow_poll_count The total number of times that polling tasks completed slowly.
# TYPE slow_poll_count counter
slow_poll_count{monitor="my_monitor"} 5
# HELP slow_poll_duration_seconds The total duration of slow polls.
# TYPE slow_poll_duration_seconds counter
slow_poll_duration_seconds{monitor="my_monitor"} 0.060772541
# HELP mean_first_poll_delay_seconds The mean duration elapsed between the instant tasks are instrumented, and the instant they are first polled.
# TYPE mean_first_poll_delay_seconds gauge
mean_first_poll_delay_seconds{monitor="my_monitor"} 0.000007
# HELP mean_idle_duration_seconds The mean duration of idles.
# TYPE mean_idle_duration_seconds gauge
mean_idle_duration_seconds{monitor="my_monitor"} 0.011858901
# HELP mean_scheduled_duration_seconds The mean duration that tasks spent waiting to be executed after awakening.
# TYPE mean_scheduled_duration_seconds gauge
mean_scheduled_duration_seconds{monitor="my_monitor"} 0.000003279
# HELP mean_poll_duration_seconds The mean duration of polls.
# TYPE mean_poll_duration_seconds gauge
mean_poll_duration_seconds{monitor="my_monitor"} 0.000003146
# HELP slow_poll_ratio The ratio between the number of polls categorized as slow or fast.
# TYPE slow_poll_ratio gauge
slow_poll_ratio{monitor="my_monitor"} 0.10869565217391304
# HELP mean_fast_poll_duration_seconds The mean duration of fast_polls.
# TYPE mean_fast_poll_duration_seconds gauge
mean_fast_poll_duration_seconds{monitor="my_monitor"} 0.000003146
# HELP mean_slow_poll_duration_seconds The mean duration of slow_polls.
# TYPE mean_slow_poll_duration_seconds gauge
mean_slow_poll_duration_seconds{monitor="my_monitor"} 0.012232616
# EOF"#;
        crate::assert_approx_output(&String::from_utf8(buffer).unwrap(), expected);
    }
}
