use criterion::{BenchmarkId, criterion_group, criterion_main, Criterion};
use stream_histogram::Histogram;
use rand::Rng;

fn criterion_benchmark(c: &mut Criterion) {

    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("histogram");
    for size in [10usize,20,40,60,80,100].iter(){
        let mut histogram = Histogram::new(*size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| histogram.add(rng.gen::<f64>()));
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);