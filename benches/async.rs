#![feature(async_await)]

use criterion::*;
use futures::executor::*;
use futures::sink::*;
use futures::task::*;
use reducer::*;
use std::iter::repeat;
use std::pin::Pin;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
struct BlackBox;

impl<T: 'static> Reducer<T> for BlackBox {
    fn reduce(&mut self, val: T) {
        black_box(val);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Never {}

impl<T: Copy> Reactor<T> for BlackBox {
    type Output = Result<(), Never>;

    fn react(&self, &val: &T) -> Self::Output {
        black_box(val);
        Ok(())
    }
}

impl<T: Copy> Sink<T> for BlackBox {
    type SinkError = Never;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, val: T) -> Result<(), Self::SinkError> {
        black_box(val);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }
}

const ACTIONS: usize = 1000;

fn dispatch(c: &mut Criterion) {
    c.bench(
        "async/dispatch",
        Benchmark::new(ACTIONS.to_string(), |b| {
            let mut executor = ThreadPool::new().unwrap();

            b.iter_batched(
                move || {
                    let store = Store::new(BlackBox, BlackBox);
                    executor.spawn_dispatcher(store).unwrap()
                },
                |(dispatcher, handle)| {
                    for (a, mut d) in repeat(dispatcher).enumerate().take(ACTIONS) {
                        d.dispatch(a).unwrap();
                    }

                    block_on(handle).unwrap();
                },
                BatchSize::SmallInput,
            );
        })
        .throughput(Throughput::Elements(ACTIONS as u32)),
    );
}

fn sink(c: &mut Criterion) {
    c.bench(
        "async/sink",
        Benchmark::new(ACTIONS.to_string(), |b| {
            let mut executor = ThreadPool::new().unwrap();

            b.iter_batched(
                move || {
                    let store = Store::new(BlackBox, BlackBox);
                    let (dispatcher, handle) = executor.spawn_dispatcher(store).unwrap();
                    (dispatcher, handle, executor.clone())
                },
                |(dispatcher, handle, mut executor)| {
                    for (a, mut d) in repeat(dispatcher).enumerate().take(ACTIONS) {
                        executor
                            .spawn(async move {
                                d.send(a).await.unwrap();
                            })
                            .unwrap();
                    }

                    block_on(handle).unwrap();
                },
                BatchSize::SmallInput,
            );
        })
        .throughput(Throughput::Elements(ACTIONS as u32)),
    );
}
criterion_group!(benches, dispatch, sink);
criterion_main!(benches);
