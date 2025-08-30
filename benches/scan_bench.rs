use merkle_kv::{RwLockEngine, KVEngineStoreTrait};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, SamplingMode};
use std::time::Duration;

fn bench_scan(c: &mut Criterion) {
    // Chuẩn bị dataset 1 lần, ngoài vòng đo
    let e = RwLockEngine::new("./bench").unwrap();
    for i in 0..100_000 {
        e.set(format!("user:{i:06}"), "x".into()).unwrap();
    }
    let prefix = String::from("user:12");

    // Nhóm benchmark + cấu hình để đỡ bị cảnh báo thời gian
    let mut g = c.benchmark_group("scan");
    g.sample_size(60)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(2))
        .sampling_mode(SamplingMode::Auto); // giữ Auto là đủ

    g.bench_function(BenchmarkId::new("scan user:12", 100_000), |b| {
        b.iter(|| {
            // 1) che input khỏi optimizer
            let p: &str = black_box(prefix.as_str());
            // 2) gọi scan
            let v = e.scan(p);
            // 3) che output (vd: dùng độ dài) để tránh bị tối ưu bỏ
            black_box(v.len());
        });
    });

    g.finish();
}

criterion_group!(benches, bench_scan);
criterion_main!(benches);
