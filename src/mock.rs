#![cfg(test)]
#![allow(clippy::unit_arg)]

use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use proptest_derive::Arbitrary;
use std::{cell::RefCell, marker::PhantomData};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Never {}

pub type Mock<T> = TaggedMock<T, ()>;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct TaggedMock<T, Tag>(RefCell<Vec<T>>, PhantomData<Tag>);

impl<T, Tag> TaggedMock<T, Tag> {
    pub fn new(states: impl Into<Vec<T>>) -> Self {
        TaggedMock(RefCell::new(states.into()), PhantomData)
    }
}

impl<A: 'static, Tag: 'static> Reducer<A> for TaggedMock<A, Tag> {
    fn reduce(&mut self, action: A) {
        self.0.borrow_mut().push(action);
    }
}

impl<S: Clone, Tag> Reactor<S> for TaggedMock<S, Tag> {
    type Output = Result<(), Never>;

    fn react(&self, state: &S) -> Self::Output {
        self.0.borrow_mut().push(state.clone());
        Ok(())
    }
}

impl<A, Tag> Dispatcher<A> for TaggedMock<A, Tag> {
    type Output = Result<(), Never>;

    fn dispatch(&mut self, action: A) -> Self::Output {
        self.0.borrow_mut().push(action);
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
        self.get_mut().0.borrow_mut().push(value);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::*;

    proptest! {
        #[test]
        fn reduce(actions: Vec<u8>) {
            let mut reducer = Mock::<_>::default();

            for &action in &actions {
                reducer.reduce(action);
            }

            assert_eq!(reducer, Mock::new(actions));
        }
    }

    proptest! {
        #[test]
        fn react(states: Vec<u8>) {
            let reactor = Mock::<_>::default();

            for action in &states {
                assert_eq!(reactor.react(action), Ok(()));
            }

            assert_eq!(reactor, Mock::new(states));
        }
    }

    proptest! {
        #[test]
        fn dispatch(actions: Vec<u8>) {
            let mut dispatcher = Mock::<_>::default();

            for &action in &actions {
                assert_eq!(dispatcher.dispatch(action), Ok(()));
            }

            assert_eq!(dispatcher, Mock::new(actions));
        }
    }
}
