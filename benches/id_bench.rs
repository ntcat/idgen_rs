use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::hint::black_box;
use idgen_rs::{id_helper, FastIdGenerator, IGOptions, init_with_capacity};
use std::sync::Once;

/// ✅ 全局幂等初始化：保证整个 Benchmark 进程仅初始化一次，彻底解决多次调用 Panic
static INIT_ONCE: Once = Once::new();

fn init_benchmark_env() {
    INIT_ONCE.call_once(|| {
        // 🔑 压测专用配置：自动分配 12 bits 序列号 (4096 IDs/ms)
        // 原 snowflake_init(1) 仅 6 bits (64 IDs/ms)，会导致单线程 1ms 后强制休眠，测出 15.6 µs 假数据
        init_with_capacity(1, 1024, 1_000_000);
    });
}

fn bench_single_id(c: &mut Criterion) {
    init_benchmark_env();
    let mut group = c.benchmark_group("global_api");
    group.throughput(Throughput::Elements(1));
    group.bench_function("next_id", |b| {
        b.iter(|| id_helper::next_id())
    });
    group.finish();
}

fn bench_batch_100(c: &mut Criterion) {
    init_benchmark_env();
    let mut group = c.benchmark_group("global_api");
    group.throughput(Throughput::Elements(100));
    group.bench_function("next_ids_100", |b| {
        b.iter(|| id_helper::next_ids(100))
    });
    group.finish();
}

fn bench_batch_1000(c: &mut Criterion) {
    init_benchmark_env();
    let mut group = c.benchmark_group("global_api");
    group.throughput(Throughput::Elements(1000));
    group.bench_function("next_ids_1000", |b| {
        b.iter(|| id_helper::next_ids(1000))
    });
    group.finish();
}

fn bench_direct_generator(c: &mut Criterion) {
    // 直接实例化测试：对比全局 API 的 `OnceLock` 获取开销
    // 同样使用高吞吐配置，确保对比公平
    let opts = IGOptions::with_capacity(1, 1024, 1_000_000);
    let gen = FastIdGenerator::new(&opts);

    let mut group = c.benchmark_group("local_instance");
    group.throughput(Throughput::Elements(1));
    group.bench_function("next_id_direct", |b| {
        b.iter(|| gen.next_id())
    });
    group.finish();
}

fn bench_extract_info(c: &mut Criterion) {
    init_benchmark_env();
    let id = id_helper::next_id();
    let mut group = c.benchmark_group("metadata");
    group.throughput(Throughput::Elements(1));
    group.bench_function("extract_id_info", |b| {
        // ✅ 使用 black_box 防止编译器提前缓存解析结果
        b.iter(|| id_helper::extract_id_info(black_box(id)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_single_id,
    bench_batch_100,
    bench_batch_1000,
    bench_direct_generator,
    bench_extract_info
);
criterion_main!(benches);
