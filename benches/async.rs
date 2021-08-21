use criterion::*;
use futures::{sink::drain, SinkExt};
use reducer::{AsyncReactor, Dispatcher, Reducer, Store};
use smol::{block_on, spawn};
use std::iter::repeat;

const ACTIONS: u64 = 500;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
struct BlackBox;

impl<T> Reducer<T> for BlackBox {
    fn reduce(&mut self, val: T) {
        black_box(val);
    }
}

fn dispatch(c: &mut Criterion) {
    c.benchmark_group("async")
        .throughput(Throughput::Elements(ACTIONS))
        .bench_function("dispatch", |b| {
            b.iter_batched(
                move || {
                    let reactor = AsyncReactor(drain());
                    let (task, dispatcher) = Store::new(BlackBox, reactor).into_task();
                    (spawn(task), dispatcher)
                },
                |(handle, mut dispatcher)| {
                    for a in 0..ACTIONS {
                        dispatcher.dispatch(a).unwrap();
                    }

                    block_on(dispatcher.close()).unwrap();
                    block_on(handle).unwrap();
                },
                BatchSize::SmallInput,
            );
        });
}

fn sink(c: &mut Criterion) {
    c.benchmark_group("async")
        .throughput(Throughput::Elements(ACTIONS))
        .bench_function("sink", |b| {
            b.iter_batched(
                move || {
                    let reactor = AsyncReactor(drain());
                    let (task, dispatcher) = Store::new(BlackBox, reactor).into_task();
                    (spawn(task), dispatcher)
                },
                |(handle, dispatcher)| {
                    for (a, mut d) in repeat(dispatcher).enumerate().take(ACTIONS as usize) {
                        spawn(async move {
                            d.send(a).await.unwrap();
                        })
                        .detach();
                    }

                    block_on(handle).unwrap();
                },
                BatchSize::SmallInput,
            );
        });
}

criterion_group!(benches, dispatch, sink);
criterion_main!(benches);
