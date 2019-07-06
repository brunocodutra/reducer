#![allow(clippy::unit_arg)]

use crate::dispatcher::Dispatcher;
use crate::reactor::Reactor;
use crate::reducer::Reducer;
use derivative::Derivative;
use proptest_derive::Arbitrary;
use std::{collections::HashMap, hash::Hash, marker::PhantomData};

pub use std::{string::String, vec::Vec};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Never {}

pub(crate) type Mock<T, E = Never> = TaggedMock<(), T, E>;

#[derive(Arbitrary, Derivative)]
#[derivative(Debug, Default(bound = ""), Eq, PartialEq, Hash)]
pub(crate) struct TaggedMock<Tag, T, E = Never>
where
    T: Eq + PartialEq + Hash,
{
    calls: Vec<T>,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    generation: usize,
    #[proptest(value = "HashMap::new()")]
    #[derivative(Debug = "ignore", PartialEq = "ignore", Hash = "ignore")]
    results: HashMap<T, E>,
    #[derivative(Debug = "ignore")]
    phantom: PhantomData<Tag>,
}

impl<Tag, T, E> TaggedMock<Tag, T, E>
where
    T: Eq + PartialEq + Hash,
{
    pub(crate) fn calls(&self) -> &[T] {
        &self.calls
    }

    pub(crate) fn generation(&self) -> usize {
        self.generation
    }

    pub(crate) fn fail_if(&mut self, arg: T, error: E) {
        self.results.insert(arg, error);
    }
}

impl<Tag, T, E> TaggedMock<Tag, T, E>
where
    T: Eq + PartialEq + Hash,
    E: Clone,
{
    pub(crate) fn call(&mut self, arg: T) -> Result<(), E> {
        let result = self.results.get(&arg).cloned().map(Err).unwrap_or(Ok(()));
        self.calls.push(arg);
        result
    }
}

impl<Tag, T, E> Clone for TaggedMock<Tag, T, E>
where
    T: Clone + Eq + PartialEq + Hash,
    E: Clone,
{
    fn clone(&self) -> Self {
        Self {
            calls: self.calls.clone(),
            generation: self.generation + 1,
            results: self.results.clone(),
            phantom: PhantomData,
        }
    }
}

impl<Tag, A> Reducer<A> for TaggedMock<Tag, A, Never>
where
    A: Eq + PartialEq + Hash,
{
    fn reduce(&mut self, action: A) {
        self.call(action).ok();
    }
}

impl<Tag, S, E> Reactor<S> for TaggedMock<Tag, S, E>
where
    S: Clone + Eq + PartialEq + Hash,
    E: Clone,
{
    type Error = E;

    fn react(&mut self, state: &S) -> Result<(), Self::Error> {
        self.call(state.clone())
    }
}

impl<Tag, A, E> Dispatcher<A> for TaggedMock<Tag, A, E>
where
    A: Eq + PartialEq + Hash,
    E: Clone,
{
    type Output = Result<(), E>;

    fn dispatch(&mut self, action: A) -> Self::Output {
        self.call(action)
    }
}

#[cfg(feature = "async")]
mod sink {
    use super::*;
    use futures::sink::Sink;
    use futures::task::{Context, Poll};
    use std::pin::Pin;

    impl<Tag, T, E> Sink<T> for TaggedMock<Tag, T, E>
    where
        T: Unpin + Eq + PartialEq + Hash,
        E: Unpin + Clone,
        Tag: Unpin,
    {
        type Error = E;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, value: T) -> Result<(), Self::Error> {
            self.get_mut().call(value)
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
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
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn reducer(actions: Vec<u8>) {
            let mut reducer = Mock::<_>::default();

            for &action in &actions {
                reduce(&mut reducer, action);
            }

            assert_eq!(reducer.calls(), &actions[..]);
        }
    }

    proptest! {
        #[test]
        fn reactor(states: Vec<u8>, error: String) {
            let mut reactor = Mock::default();

            for action in &states {
                assert_eq!(react(&mut reactor, action), Ok(()));
            }

            assert_eq!(reactor.calls(), &states[..]);

            reactor.fail_if(0, &error[..]);
            assert_eq!(react(&mut reactor, &0), Err(&error[..]));
            assert_eq!(react(&mut reactor, &1), Ok(()));
        }
    }

    proptest! {
        #[test]
        fn dispatcher(actions: Vec<u8>, error: String) {
            let mut dispatcher = Mock::default();

            for &action in &actions {
                assert_eq!(dispatch(&mut dispatcher, action), Ok(()));
            }

            assert_eq!(dispatcher.calls(), &actions[..]);

            dispatcher.fail_if(0, &error[..]);
            assert_eq!(dispatch(&mut dispatcher, 0), Err(&error[..]));
            assert_eq!(dispatch(&mut dispatcher, 1), Ok(()));
        }
    }
}
