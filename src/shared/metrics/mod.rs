use once_cell::sync::Lazy;
use prometheus::{
    Encoder, HistogramVec, IntCounterVec, IntGauge, IntGaugeVec, Registry, TextEncoder,
    register_histogram_vec_with_registry, register_int_counter_vec_with_registry,
    register_int_gauge_vec_with_registry, register_int_gauge_with_registry,
};

static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

pub static JOBS_PUBLISHED_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec_with_registry!(
        "jobs_published_total",
        "Number of jobs published to the queue",
        &["job_type"],
        REGISTRY.clone()
    )
    .expect("jobs_published_total metric")
});

pub static JOBS_CONSUMED_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec_with_registry!(
        "jobs_consumed_total",
        "Number of jobs consumed by workers",
        &["job_type"],
        REGISTRY.clone()
    )
    .expect("jobs_consumed_total metric")
});

pub static JOBS_SUCCEEDED_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec_with_registry!(
        "jobs_succeeded_total",
        "Number of successful jobs",
        &["job_type"],
        REGISTRY.clone()
    )
    .expect("jobs_succeeded_total metric")
});

pub static JOBS_FAILED_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec_with_registry!(
        "jobs_failed_total",
        "Number of failed jobs",
        &["job_type"],
        REGISTRY.clone()
    )
    .expect("jobs_failed_total metric")
});

pub static JOBS_RETRIED_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec_with_registry!(
        "jobs_retried_total",
        "Number of retried jobs",
        &["job_type"],
        REGISTRY.clone()
    )
    .expect("jobs_retried_total metric")
});

pub static JOBS_PROCESSING_SECONDS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec_with_registry!(
        "job_processing_seconds",
        "Time spent processing jobs",
        &["job_type"],
        REGISTRY.clone()
    )
    .expect("job_processing_seconds metric")
});

pub static JOBS_IN_PROGRESS: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec_with_registry!(
        "jobs_in_progress",
        "Current jobs being processed",
        &["job_type"],
        REGISTRY.clone()
    )
    .expect("jobs_in_progress metric")
});

pub static QUEUE_DEPTH_ESTIMATE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge_with_registry!(
        "queue_depth_estimate",
        "Estimated queue depth from database state",
        REGISTRY.clone()
    )
    .expect("queue_depth_estimate metric")
});

pub fn record_job_published(job_type: &str) {
    JOBS_PUBLISHED_TOTAL.with_label_values(&[job_type]).inc();
}

pub fn record_job_consumed(job_type: &str) {
    JOBS_CONSUMED_TOTAL.with_label_values(&[job_type]).inc();
}

pub fn record_job_succeeded(job_type: &str, processing_seconds: f64) {
    JOBS_SUCCEEDED_TOTAL.with_label_values(&[job_type]).inc();
    JOBS_PROCESSING_SECONDS
        .with_label_values(&[job_type])
        .observe(processing_seconds);
}

pub fn record_job_failed(job_type: &str, processing_seconds: f64) {
    JOBS_FAILED_TOTAL.with_label_values(&[job_type]).inc();
    JOBS_PROCESSING_SECONDS
        .with_label_values(&[job_type])
        .observe(processing_seconds);
}

pub fn record_job_retried(job_type: &str) {
    JOBS_RETRIED_TOTAL.with_label_values(&[job_type]).inc();
}

pub fn job_started(job_type: &str) {
    JOBS_IN_PROGRESS.with_label_values(&[job_type]).inc();
}

pub fn job_finished(job_type: &str) {
    JOBS_IN_PROGRESS.with_label_values(&[job_type]).dec();
}

pub fn set_queue_depth(depth: i64) {
    QUEUE_DEPTH_ESTIMATE.set(depth);
}

pub fn gather_metrics() -> Result<String, std::io::Error> {
    let metric_families = REGISTRY.gather();
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .map_err(std::io::Error::other)?;
    String::from_utf8(buffer).map_err(std::io::Error::other)
}
