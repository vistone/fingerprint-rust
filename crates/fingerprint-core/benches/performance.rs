// ! performance基准testing
// ! validateHTTP/1.1、HTTP/2、HTTP/3of响应timeperformance指标

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Instant;

fn benchmark_ja4_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ja4_generation");

    // testingJA4fingerprintgenerateperformance
    group.bench_function("ja4_fingerprint_generation", |b| {
        b.iter(|| {
            use fingerprint_core::ja4::JA4;

            let start = Instant::now();
            let _ja4 = black_box(JA4::generate(
                't',
                "1.3",
                true,
                &[0x1301, 0x1302],
                &[0x0000, 0x0010, 0x0023],
                Some("h2"),
                &[0x0403, 0x0804],
            ));
            let duration = start.elapsed();

            // JA4generate应该快速完成
            assert!(
                duration.as_micros() < 1000,
                "JA4 generation should be under 1ms"
            );
            duration
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_ja4_generation);
criterion_main!(benches);
