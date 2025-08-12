use criterion::{Criterion, criterion_group, criterion_main};
use threadid::{LiveThreadId, StdThreadId, UniqueThreadId};

fn std_current(c: &mut Criterion) {
    c.bench_function("std::thread::current().id()", |x| {
        x.iter(|| std::thread::current().id())
    });
}

fn std_id_current(c: &mut Criterion) {
    c.bench_function("threadid::StdThreadId::current()", |x| {
        x.iter(|| StdThreadId::current())
    });
}

fn unique_id_current(c: &mut Criterion) {
    c.bench_function("threadid::UniqueThreadId::current()", |x| {
        x.iter(|| UniqueThreadId::current())
    });
}

fn live_id_current(c: &mut Criterion) {
    c.bench_function("threadid::LiveThreadId::current()", |x| {
        x.iter(|| LiveThreadId::current())
    });
}

criterion_group!(access, std_current, std_id_current, unique_id_current, live_id_current);
criterion_main!(access);
