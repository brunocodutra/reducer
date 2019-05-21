use criterion::*;
use futures::executor::*;
use futures::future::*;
use futures::task::*;
use reducer::*;
use std::iter::repeat;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Action<T>(T);

impl<T: 'static> Reducer<Action<T>> for Action<T> {
    fn reduce(&mut self, val: Action<T>) {
        *self = val;
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
struct BlackBox;

impl<T: Copy> Reactor<T> for BlackBox {
    type Output = T;

    fn react(&self, &val: &T) -> Self::Output {
        black_box(val)
    }
}

const ACTIONS: usize = 500;
const CONCURRENCY: usize = 100;

fn bench(c: &mut Criterion) {
    c.bench(
        "async/dispatch",
        Benchmark::new(
            format!("{}x{}", CONCURRENCY, ACTIONS / CONCURRENCY),
            move |b| {
                let mut executor = ThreadPool::new().unwrap();
                let store = Async::new(Store::new(Action(0), BlackBox));
                let dispatcher = store.spawn(&mut executor).unwrap();

                b.iter(|| {
                    block_on(join_all(
                        repeat(dispatcher.clone())
                            .take(CONCURRENCY)
                            .flat_map(|mut d| {
                                (0..ACTIONS / CONCURRENCY).map(move |a| d.dispatch(Action(a)))
                            })
                            .map(|f| executor.spawn_with_handle(f))
                            .map(Result::unwrap),
                    ));
                });
            },
        )
        .throughput(Throughput::Elements(ACTIONS as u32)),
    );
}

criterion_group!(benches, bench);
criterion_main!(benches);
