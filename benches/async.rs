extern crate criterion;
extern crate futures;
extern crate reducer;

use criterion::*;
use futures::executor::*;
use reducer::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
struct BlackBox;

impl<T> Reducer<T> for BlackBox {
    fn reduce(&mut self, val: T) {
        black_box(val);
    }
}

impl<T: Copy> Reactor<T> for BlackBox {
    type Output = T;

    fn react(&self, &val: &T) -> Self::Output {
        black_box(val)
    }
}

fn bench(c: &mut Criterion) {
    let store = AsyncStore::new(BlackBox, BlackBox);
    let mut executor = ThreadPoolBuilder::new().create().unwrap();
    let handle = store.spawn(&mut executor).unwrap();

    let mut d1 = handle.clone();
    let mut d2 = handle.clone();

    c.bench_functions(
        "dispatch",
        vec![
            Fun::new("sync", move |b, &x| b.iter(|| block_on(d1.dispatch(x)))),
            Fun::new("async", move |b, &x| b.iter(|| d2.dispatch(x))),
        ],
        42,
    );
}

criterion_group!(benches, bench);
criterion_main!(benches);
