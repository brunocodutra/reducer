use criterion::*;
use futures::{never::Never, prelude::*};
use reducer::{Dispatcher, Reactor, Reducer, Store};
use smol::{block_on, spawn};
use std::iter::repeat;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
struct BlackBox;

impl<T> Reducer<T> for BlackBox {
    fn reduce(&mut self, val: T) {
        black_box(val);
    }
}

impl<T: Copy> Reactor<T> for BlackBox {
    type Error = Never;

    fn react(&mut self, &val: &T) -> Result<(), Self::Error> {
        black_box(val);
        Ok(())
    }
}

impl<T: Copy> Sink<T> for BlackBox {
    type Error = Never;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, val: T) -> Result<(), Self::Error> {
        black_box(val);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

fn dispatch(c: &mut Criterion) {
    const ACTIONS: usize = 50;

    c.bench(
        "async/dispatch",
        Benchmark::new(ACTIONS.to_string(), |b| {
            b.iter_batched(
                move || {
                    let store = Store::new(BlackBox, BlackBox);
                    let (task, dispatcher) = store.into_task();
                    (dispatcher, spawn(task))
                },
                |(mut dispatcher, handle)| {
                    for a in 0..ACTIONS {
                        dispatcher.dispatch(a).unwrap();
                    }

                    block_on(dispatcher.close()).unwrap();
                    block_on(handle).unwrap();
                },
                BatchSize::SmallInput,
            );
        })
        .throughput(Throughput::Elements(ACTIONS as u64)),
    );
}

fn sink(c: &mut Criterion) {
    const ACTIONS: usize = 500;

    c.bench(
        "async/sink",
        Benchmark::new(ACTIONS.to_string(), |b| {
            b.iter_batched(
                move || {
                    let store = Store::new(BlackBox, BlackBox);
                    let (task, dispatcher) = store.into_task();
                    (dispatcher, spawn(task))
                },
                |(dispatcher, handle)| {
                    for (a, mut d) in repeat(dispatcher).enumerate().take(ACTIONS) {
                        spawn(async move {
                            d.send(a).await.unwrap();
                        })
                        .detach();
                    }

                    block_on(handle).unwrap();
                },
                BatchSize::SmallInput,
            );
        })
        .throughput(Throughput::Elements(ACTIONS as u64)),
    );
}
criterion_group!(benches, dispatch, sink);
criterion_main!(benches);
