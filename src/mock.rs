#![cfg(test)]

use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use std::{cell::RefCell, marker::PhantomData};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Never {}

pub type MockReducer<A> = TaggedMockReducer<A, ()>;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct TaggedMockReducer<A, Tag>(Vec<A>, PhantomData<Tag>);

impl<A, Tag> TaggedMockReducer<A, Tag> {
    pub fn new(actions: impl Into<Vec<A>>) -> Self {
        TaggedMockReducer(actions.into(), PhantomData)
    }
}

impl<A: 'static, Tag: 'static> Reducer<A> for TaggedMockReducer<A, Tag> {
    fn reduce(&mut self, action: A) {
        self.0.push(action);
    }
}

pub type MockReactor<S> = TaggedMockReactor<S, ()>;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct TaggedMockReactor<S, Tag>(RefCell<Vec<S>>, PhantomData<Tag>);

impl<S, Tag> TaggedMockReactor<S, Tag> {
    pub fn new(states: impl Into<Vec<S>>) -> Self {
        TaggedMockReactor(RefCell::new(states.into()), PhantomData)
    }
}

impl<S: Clone, Tag> Reactor<S> for TaggedMockReactor<S, Tag> {
    type Output = Result<(), Never>;

    fn react(&self, state: &S) -> Self::Output {
        self.0.borrow_mut().push(state.clone());
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
impl<S: Unpin, Tag: Unpin> Sink<S> for TaggedMockReactor<S, Tag> {
    type SinkError = Never;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, state: S) -> Result<(), Self::SinkError> {
        self.get_mut().0.borrow_mut().push(state);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::SinkError>> {
        Poll::Ready(Ok(()))
    }
}

pub type MockDispatcher<A> = TaggedMockDispatcher<A, ()>;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TaggedMockDispatcher<A, Tag>(PhantomData<(A, Tag)>);

impl<A, Tag> Dispatcher<A> for TaggedMockDispatcher<A, Tag> {
    type Output = A;

    fn dispatch(&mut self, action: A) -> Self::Output {
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::*;

    proptest! {
        #[test]
        fn reduce(actions: Vec<u8>) {
            let mut reducer = MockReducer::<_>::default();

            for &action in &actions {
                reducer.reduce(action);
            }

            assert_eq!(reducer, MockReducer::new(actions));
        }
    }

    proptest! {
        #[test]
        fn react(states: Vec<u8>) {
            let reactor = MockReactor::<_>::default();

            for action in &states {
                assert_eq!(reactor.react(action), Ok(()));
            }

            assert_eq!(reactor, MockReactor::new(states));
        }
    }

    proptest! {
        #[test]
        fn dispatch(actions: Vec<u8>) {
            let mut dispatcher = MockDispatcher::<_>::default();

            for action in actions {
                assert_eq!(dispatcher.dispatch(action), action);
            }
        }
    }
}
