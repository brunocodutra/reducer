#![cfg(test)]

use dispatcher::Dispatcher;
use reactor::Reactor;
use reducer::Reducer;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct NotSync<T: Clone>(RefCell<T>);

impl<T: Clone> NotSync<T> {
    pub fn new(t: T) -> Self {
        NotSync(RefCell::new(t))
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct NotSyncOrSend<T: Clone>(Rc<T>);

impl<T: Clone> NotSyncOrSend<T> {
    pub fn new(t: T) -> Self {
        NotSyncOrSend(Rc::new(t))
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct MockReducer<A: 'static> {
    actions: Vec<A>,
}

impl<A> MockReducer<A> {
    pub fn new(actions: Vec<A>) -> Self {
        Self { actions }
    }
}

impl<A> Reducer<A> for MockReducer<A> {
    fn reduce(&mut self, action: A) {
        self.actions.push(action);
    }
}

impl<A: Clone> Reducer<NotSync<A>> for MockReducer<A> {
    fn reduce(&mut self, action: NotSync<A>) {
        self.actions.push((*action.0.borrow()).clone());
    }
}

impl<A: Clone> Reducer<NotSyncOrSend<A>> for MockReducer<A> {
    fn reduce(&mut self, action: NotSyncOrSend<A>) {
        self.actions.push((*action.0).clone());
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct MockReactor<S>(PhantomData<S>);

impl<S: Clone> Reactor<S> for MockReactor<S> {
    type Output = S;

    fn react(&self, state: &S) -> Self::Output {
        state.clone()
    }
}

impl<S: Clone> Reactor<NotSync<S>> for MockReactor<S> {
    type Output = S;

    fn react(&self, action: &NotSync<S>) -> Self::Output {
        (*action.0.borrow()).clone()
    }
}

impl<S: Clone> Reactor<NotSyncOrSend<S>> for MockReactor<S> {
    type Output = S;

    fn react(&self, action: &NotSyncOrSend<S>) -> Self::Output {
        (*action.0).clone()
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct MockDispatcher<A>(PhantomData<A>);

impl<A> Dispatcher<A> for MockDispatcher<A> {
    type Output = A;

    fn dispatch(&mut self, state: A) -> Self::Output {
        state
    }
}

impl<A: Clone> Dispatcher<NotSync<A>> for MockDispatcher<A> {
    type Output = A;

    fn dispatch(&mut self, action: NotSync<A>) -> Self::Output {
        action.0.into_inner()
    }
}

impl<A: Clone> Dispatcher<NotSyncOrSend<A>> for MockDispatcher<A> {
    type Output = A;

    fn dispatch(&mut self, action: NotSyncOrSend<A>) -> Self::Output {
        (*action.0).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn react() {
        let reactor = MockReactor::default();

        assert_eq!(reactor.react(&5), 5);
        assert_eq!(reactor.react(&NotSync::new(1)), 1);
        assert_eq!(reactor.react(&NotSyncOrSend::new(3)), 3);
    }

    #[test]
    fn reduce() {
        let mut state = MockReducer::default();

        state.reduce(5);
        state.reduce(NotSync::new(1));
        state.reduce(NotSyncOrSend::new(3));

        assert_eq!(state, MockReducer::new(vec![5, 1, 3]));
    }

    #[test]
    fn dispatch() {
        let mut dispatcher = MockDispatcher::default();

        assert_eq!(dispatcher.dispatch(5), 5);
        assert_eq!(dispatcher.dispatch(NotSync::new(1)), 1);
        assert_eq!(dispatcher.dispatch(NotSyncOrSend::new(3)), 3);
    }
}
