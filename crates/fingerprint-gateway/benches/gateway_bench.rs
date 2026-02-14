use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use fingerprint_gateway::{auth::ApiKeyValidator, metrics};

fn bench_api_key_validation(c: &mut Criterion) {
    let validator = ApiKeyValidator::new();

    c.bench_function("api_key_validate_existing", |b| {
        b.iter(|| {
            let _ = validator.validate("sk_test_demo123");
        })
    });

    c.bench_function("api_key_validate_prefix", |b| {
        b.iter(|| {
            let _ = validator.validate("sk_live_unknown");
        })
    });
}

fn bench_metrics_recording(c: &mut Criterion) {
    c.bench_function("metrics_record_http_request", |b| {
        b.iter(|| {
            metrics::record_http_request("GET", "/health", 200);
        })
    });

    c.bench_function("metrics_record_rate_limit_check", |b| {
        b.iter(|| {
            metrics::record_rate_limit_check("Free", true);
        })
    });

    c.bench_function("metrics_gather", |b| {
        b.iter_batched(
            || (),
            |_| {
                let _ = metrics::gather_metrics();
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_request_timer(c: &mut Criterion) {
    c.bench_function("request_timer_observe", |b| {
        b.iter_batched(
            || metrics::RequestTimer::new("GET".to_string(), "/health".to_string()),
            |timer| {
                timer.observe();
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_api_key_validation,
    bench_metrics_recording,
    bench_request_timer
);
criterion_main!(benches);
