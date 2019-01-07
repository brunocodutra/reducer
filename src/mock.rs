#![cfg(test)]

use reactor::Reactor;
use reducer::Reducer;
use std::marker::PhantomData;

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

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct MockReactor<S>(PhantomData<S>);

impl<S: Clone> Reactor<S> for MockReactor<S> {
    type Output = S;

    fn react(&self, state: &S) -> Self::Output {
        state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn react() {
        let reactor = MockReactor::default();

        assert_eq!(reactor.react(&5), 5);
        assert_eq!(reactor.react(&1), 1);
        assert_eq!(reactor.react(&3), 3);
    }

    #[test]
    fn reduce() {
        let mut state = MockReducer::default();

        state.reduce(5);
        state.reduce(1);
        state.reduce(3);

        assert_eq!(state, MockReducer::new(vec![5, 1, 3]));
    }
}
