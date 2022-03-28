macro_rules! register {
    ( $registry:expr, $metric:expr ) => {
        $registry.register($metric.name(), $metric.description(), Box::new($metric));
    };
}

mod runtime;
mod task;

pub use runtime::RuntimeCollector;
pub use task::TaskCollector;
