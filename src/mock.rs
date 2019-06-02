#![cfg(test)]

use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use std::{cell::RefCell, marker::PhantomData};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Never {}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct MockReducer<A: 'static>(Vec<A>);

impl<A: 'static> MockReducer<A> {
    pub fn new(actions: impl Into<Vec<A>>) -> Self {
        MockReducer(actions.into())
    }
}

impl<A> Reducer<A> for MockReducer<A> {
    fn reduce(&mut self, action: A) {
        self.0.push(action);
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct MockReactor<S>(RefCell<Vec<S>>);

impl<S> MockReactor<S> {
    pub fn new(states: impl Into<Vec<S>>) -> Self {
        MockReactor(RefCell::new(states.into()))
    }
}

impl<S: Clone> Reactor<S> for MockReactor<S> {
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
impl<S: Unpin> Sink<S> for MockReactor<S> {
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

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct MockDispatcher<A>(PhantomData<A>);

impl<A> Dispatcher<A> for MockDispatcher<A> {
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
        fn reduce(actions: Vec<char>) {
            let mut reducer = MockReducer::default();

            for &action in &actions {
                reducer.reduce(action);
            }

            assert_eq!(reducer, MockReducer::new(actions));
        }
    }

    proptest! {
        #[test]
        fn react(states: Vec<char>) {
            let reactor = MockReactor::default();

            for action in &states {
                assert_eq!(reactor.react(action), Ok(()));
            }

            assert_eq!(reactor, MockReactor::new(states));
        }
    }

    proptest! {
        #[test]
        fn dispatch(actions: Vec<char>) {
            let mut dispatcher = MockDispatcher::default();

            for action in actions {
                assert_eq!(dispatcher.dispatch(action), action);
            }
        }
    }
}
