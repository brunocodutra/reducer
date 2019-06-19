#![cfg(test)]
#![allow(clippy::unit_arg)]

use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use proptest_derive::Arbitrary;
use std::marker::PhantomData;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Never {}

pub(crate) type Mock<T> = TaggedMock<T, ()>;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub(crate) struct TaggedMock<T, Tag>(Vec<T>, PhantomData<Tag>);

impl<T, Tag> TaggedMock<T, Tag> {
    pub(crate) fn new(states: impl Into<Vec<T>>) -> Self {
        TaggedMock(states.into(), PhantomData)
    }
}

impl<A, Tag> Reducer<A> for TaggedMock<A, Tag> {
    fn reduce(&mut self, action: A) {
        self.0.push(action);
    }
}

impl<S: Clone, Tag> Reactor<S> for TaggedMock<S, Tag> {
    type Error = Never;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        self.0.push(state.clone());
        Ok(())
    }
}

impl<A, Tag> Dispatcher<A> for TaggedMock<A, Tag> {
    type Output = Result<(), Never>;

    fn dispatch(&mut self, action: A) -> Self::Output {
        self.0.push(action);
        Ok(())
    }
}

#[cfg(feature = "async")]
use futures::sink::Sink;

#[cfg(feature = "async")]
use futures::task::{Context, Poll};

#[cfg(feature = "async")]
use std::pin::Pin;

#[cfg(feature = "async")]
impl<T: Unpin, Tag: Unpin> Sink<T> for TaggedMock<T, Tag> {
    type SinkError = Never;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, value: T) -> Result<(), Self::SinkError> {
        self.get_mut().0.push(value);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }
}

pub(crate) fn reduce<R: Reducer<A> + ?Sized, A>(reducer: &mut R, action: A) {
    reducer.reduce(action);
}

pub(crate) fn react<R: Reactor<S> + ?Sized, S>(reactor: &mut R, state: &S) -> Result<(), R::Error> {
    reactor.react(state)
}

pub(crate) fn dispatch<D: Dispatcher<A> + ?Sized, A>(dispatcher: &mut D, action: A) -> D::Output {
    dispatcher.dispatch(action)
}

mod tests {
    use super::*;
    use proptest::*;

    proptest! {
        #[test]
        fn reducer(actions: Vec<u8>) {
            let mut reducer = Mock::<_>::default();

            for &action in &actions {
                reduce(&mut reducer, action);
            }

            assert_eq!(reducer, Mock::new(actions));
        }
    }

    proptest! {
        #[test]
        fn reactor(states: Vec<u8>) {
            let mut reactor = Mock::<_>::default();

            for action in &states {
                assert_eq!(react(&mut reactor, action), Ok(()));
            }

            assert_eq!(reactor, Mock::new(states));
        }
    }

    proptest! {
        #[test]
        fn dispatcher(actions: Vec<u8>) {
            let mut dispatcher = Mock::<_>::default();

            for &action in &actions {
                assert_eq!(dispatch(&mut dispatcher, action), Ok(()));
            }

            assert_eq!(dispatcher, Mock::new(actions));
        }
    }
}
